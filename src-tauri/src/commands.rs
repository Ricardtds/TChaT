use std::sync::Arc;

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use sqlx::{sqlite::SqlitePool, Pool, query, query_as};
use tauri::{AppHandle, Emitter, Manager, State, Window};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use std::env;

use crate::models::api::{ApiChatMessage, ApiOuterMessage, ApiSender, ApiIdentity};
use crate::models::app::{AppChatMessage, AppSender, AppIdentity};

#[derive(serde::Serialize)]
struct SubscriptionData {
    channel: String,
}
#[derive(serde::Serialize)]
struct PusherEvent {
    event: String,
    data: SubscriptionData,
}

pub type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub struct WsState(pub Arc<Mutex<Option<WsWriter>>>);
const PI: f64 = 3.14159; // Declaração de uma constante PI do tipo f64
const WS_KICK_URL: &str = "wss://ws-us2.pusher.com/app/32cbd69e4b950bf97679?protocol=7";
const DATABASE_URL: &str = "sqlite:./database.db";
#[tauri::command]
pub async fn subscribe_to_channel(
    chatroom_id: String,
    app: AppHandle,
    window: Window,
    ws_state: State<'_, WsState>,
    db_pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let mut writer_lock = ws_state.0.lock().await;
    let db_pool = db_pool.inner().clone();

    if writer_lock.is_none() {
        println!("[RUST]: Nenhuma conexão ativa. Estabelecendo conexão WebSocket...");
        let (ws_stream, _) = connect_async(WS_KICK_URL).await.map_err(|e| e.to_string())?;
        println!("[RUST]: Conexão WebSocket estabelecida.");

        let (write, read) = ws_stream.split();

        // Salva a parte "escritora" no estado global.
        *writer_lock = Some(write);

        let task_db_pool = db_pool.clone();
        let app_clone = app.clone();
        tokio::spawn(async move {
            println!("[RUST]: Tarefa leitora de WebSocket iniciada.");
            let mut read_stream = read;
            while let Some(message_result) = read_stream.next().await {
                match message_result {
                    Ok(msg) => {
                        if handle_message(&task_db_pool, msg, &window).await.is_err() {
                            println!("[RUST]: Conexão fechada. Encerrando tarefa leitora.");
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Erro ao receber mensagem WebSocket: {}", e);
                        break;
                    }
                }
            }
            let ws_state = app_clone.state::<WsState>();
            *ws_state.0.lock().await = None;
            println!("[RUST]: Tarefa leitora de WebSocket finalizada.");
        });
    }

    if let Some(writer) = writer_lock.as_mut() {
        let subscribe_message = PusherEvent {
            event: "pusher:subscribe".to_string(),
            data: SubscriptionData {
                channel: format!("chatrooms.{}.v2", chatroom_id),
            },
        };
        let json_message = serde_json::to_string(&subscribe_message).map_err(|e| e.to_string())?;
        writer
            .send(Message::Text(json_message.into()))
            .await
            .map_err(|e| e.to_string())?;
        println!("[RUST]: Inscrito no canal {}", &chatroom_id);

        let _ = cleanup_messages(&db_pool, &chatroom_id, 2000).await;
    } else {
        return Err("Não foi possível obter o escritor da conexão WebSocket.".into());
    }

    Ok(())
}

#[tauri::command]
pub async fn unsubscribe_from_channel(
    chatroom_id: String,
    ws_state: State<'_, WsState>,
    db_pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let mut writer_lock = ws_state.0.lock().await;
    if let Some(writer) = writer_lock.as_mut() {
        let unsubscribe_message = PusherEvent {
            event: "pusher:unsubscribe".to_string(),
            data: SubscriptionData {
                channel: format!("chatrooms.{}.v2", chatroom_id),
            },
        };
        let json_message = serde_json::to_string(&unsubscribe_message).map_err(|e| e.to_string())?;
        writer
            .send(Message::Text(json_message.into()))
            .await
            .map_err(|e| e.to_string())?;
        println!("[RUST]: Inscrição cancelada para o canal {}", &chatroom_id);
        let _ = cleanup_messages(&db_pool, &chatroom_id, 0).await;
        Ok(())
    } else {
        Err("Não há conexão ativa para cancelar a inscrição.".into())
    }
}


async fn handle_message(pool: &Pool<sqlx::Sqlite>, msg: Message, window: &Window) -> Result<(), ()> {
    if let Message::Text(text) = msg {
        if let Ok(outer_message) = serde_json::from_str::<ApiOuterMessage>(&text) {
            if outer_message.event == "App\\Events\\ChatMessageEvent" {
                if let Ok(api_message) = serde_json::from_str::<ApiChatMessage>(&outer_message.data) {
                    let _ = insert_message(pool, &api_message).await;
                    // println!("💬 Recebido de [{}]: {}", api_message.sender.username, api_message.content);
                    let app_message: AppChatMessage = api_message.into();
                    window.emit("new-chat-message", app_message).unwrap();
                }
            }
        }
    } else if let Message::Close(_) = msg {
        return Err(());
    }
    Ok(())
}

async fn insert_message(pool: &Pool<sqlx::Sqlite>, api_message: &ApiChatMessage) -> Result<(), sqlx::Error> {
    sqlx::query!("INSERT OR IGNORE INTO senders(id, username, slug, color) VALUES (?, ?, ?, ?)",
            api_message.sender.id as i64,
            &api_message.sender.username,
            &api_message.sender.slug,
            &api_message.sender.identity.color)
        .bind(api_message.sender.id as i64)
        .bind(&api_message.sender.username)
        .bind(&api_message.sender.slug)
        .bind(&api_message.sender.identity.color)
        .execute(pool).await?;
    
    sqlx::query("INSERT OR IGNORE INTO messages(id, chatroom_id, content, created_at, sender_id) VALUES (?, ?, ?, ?, ?)")
        .bind(&api_message.id)
        .bind(api_message.chatroom_id as i64)
        .bind(&api_message.content)
        .bind(&api_message.created_at)
        .bind(api_message.sender.id as i64)
        .execute(pool).await?;
    
    Ok(())
}

pub async fn cleanup_messages(pool: &Pool<sqlx::Sqlite>, chatroom_id: &String, keep_rows: i64) -> Result<(), sqlx::Error> {
    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM messages WHERE chatroom_id = ?", chatroom_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    if total > keep_rows {
        let offset = total - keep_rows;
        let result = sqlx::query!(
            "DELETE FROM messages WHERE id IN (SELECT id FROM messages WHERE chatroom_id = ? ORDER BY created_at ASC LIMIT ?)",
            chatroom_id,
            offset
        )
        .execute(pool)
        .await;

        if let Ok(res) = result {
             println!("Limpeza de mensagens antigas para o chat {}: {} linhas deletadas.", chatroom_id, res.rows_affected());
        }
    }
    Ok(())
}
#[derive(sqlx::FromRow, Debug)]
struct MessageRow {
    id: String,
    chatroom_id: i64,
    content: String,
    created_at: String,
    sender_id: i64,
    username: String,
    slug: String,
    color: String,
}

