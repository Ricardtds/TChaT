// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(unused_crate_dependencies)]

use rustls as _;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::fs;
use std::str::FromStr;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

mod commands;
mod models;
mod utils;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle: tauri::AppHandle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let app_dir = app_handle
                    .path()
                    .app_data_dir()
                    .expect("❌ Não foi possível obter o diretório de dados do app.");

                fs::create_dir_all(&app_dir)
                    .expect("❌ Não foi possível criar o diretório de dados do app.");

                let cache_dir = app_handle
                    .path()
                    .app_cache_dir()
                    .expect("❌ Não foi possível obter o diretório de cache do app.");

                eprintln!("✅ [RUST]: Caminho do cache: {:?}", cache_dir);

                // Cria o diretório se ele não existir
                // fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;

                let db_path = app_dir.join("tchat.db");

                println!(
                    "✅ [RUST]: Caminho do banco de dados: {}",
                    db_path.display()
                );

                let connect_options = SqliteConnectOptions::from_str(db_path.to_str().unwrap())
                    .expect("❌ Não foi possível criar opções de conexão")
                    .create_if_missing(true);

                let pool = SqlitePool::connect_with(connect_options)
                    .await
                    .expect("❌ Failed to connect to database.");

                if let Err(e) = MIGRATOR.run(&pool).await {
                    eprintln!("❌ Failed to run database migrations: {}", e);
                }

                println!("✅ [RUST]: Banco de dados inicializado com sucesso.");

                app_handle.manage(pool);
            });
            Ok(())
        })
        .manage(commands::WsState(Arc::new(Mutex::new(None))))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::subscribe_to_channel,
            commands::unsubscribe_from_channel,
            commands::get_chat_history,
            commands::read_emote_from_cache,
            commands::cache_emote_locally,
            commands::fetch_emote_from_web,
            commands::clear_emote_cache,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
