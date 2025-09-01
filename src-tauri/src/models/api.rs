#![allow(dead_code)]
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Debug;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct MessageFront {
    pub message_id: String,
    pub message_chatroom_id: i64,
    pub message_content: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub message_created_at: String,
    pub sender_id: i64,
    pub sender_username: String,
    pub sender_slug: String,
    pub sender_color: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct SenderBadge {
    pub sender_id: i64,
    pub badge_id: i64,
    pub chatroom_id: i64,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ChatMessageEnum {
    ChatMessageWithMetadata(ChatMessageWithMetadata),
    ChatMessage(ChatMessage),
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub id: String,
    pub chatroom_id: i64,
    pub content: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub created_at: String,
    pub sender: Sender,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct ChatMessageWithMetadata {
    pub id: String,
    pub chatroom_id: i64,
    pub content: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub created_at: String,
    pub sender: Sender,
    pub metadata: Metadata,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Metadata {
    pub message_ref: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Sender {
    pub id: i64,
    pub username: String,
    pub slug: String,
    pub identity: Identity,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Identity {
    pub color: String,
    pub badges: Option<Vec<Badge>>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct BadgeId {
    pub id: i64,
    #[serde(rename = "type")]
    pub badge_type: String,
    pub text: String,
}
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Badge {
    #[serde(rename = "type")]
    pub badge_type: String,
    pub text: String,
    pub count: Option<i64>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct BadgeCount {
    #[serde(rename = "type")]
    pub badge_type: String,
    pub text: String,
    pub count: i32,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Chatroom {
    pub id: i32,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ChatroomName {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct V2Channel {
    pub id: i32,
    pub slug: String,
    pub chatroom: Chatroom,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscriptionData {
    pub channel: String,
}

#[derive(Serialize, Deserialize)]
pub struct UnSubscriptionData {
    pub channel: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OuterChatMessage {
    pub event: String,
    pub data: SubscriptionData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorEvent {
    pub code: Option<i32>,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Connection {
    pub socket_id: String,
    pub activity_timeout: i32,
}

pub enum OutgoingEvent {
    Subscribe(SubscriptionData),
    Unsubscribe(UnSubscriptionData),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum IncomingEvent {
    PusherEventChannel(PusherEventChannel),
    PusherEvent(PusherEvent),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PusherEventEnum {
    ChatMessage(ChatMessageEnum),
    Connection(Connection),
    Error(ErrorEvent),
    Subscription(OuterChatMessage),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PusherEvent {
    pub event: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub data: PusherEventEnum,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PusherEventChannel {
    pub event: String,
    pub channel: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    pub data: PusherEventEnum,
}

use serde::de::DeserializeOwned;
use serde::de::Error as DeError;

#[derive(Deserialize)]
#[serde(untagged)]
enum MaybeString<T> {
    String(String),
    Obj(T),
}

pub trait Channel {
    fn get_chatmessage(&self) -> ChatMessage;
}

impl Channel for ChatMessageWithMetadata {
    fn get_chatmessage(&self) -> ChatMessage {
        ChatMessage {
            id: self.id.clone(),
            chatroom_id: self.chatroom_id.clone(),
            content: self.content.clone(),
            message_type: self.message_type.clone(),
            created_at: self.created_at.clone(),
            sender: self.sender.clone(),
        }
    }
}

impl Channel for ChatMessage {
    fn get_chatmessage(&self) -> ChatMessage {
        ChatMessage {
            id: self.id.clone(),
            chatroom_id: self.chatroom_id.clone(),
            content: self.content.clone(),
            message_type: self.message_type.clone(),
            created_at: self.created_at.clone(),
            sender: self.sender.clone(),
        }
    }
}

fn deserialize_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: DeserializeOwned + Debug,
    D: Deserializer<'de>,
{
    match MaybeString::<T>::deserialize(deserializer)? {
        MaybeString::String(s) => serde_json::from_str(&s).map_err(DeError::custom),
        MaybeString::Obj(obj) => Ok(obj),
    }
}