#[tauri::command]
pub async fn get_older_messages(
    chatroom_id: String,
    before_date: String,
    rows_qtd: i64,
    db_pool: State<'_, SqlitePool>,
) -> Result<Vec<AppChatMessage>, String> {
    println!("[RUST]: Buscando {} mensagens para o chat {} antes de {}",&rows_qtd, &chatroom_id, &before_date);
    let pool = db_pool.inner(); 

    let rows = sqlx::query_as!(
        AppChatMessage,
        "SELECT m.id, m.chatroom_id, m.content, m.created_at, s as sender, s.username, s.slug, s.color FROM messages m INNER JOIN senders s ON m.sender_id = s.id WHERE m.chatroom_id = (1) AND datetime(m.created_at) < datetime((2)) ORDER BY datetime(m.created_at) DESC LIMIT (3)"
    ).fetch_all(pool)
    .await?;


    let messages = rows
            .into_iter()
            .map(|row| AppChatMessage {
                id: row.id,
                chatroom_id: row.chatroom_id as u64,
                content: row.content,
                created_at: row.created_at,
                message_type: "message".into(),
                sender: AppSender {
                    id: row.sender_id as u64,
                    username: row.username,
                    slug: row.slug,
                    identity: AppIdentity {
                        color: row.color,
                        badges: Vec::new(),
                    },
                },
            }).collect();
    Ok(messages)
}
use serde::Serialize;
use sqlx::FromRow;
// Seus structs (assumindo que já estão definidos corretamente)
#[derive(Debug, Serialize, FromRow)]
pub struct ChatMessage {
    pub id: i64,
    pub chatroom_id: i64,
    pub content: Option<String>,
    pub created_at: Option<String>, // <-- alterado para Option
    pub sender_id: Option<i64>,     // <-- alterado para Option
}

#[tauri::command]
pub async fn get_chat_history(
    chatroom_id: i64,
    rows_qtd: i64,
    db_pool: State<'_, SqlitePool>,
) -> u64 {
    let pool = db_pool.inner();



    let account = sqlx::query_as!(
        ChatMessage,
        "select * from messages where id = ?",
        1i32
    )
    .fetch_one(pool)
    .await?;

    return 0 as u64;
}
