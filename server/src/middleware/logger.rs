use std::{
    fs::{self, File},
    io::Write,
    net::SocketAddr,
    path::Path,
};

use axum::{
    extract::ConnectInfo,
    http::{header::LOCATION, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Local};
use colored::Colorize;

const LOG_NAME: &'static str = "logs/%Y-%m-%d.log";
static mut LOG_FILE: Option<LogFile> = None;
struct LogFile {
    file: File,
    time: DateTime<Local>,
}

impl LogFile {
    fn write(time: &DateTime<Local>, s: &[u8]) {
        let log = unsafe {
            // 日志文件不存在或者日期更新 生成新日志文件
            if LOG_FILE.is_none()
                || time.date_naive() != LOG_FILE.as_ref().unwrap().time.date_naive()
            {
                LOG_FILE = Some(Self::default());
            }
            LOG_FILE.as_mut().unwrap()
        };
        if let Err(err) = log.file.write(s) {
            eprintln!("日志写入文件失败 -> {err}")
        }
    }
}
impl Default for LogFile {
    fn default() -> Self {
        let time = Local::now();
        let path = time.format(LOG_NAME).to_string();
        let path = Path::new(&path);
        if let Some(p) = path.parent() {
            fs::create_dir_all(p).expect("创建日志文件父级目录失败")
        }

        Self {
            file: File::options()
                .create(true)
                .append(true)
                .write(true)
                .open(path)
                .expect("日志文件创建失败"),
            time,
        }
    }
}

// 日志中间件
pub async fn logger<B>(
    ConnectInfo(ip): ConnectInfo<SocketAddr>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // 请求方式
    let method = req.method().to_string();
    // 请求路径
    let mut path = req.uri().path().to_string();
    // 开始时间
    let time = Local::now();

    // 执行结果
    let res = next.run(req).await;
    // 状态码
    let status = res.status().as_u16();

    // 是否重定向
    if let Some(p) = res.headers().get(LOCATION) {
        path = format!("{path} -> {}", p.to_str().unwrap())
    }

    LogFile::write(
        &time,
        format!(
            "{} {} |{}| {:>6} | {:>15} | {:<5} {}\n",
            "[AXUM]",
            time.format("%Y-%m-%d %H:%M:%S"),
            status,
            format!("{}ms", (Local::now() - time).num_milliseconds()),
            ip.ip(),
            method,
            path
        )
        .as_bytes(),
    );

    let status = match status / 100 {
        2 => format!(" {status} ").on_green(),
        3 => format!(" {status} ").on_blue(),
        4 | 5 => format!(" {status} ").on_red(),
        _ => format!(" {status} ").on_yellow(),
    };

    println!(
        "{} {} |{}| {:>6} | {:>15} | {:<5} {}",
        "[AXUM]".bold().yellow(),
        time.format("%Y-%m-%d %H:%M:%S"),
        status,
        format!("{}ms", (Local::now() - time).num_milliseconds()),
        ip.ip(),
        method,
        path
    );
    Ok(res)
}
