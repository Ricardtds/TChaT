use sqlx::{Pool, SqlitePool};
use tauri::{Emitter, Window};
use tokio_tungstenite::tungstenite::Message;

use crate::models::{
    api::{self, Channel},
    app,
};

pub async fn handle_message(
    pool: &Pool<sqlx::Sqlite>,
    msg: Message,
    window: &Window,
) -> Result<(), ()> {
    if let Message::Text(text) = msg {
        let outer_message = serde_json::from_str::<api::IncomingEvent>(&text);
        match outer_message {
            Ok(outer_message) => match outer_message {
                api::IncomingEvent::PusherEvent(data) => match data.data {
                    api::PusherEventEnum::ChatMessage(msg) => {}
                    api::PusherEventEnum::Connection(msg) => {}
                    api::PusherEventEnum::Error(msg) => {}
                    api::PusherEventEnum::Subscription(msg) => {}
                },
                api::IncomingEvent::PusherEventChannel(data) => match data.data {
                    api::PusherEventEnum::ChatMessage(data) => match data {
                        api::ChatMessageEnum::ChatMessage(msg) => {
                            // println!("💬 Recebido de [{}]: {}", msg.sender.username, msg.content);
                            let _ = insert_message(pool, &msg).await;
                            let app_message: app::ChatMessage = msg.into();
                            window.emit("new-chat-message", app_message).unwrap();
                        }
                        api::ChatMessageEnum::ChatMessageWithMetadata(msg) => {
                            // println!("💬 Recebido de [{}]: {}", msg.sender.username, msg.content);
                            let _ = insert_message(pool, &msg).await;
                            let app_message: app::ChatMessageWithMetadata = msg.into();
                            window.emit("new-chat-message", app_message).unwrap();
                        }
                    },
                    api::PusherEventEnum::Connection(msg) => {}
                    api::PusherEventEnum::Error(msg) => {}
                    api::PusherEventEnum::Subscription(msg) => {}
                },
            },
            Err(e) => {
                eprintln!(
                    "❌ Erro ao desserializar mensagem: {} O json é {} \n",
                    e, &text
                );
                // return Err(());
            }
        }
    } else if let Message::Close(_) = msg {
        return Err(());
    }
    Ok(())
}

pub async fn insert_chatroom(
    pool: &Pool<sqlx::Sqlite>,
    chatroom_name: String,
    chatroom_id: i32,
) -> Result<(), sqlx::Error> {
    println!("task executada");
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
    println!("task finalizada");
    Ok(())
}

async fn insert_message(
    pool: &Pool<sqlx::Sqlite>,
    api_message: &impl Channel,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let message = api_message.get_chatmessage();

    sqlx::query!(
        r#"
            INSERT OR IGNORE INTO 
                senders(id, username, slug, color)
            VALUES 
                (?, ?, ?, ?)
        "#,
        message.sender.id,
        message.sender.username,
        message.sender.slug,
        message.sender.identity.color
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Erro no insert: {:?}", e);
        e
    })?;

    sqlx::query!(
        r#"
            INSERT OR IGNORE INTO 
                messages(id, chatroom_id, sender_id, content, type, created_at)
            VALUES
                ($1, $2, $3, $4, $5, $6)
        "#,
        message.id,
        message.chatroom_id,
        message.sender.id,
        message.content,
        message.message_type,
        message.created_at
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Erro no insert: {:?}", e);
        e
    })?;

    if let Some(badges) = message.sender.identity.badges {
        for badge in badges {
            println!("Inserindo selo: {:?}", badge);
            sqlx::query!(
                r#"
                INSERT OR IGNORE INTO 
                    badges(type, text, count) 
                VALUES 
                    ($1, $2, $3)
                "#,
                badge.badge_type,
                badge.text,
                badge.count
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                eprintln!("❌ Erro no insert de badges: {:?}", e);
                e
            })?;

            sqlx::query!(
                r#"
                INSERT OR IGNORE INTO sender_badges (sender_id, badge_id, chatroom_id)
                VALUES (
                    $1, 
                    (SELECT id FROM badges WHERE type = $2 AND text = $3 AND count IS $4),
                    $5
                )
                "#,
                message.sender.id,
                badge.badge_type,
                badge.text,
                badge.count,
                message.chatroom_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                eprintln!("❌ Erro ao inserir sender_badge: {:?}", e);
                e
            })?;
        }
    }
    tx.commit().await.map_err(|e| {
        eprintln!("❌ Erro no insert: {:?}", e);
        e
    })?;
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
    .await
    .map_err(|e| {
        eprintln!("Erro no insert: {:?}", e);
        e
    })?;

    Ok(())
}

pub async fn cleanup_messages_by_chatroom(
    pool: &SqlitePool,
    chatroom_id: i32,
) -> Result<(), sqlx::Error> {
    println!(
        "Iniciando limpeza de mensagens para o chatroom_id: {}",
        chatroom_id
    );
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
    .await
    .map_err(|e| {
        eprintln!("Erro na limpeza chat: {:?}", e);
        e
    })?;

    Ok(())
}
