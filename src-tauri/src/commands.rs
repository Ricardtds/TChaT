use std::sync::Arc;

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use sqlx::{sqlite::SqlitePool, Pool};
use tauri::{AppHandle, Emitter, Manager, State, Window};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

use crate::models::api::{ApiChatMessage, ApiOuterMessage};
use crate::models::app::AppChatMessage;

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
        let ws_url = "wss://ws-us2.pusher.com/app/32cbd69e4b950bf97679?protocol=7".to_string();
        let (ws_stream, _) = connect_async(&ws_url).await.map_err(|e| e.to_string())?;
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
                    println!("💬 Recebido de [{}]: {}", api_message.sender.username, api_message.content);
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
    sqlx::query("INSERT OR IGNORE INTO senders(id, username, slug, color) VALUES (?, ?, ?, ?)")
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
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE chatroom_id = ?")
        .bind(chatroom_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    if total > keep_rows {
        let offset = total - keep_rows;
        let result = sqlx::query(
            "DELETE FROM messages WHERE id IN (SELECT id FROM messages WHERE chatroom_id = ? ORDER BY created_at ASC LIMIT ?)"
        )
        .bind(chatroom_id)
        .bind(offset)
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
pub async fn get_chat_history(
    chatroom_id: String,
    db_pool: State<'_, SqlitePool>,
) -> Result<Vec<AppChatMessage>, String> {
    println!("[RUST]: Buscando histórico de mensagens para o chat {}", &chatroom_id);

    let pool = db_pool.inner(); 
    let rows: Result<Vec<MessageRow>, sqlx::Error> = sqlx::query_as("SELECT 
       messages.id, messages.chatroom_id, messages.content, messages.created_at,
       senders.id AS sender_id, senders.username, senders.slug, senders.color
        FROM messages
        INNER JOIN senders ON messages.sender_id = senders.id
        WHERE messages.chatroom_id = ?
        ORDER BY datetime(messages.created_at) ASC
        LIMIT 500")
        .bind(chatroom_id)
        .fetch_all(pool)
        .await;

    let messages = match rows {
        Ok(rows) => {
            rows
            .into_iter()
            .map(|row| AppChatMessage {
                id: row.id,
                chatroom_id: row.chatroom_id as u64,
                content: row.content,
                created_at: row.created_at,
                message_type: "message".into(),
                sender: crate::models::app::AppSender {
                    id: row.sender_id as u64,
                    username: row.username,
                    slug: row.slug,
                    identity: crate::models::app::AppIdentity {
                        color: row.color,
                        badges: Vec::new(),
                    },
                },
            }).collect()
        }
        Err(e) => {
            eprintln!("Erro ao tentar pegar chat_history {:?}",e);
            std::process::exit(0);
        }
    };
   
    Ok(messages)
}