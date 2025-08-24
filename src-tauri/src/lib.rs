#![warn(unused_crate_dependencies)]

use rustls as _;
use sqlx::SqlitePool; // Importe o SqlitePool
use std::sync::Arc;
use tauri_plugin_sql::{Migration, MigrationKind};
use tokio::sync::Mutex;
use tauri::{Manager};

mod commands;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        Migration {
            version: 1,
            sql: "CREATE TABLE IF NOT EXISTS senders (id INTEGER PRIMARY KEY, username TEXT,slug TEXT, color TEXT);",
            description: "create_messages_senders",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_messages_table",
            sql: "CREATE TABLE IF NOT EXISTS messages (id TEXT PRIMARY KEY, chatroom_id INTEGER, content TEXT, created_at TEXT, sender_id INTEGER, FOREIGN KEY(sender_id) REFERENCES senders(id));",
            kind: MigrationKind::Up,
        }
    ];

    tauri::Builder::default()
        .manage(commands::WsState(Arc::new(Mutex::new(None))))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:mydatabase.db", migrations)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let app_dir = app_handle
                    .path()
                    .app_data_dir()
                    .expect("Failed to get app data dir");
                let db_path = app_dir.join("mydatabase.db");
                let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.to_str().unwrap()))
                    .await
                    .expect("Failed to connect to database");
                app_handle.manage(pool);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::subscribe_to_channel,
            commands::unsubscribe_from_channel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}