use crate::middleware::logger::logger;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    middleware::from_fn,
    response::Response,
    routing::get,
    Extension, Router,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    sync::{
        mpsc::{channel, Sender},
        Mutex,
    },
    task::JoinHandle,
    time::sleep,
};

use tower_http::services::ServeDir;

// 用户状态
type UsersState = Arc<Mutex<HashMap<String, User>>>;

// ws通信地址
#[derive(Clone)]
struct WsAddr(String);

pub fn run(addr: &'static str) -> JoinHandle<()> {
    let users: UsersState = Arc::new(Mutex::new(HashMap::new()));

    tokio::spawn(async move {
        println!("Server running at http://{addr}");

        let app = Router::new()
            .nest_service(
                "/",
                ServeDir::new("dist").append_index_html_on_directories(true),
            )
            .route("/ws", get(webscoket))
            .layer(Extension(users))
            .layer(from_fn(logger));

        axum::Server::bind(&addr.parse().unwrap())
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();

        println!("{} Server stop {}", "-".repeat(10), "-".repeat(10));
    })
}

async fn webscoket(ws: WebSocketUpgrade, Extension(users): Extension<UsersState>) -> Response {
    ws.on_upgrade(move |ws| handle_socket(ws, users))
}

async fn handle_socket(socket: WebSocket, users: UsersState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = channel::<Message>(100);
    // 登录请求
    let user = {
        let msg = match receiver.next().await {
            Some(Ok(Message::Text(res))) => res,
            _ => {
                let _ = sender.send(Data::error("登录失败").to_message()).await;
                return;
            }
        };
        let mut data = match Data::from_str(&msg.to_string()) {
            Ok(res) => res,
            _ => {
                let _ = sender.send(Data::error("登录失败").to_message()).await;
                return;
            }
        };
        match (data.r#type.as_ref(), data.target.clone()) {
            ("login", Some(mut user)) => {
                user.channel = Some(tx.clone()); // 设置信道
                users.lock().await.insert(user.id.clone(), user.clone()); // 缓存用户信息
                data.msg = format!("用户 {} 加入聊天室", user.name.clone());
                send_users(users.clone(), data).await; // 群发消息
                user
            }
            _ => {
                let _ = sender.send(Data::error("登录失败").to_message()).await;
                return;
            }
        }
    };

    // 心跳包
    tokio::spawn(async move {
        let tx = tx.clone();
        let msg = Message::Ping("hi".as_bytes().to_vec());
        while let Ok(_) = tx.send(msg.clone()).await {
            sleep(Duration::from_secs(20)).await
        }
    });

    tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            let result = match result {
                Ok(res) => res,
                Err(_) => {
                    continue;
                }
            };
            let users = users.clone();
            match result {
                Message::Text(text) => {
                    let mut msgs = vec![];
                    if let Ok(mut data) = Data::from_str(&text) {
                        match (data.r#type.as_str(), data.target.take()) {
                            ("public", _) => {
                                data.target = Some(user.clone());
                                send_users(users, data).await;
                            }
                            ("private", Some(u)) => match users.lock().await.get(&u.id) {
                                Some(t) => {
                                    data.target = Some(user.clone());
                                    t.channel
                                        .clone()
                                        .unwrap()
                                        .send(data.to_message())
                                        .await
                                        .unwrap();
                                }
                                _ => msgs.push(Data::error("对方不在线").to_message()),
                            },
                            _ => msgs.push(Data::error("消息类型有误").to_message()),
                        }
                    } else {
                        msgs.push(Data::error("消息格式有误").to_message())
                    }

                    let ch = user.channel.clone().unwrap();
                    for msg in msgs {
                        if let Err(_) = ch.send(msg).await {
                            break;
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        users.lock().await.remove(&user.id);
        let name = user.name.clone();
        let data = Data::new("logout", user, format!("用户 {name} 退出聊天室"));
        send_users(users, data).await;
    });

    tokio::spawn(async move {
        while let Some(item) = rx.recv().await {
            if let Err(_) = sender.send(item).await {
                break;
            }
        }
    });
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
struct User {
    id: String,
    name: String,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    channel: Option<Sender<Message>>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
struct Data {
    r#type: String,       // 消息类型
    msg: String,          // 消息
    target: Option<User>, // public 自动设置为发信者 private 进来目标对象出去发信者
    list: Option<Vec<User>>,
}

impl Data {
    fn new(t: &str, target: User, msg: String) -> Self {
        Self {
            msg,
            r#type: t.into(),
            target: Some(target),
            list: None,
        }
    }
    fn error(msg: &str) -> Self {
        Self {
            msg: msg.into(),
            r#type: "error".into(),
            ..Self::default()
        }
    }

    fn from_str(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }

    fn to_message(&self) -> Message {
        Message::from(serde_json::to_string(self).unwrap())
    }
}

// 消息广播 给连接池所有用户发送效消息
async fn send_users(users: Arc<Mutex<HashMap<String, User>>>, mut data: Data) {
    let mut users = users.lock().await;
    match data.r#type.as_ref() {
        "login" | "logout" => {
            let mut list: Vec<User> = users.values().map(|m| m.clone()).collect();
            list.sort_by(|u1, u2| u1.name.cmp(&u2.name));
            data.list = Some(list);
        }
        _ => {}
    };

    let mut disabled = vec![];
    let data = data.to_message();
    for (k, v) in users.iter_mut() {
        if v.channel
            .as_mut()
            .unwrap()
            .send(data.clone())
            .await
            .is_err()
        {
            disabled.push(k.clone())
        }
    }

    // 删除发送失败的用户连接
    for k in disabled {
        users.remove(&k);
    }
}
