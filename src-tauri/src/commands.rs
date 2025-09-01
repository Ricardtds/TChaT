use std::fs;
use std::sync::Arc;

use crate::models::api::{OuterChatMessage, SubscriptionData};
use crate::models::app;
use crate::utils::messages::{self, cleanup_messages_by_chatroom, delete_chatroom};
use base64::{engine::general_purpose, Engine};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use tauri::{AppHandle, Manager, State, Window};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
pub type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub struct WsState(pub Arc<Mutex<Option<WsWriter>>>);
const WS_KICK_URL: &str = "wss://ws-us2.pusher.com/app/32cbd69e4b950bf97679?protocol=7";
#[tauri::command]
pub async fn subscribe_to_channel(
    chatroom_id: i32,
    chatroom_name: String,
    app: AppHandle,
    window: Window,
    ws_state: State<'_, WsState>,
    db_pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let mut writer_lock = ws_state.0.lock().await;
    let db_pool = db_pool.inner().clone();
    _ = messages::insert_chatroom(&db_pool, chatroom_name, chatroom_id)
        .await
        .map_err(|e| {
            eprintln!("❌ Erro no insert: {:?}", e);
            e
        });
    _ = cleanup_messages_by_chatroom(&db_pool, chatroom_id)
        .await
        .map_err(|e| {
            eprintln!("❌ Erro na limpeza de chat: {:?}", e);
            e
        });

    if writer_lock.is_none() {
        println!("[RUST]: Nenhuma conexão ativa. Estabelecendo conexão WebSocket...");
        let (ws_stream, _) = connect_async(WS_KICK_URL)
            .await
            .map_err(|e| e.to_string())?;
        println!("[RUST]: Conexão WebSocket estabelecida.");

        let (write, read) = ws_stream.split();

        // Salva a parte "escritora" no estado global.
        *writer_lock = Some(write);
        let task_db_pool = db_pool.clone();
        let app_clone = app.clone();
        tokio::spawn(async move {
            println!("[RUST]: Tarefa de WebSocket iniciada.");
            let mut read_stream = read;
            while let Some(message_result) = read_stream.next().await {
                match message_result {
                    Ok(msg) => {
                        if messages::handle_message(&task_db_pool, msg, &window)
                            .await
                            .is_err()
                        {
                            println!("[RUST]: Conexão fechada. Encerrando tarefa de leitura.");
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
            println!("[RUST]: Tarefa de WebSocket finalizada.");
        });
    }

    if let Some(writer) = writer_lock.as_mut() {
        let subscribe_message = OuterChatMessage {
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

        // let _ = cleanup_messages(&db_pool, &chatroom_id, 2000).await;
    } else {
        return Err("❌ Não foi possível obter o escritor da conexão WebSocket.".into());
    }

    Ok(())
}

#[tauri::command]
pub async fn unsubscribe_from_channel(
    chatroom_id: i64,
    ws_state: State<'_, WsState>,
    db_pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let db_pool = db_pool.inner().clone();
    let mut writer_lock = ws_state.0.lock().await;
    if let Some(writer) = writer_lock.as_mut() {
        let unsubscribe_message = OuterChatMessage {
            event: "pusher:unsubscribe".to_string(),
            data: SubscriptionData {
                channel: format!("chatrooms.{}.v2", chatroom_id),
            },
        };
        let json_message =
            serde_json::to_string(&unsubscribe_message).map_err(|e| e.to_string())?;
        writer
            .send(Message::Text(json_message.into()))
            .await
            .map_err(|e| e.to_string())?;
        println!("[RUST]: Inscrição cancelada para o canal {}", &chatroom_id);
        delete_chatroom(&db_pool, chatroom_id)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("❌ Não há conexão ativa para cancelar a inscrição.".into())
    }
}

#[tauri::command]
pub async fn get_chat_history(
    chatroom_id: i32,
    before_date: String,
    rows_qtd: i32,
    db_pool: State<'_, SqlitePool>,
) -> Result<Vec<app::ChatMessage>, String> {
    // println!(
    //     "[RUST]: Buscando {} mensagens para o chat {} antes de {}",
    //     &rows_qtd, &chatroom_id, &before_date
    // );
    let pool = db_pool.inner();

    let rows = sqlx::query!(
        r#"
            SELECT
                m.id,
                m.chatroom_id,
                m.content,
                m.type,
                m.created_at,
                s.id as sender_id,
                s.username,
                s.slug,
                s.color
            FROM messages m
            JOIN senders s ON m.sender_id = s.id
            WHERE m.chatroom_id = $1 AND m.created_at < $2
            ORDER BY m.created_at DESC
            LIMIT $3
    "#,
        chatroom_id,
        before_date,
        rows_qtd
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let sender_ids: Vec<String> = rows.iter().map(|m| m.sender_id.to_string()).collect();

    let joined = sender_ids.join(",");

    let badges = sqlx::query!(
        r#"
            SELECT
                sb.sender_id,
                b.type as badge_type,
                b.text as badge_text,
                b.count as badge_count
            FROM sender_badges sb
            JOIN badges b ON sb.badge_id = b.id
            WHERE sb.sender_id IN ($1)
            "#,
        joined
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut badges_map: HashMap<i64, Vec<app::Badge>> = HashMap::new();

    for row in badges {
        badges_map
            .entry(row.sender_id)
            .or_default()
            .push(app::Badge {
                badge_type: row.badge_type,
                text: row.badge_text,
                count: row.badge_count,
            });
    }

    let messages: Vec<app::ChatMessage> = rows
        .into_iter()
        .map(|row| app::ChatMessage {
            id: row.id.to_string(),
            chatroom_id: row.chatroom_id,
            content: row.content,
            message_type: row.r#type,
            created_at: row.created_at,
            sender: app::Sender {
                id: row.sender_id,
                username: row.username,
                slug: row.slug,
                identity: app::Identity {
                    color: row.color,
                    badges: badges_map.get(&row.sender_id).cloned(),
                },
            },
        })
        .collect();

    Ok(messages)
}

#[tauri::command]
pub async fn cache_emote_locally(
    app: tauri::AppHandle,
    emote_id: String,
    image_data: Vec<u8>,
    imagem_mime: String,
) -> Result<(), String> {
    println!("Cacheando emote: {}", emote_id);
    let cache_dir = app
        .path()
        .app_cache_dir()
        .expect("❌ Não foi possível obter o diretório de cache do app.")
        .join("emote_cache");

    fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;

    let data_file_path = cache_dir.join(format!("{}.data", emote_id));
    fs::write(data_file_path, image_data).map_err(|e| e.to_string())?;

    let meta_file_path = cache_dir.join(format!("{}.mimetype", emote_id));
    fs::write(meta_file_path, imagem_mime).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn read_emote_from_cache(
    app: tauri::AppHandle,
    emote_id: String,
) -> Result<Option<String>, String> {
    println!("Lendo emote do cache: {}", emote_id);
    let cache_dir = app
        .path()
        .app_cache_dir()
        .expect("❌ Não foi possível obter o diretório de cache do app.")
        .join("emote_cache");

    let data_file_path = cache_dir.join(format!("{}.data", emote_id));
    let meta_file_path = cache_dir.join(format!("{}.mimetype", emote_id));

    if data_file_path.exists() && meta_file_path.exists() {
        let data = fs::read(data_file_path).map_err(|e| e.to_string())?;
        let mime_type = fs::read_to_string(meta_file_path).map_err(|e| e.to_string())?;

        let base64_encoded = general_purpose::STANDARD.encode(&data);

        let data_url = format!("data:{};base64,{}", mime_type.trim(), base64_encoded);

        Ok(Some(data_url))
    } else {
        Ok(None)
    }
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FetchedEmote {
    mime_type: String,
    data: Vec<u8>,
}

#[tauri::command]
pub async fn fetch_emote_from_web(emote_id: String) -> Result<FetchedEmote, String> {
    println!("Buscando emote da web: {}", emote_id);
    let url = format!("https://files.kick.com/emotes/{}/fullsize", emote_id);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?;


    let mime_type = response
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap_or("application/octet-stream"))
        .unwrap_or("application/octet-stream")
        .to_string();


    let data = response
        .bytes()
        .await
        .map_err(|e| e.to_string())?
        .to_vec();
    println!("Enviando para o frontend: mime_type={}, tamanho_dos_dados={}", &mime_type, data.len());
    Ok(FetchedEmote { mime_type, data })
}

#[tauri::command]
pub async fn clear_emote_cache(app: tauri::AppHandle) -> Result<(), String> {
    let cache_dir = app
        .path()
        .app_cache_dir()
        .expect("❌ Não foi possível obter o diretório de cache do app.")
        .join("emote_cache");

    if cache_dir.exists() {
        fs::remove_dir_all(cache_dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}