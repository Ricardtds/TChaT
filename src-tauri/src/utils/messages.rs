use std::collections::{HashMap, HashSet};

use serde_json::Error;
use sqlx::{Pool, Sqlite, SqlitePool};
use tauri::{Emitter, Window};
use tokio_tungstenite::tungstenite::Message;

use crate::models::{api, app, db};

pub async fn handle_messages(
    pool: &Pool<sqlx::Sqlite>,
    msgs: Vec<Message>,
    window: &Window,
) -> Result<(), Error> {
    let mut chat_messages: Vec<api::ChatMessageData> = Vec::new();
    let mut app_messages: Vec<app::Message> = Vec::new();

    for msg in msgs {
        if let Message::Text(text) = msg {
            let event: api::ChatMessageEvent = serde_json::from_str(&text)?;
            match event.event.as_str() {
                "App\\Events\\ChatMessageEvent" => {
                    let data: api::ChatMessageData = serde_json::from_str(&event.data)?;
                    chat_messages.push(data.clone());
                    let app_message = app::Message {
                        id: data.id,
                        chatroom_id: data.chatroom_id,
                        content: data.content,
                        created_at: data.created_at,
                        r#type: data.r#type,
                        sender: app::Sender {
                            id: data.sender.id,
                            username: data.sender.username,
                            identity: app::Identity {
                                color: data.sender.identity.color,
                                badges: data
                                    .sender
                                    .identity
                                    .badges
                                    .iter()
                                    .cloned()
                                    .map(|opt| opt.map(|b| b.into_app()))
                                    .collect(),
                            },
                        },
                    };
                    // println!("✅ Mensagem desserializada {:?}",&app_message);
                    app_messages.push(app_message);
                }
                _ => {
                    eprintln!("❌ Evento desconhecido: {}", event.event)
                }
            }
        }
    }

    emit_grouped_messages(window, app_messages);
    // window.emit("new-chat-messages", app_messages).unwrap();

    insert_messages(pool, chat_messages)
        .await
        .map_err(|e| eprintln!("❌ Erro ao inserir mensagens: {:?}", e));
    Ok(())
}

fn emit_grouped_messages(window: &Window, app_messages: Vec<app::Message>) {
    let mut grouped: HashMap<i32, Vec<app::Message>> = HashMap::new();

    for msg in app_messages {
        grouped.entry(msg.chatroom_id).or_default().push(msg);
    }

    for (chatroom_id, messages) in grouped {
        let event_name = format!("new-chat-messages-{}", chatroom_id);
        println!("{}", event_name);
        window.emit(&event_name, messages).unwrap();
    }
}

pub async fn insert_chatroom(
    pool: &Pool<sqlx::Sqlite>,
    // TODO Fazer
    chatroom_name: String,
    chatroom_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT OR IGNORE INTO 
                chatrooms(id, name)
            VALUES
                ($1, $2)
        "#,
        chatroom_id,
        chatroom_name
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_messages(
    pool: &Pool<Sqlite>,
    api_messages: Vec<api::ChatMessageData>,
) -> Result<(), sqlx::Error> {
    let mut unique_senders = HashSet::new();
    let mut messages_to_insert = Vec::new();
    let mut sender_badge_relations = Vec::new();

    for api_message in api_messages {
        let message = api_message;

        let message_item = db::Message {
            id: message.id,
            chatroom_id: message.chatroom_id,
            sender_id: message.sender.id,
            content: message.content,
            r#type: message.r#type,
            created_at: message.created_at,
            metadata: message.metadata.and_then(|data| Some(data.message_ref)),
        };
        messages_to_insert.push(message_item);

        let sender = api::Sender {
            id: message.sender.id,
            username: message.sender.username,
            slug: message.sender.slug,
            identity: message.sender.identity.clone(),
        };
        unique_senders.insert(sender);

        for badge_option in message.sender.identity.badges {
            if let Some(badge) = badge_option {
                let badge_id = match badge.r#type.as_str() {
                    "og" => 1,
                    "bot" => 2,
                    "vip" => 3,
                    "founder" => 4,
                    "moderator" => 5,
                    "sub_gifter" => 6,
                    "subscriber" => 7,
                    "broadcaster" => 8,
                    "verified" => 9,
                    _ => 0,
                };

                sender_badge_relations.push(db::SenderBadge {
                    sender_id: message.sender.id,
                    chatroom_id: message.chatroom_id,
                    badge_id: badge_id,
                    count: badge.count,
                });
            }
        }
    }

    let mut tx = pool.begin().await?;

    for sender in &unique_senders {
        sqlx::query!(
            "INSERT OR IGNORE INTO senders(id, username, slug, color) VALUES ($1, $2, $3, $4)",
            sender.id,
            sender.username,
            sender.slug,
            sender.identity.color
        )
        .execute(&mut *tx)
        .await?;
    }

    for message in &messages_to_insert {
        sqlx::query!(
            "INSERT OR IGNORE INTO messages(id, chatroom_id, sender_id, content, type, created_at) VALUES ($1, $2, $3, $4, $5, $6)",
            message.id,
            message.chatroom_id,
            message.sender_id,
            message.content,
            message.r#type,
            message.created_at
        )
        .execute(&mut *tx)
        .await?;
    }

    for relation in &sender_badge_relations {
        sqlx::query!(
            "INSERT OR IGNORE INTO sender_badges (sender_id, badge_id, chatroom_id, count) VALUES ($1, $2, $3, $4)",
            relation.sender_id,
            relation.badge_id,
            relation.chatroom_id,
            relation.count
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn delete_chatroom(
    pool: &Pool<sqlx::Sqlite>,
    chatroom_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            DELETE FROM chatrooms WHERE id = $1;
        "#,
        chatroom_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn cleanup_messages_by_chatroom(
    pool: &SqlitePool,
    chatroom_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            DELETE FROM messages
            WHERE id IN (
                SELECT id
                FROM messages
                WHERE chatroom_id = $1
                ORDER BY created_at ASC
                LIMIT (
                    SELECT CASE
                        WHEN COUNT(*) > 20000 THEN 5000
                        ELSE 0
                    END
                    FROM messages
                    WHERE chatroom_id = $1
                )
            )
            "#,
        chatroom_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}
