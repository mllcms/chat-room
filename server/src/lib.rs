use axum::extract::State;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::Response,
};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::time::sleep;

pub async fn websocket(ws: WebSocketUpgrade, State(state): State<UsersState>) -> Response {
    ws.on_upgrade(move |ws| handle_socket(ws, state))
}

async fn handle_socket(socket: WebSocket, state: UsersState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = unbounded_channel::<Message>();
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
                let _ = sender
                    .send(Data::error("数据格式不正确").to_message())
                    .await;
                return;
            }
        };
        match (&data.r#type, data.target.clone()) {
            (MsgType::Login, Some(mut user)) => {
                user.channel = Some(tx.clone()); // 设置信道
                state.insert(user.id.clone(), user.clone()); // 缓存用户信息
                data.msg = format!("用户 {} 加入聊天室", user.name.clone());
                state.broadcast(data); // 群发消息
                user
            }
            _ => {
                let _ = sender.send(Data::error("登录参数有误").to_message()).await;
                return;
            }
        }
    };

    // 心跳包
    tokio::spawn(async move {
        let msg = Message::Ping("hi".as_bytes().to_vec());
        while tx.send(msg.clone()).is_ok() {
            sleep(Duration::from_secs(20)).await
        }
    });

    tokio::spawn(async move {
        let ch = user.channel.clone().unwrap();
        while let Some(result) = receiver.next().await.and_then(Result::ok) {
            match result {
                Message::Text(text) => {
                    let mut msgs = vec![];
                    if let Ok(mut data) = Data::from_str(&text) {
                        match (&data.r#type, data.target.take()) {
                            (MsgType::Public, _) => {
                                data.target = Some(user.clone());
                                state.broadcast(data);
                            }
                            (MsgType::Private, Some(u)) => match state.get(&u.id) {
                                Some(t) => {
                                    data.target = Some(user.clone());
                                    t.channel.as_ref().unwrap().send(data.to_message()).unwrap();
                                }
                                _ => msgs.push(Data::error("对方不在线").to_message()),
                            },
                            _ => msgs.push(Data::error("消息类型有误").to_message()),
                        }
                    } else {
                        msgs.push(Data::error("消息格式有误").to_message())
                    }

                    for msg in msgs {
                        if ch.send(msg).is_err() {
                            break;
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        state.remove(&user.id);
        let name = user.name.clone();
        let data = Data::new(MsgType::Logout, user, format!("用户 {name} 退出聊天室"));
        state.broadcast(data);
    });

    tokio::spawn(async move {
        while let Some(item) = rx.recv().await {
            if (sender.send(item).await).is_err() {
                break;
            }
        }
    });
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct User {
    id: String,
    name: String,
    #[serde(skip)]
    channel: Option<UnboundedSender<Message>>,
}

/// 用户状态
#[derive(Debug, Clone, Default)]
pub struct UsersState(pub Arc<DashMap<String, User>>);
impl Deref for UsersState {
    type Target = DashMap<String, User>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UsersState {
    // 消息广播 给连接池所有用户发送效消息
    pub fn broadcast(&self, mut data: Data) {
        match &data.r#type {
            MsgType::Login | MsgType::Logout => {
                let mut list: Vec<User> = self.iter().map(|m| m.clone()).collect();
                list.sort_by(|u1, u2| u1.name.cmp(&u2.name));
                data.list = Some(list);
            }
            _ => {}
        };

        let data = data.to_message();
        // 发送信息失败移出在线队列
        self.retain(|_, v| v.channel.as_ref().unwrap().send(data.clone()).is_ok());
    }
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgType {
    Login,
    Logout,
    System,
    Error,
    Public,
    Private,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Data {
    /// 消息类型
    r#type: MsgType,
    /// 消息
    msg: String,
    /// public 自动设置为发信者 private 进来目标对象出去发信者
    target: Option<User>,
    /// 用户列表
    list: Option<Vec<User>>,
}

impl Data {
    fn new(t: MsgType, target: User, msg: String) -> Self {
        Self {
            msg,
            r#type: t,
            target: Some(target),
            list: None,
        }
    }
    fn error(msg: &str) -> Self {
        Self {
            msg: msg.into(),
            target: None,
            r#type: MsgType::Error,
            list: None,
        }
    }

    fn from_str(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }

    fn to_message(&self) -> Message {
        Message::from(serde_json::to_string(self).unwrap())
    }
}