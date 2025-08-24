// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(unused_crate_dependencies)]

use rustls as _;
use sqlx::migrate::Migrator;
// --- MUDANÇA 1: Importa a struct de opções de conexão ---
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::fs;
use std::str::FromStr; // Necessário para criar SqliteConnectOptions a partir de uma string
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

mod commands;
mod models;

// Inclui os arquivos SQL da nossa pasta de migrações no binário da aplicação
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(commands::WsState(Arc::new(Mutex::new(None))))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Resolve o caminho correto para o banco de dados
                let app_dir = app_handle
                    .path()
                    .app_data_dir()
                    .expect("Não foi possível obter o diretório de dados do app");
                
                fs::create_dir_all(&app_dir).expect("Não foi possível criar o diretório de dados do app");
                let db_path = app_dir.join("mydatabase.db");

                println!("[RUST]: Caminho do banco de dados: {}", db_path.display());

                // --- MUDANÇA 2: Cria as opções de conexão explicitamente ---
                let connect_options = SqliteConnectOptions::from_str(db_path.to_str().unwrap())
                    .expect("Não foi possível criar opções de conexão")
                    .create_if_missing(true); // A ordem explícita para criar o arquivo se não existir

                // --- MUDANÇA 3: Usa .connect_with() para conectar com as opções customizadas ---
                let pool = SqlitePool::connect_with(connect_options)
                    .await
                    .expect("Falha ao conectar no banco de dados com opções");
                
                // Roda as migrações para criar as tabelas
                MIGRATOR.run(&pool).await.expect("Falha ao rodar as migrações do banco");
                
                println!("[RUST]: Banco de dados inicializado com sucesso.");

                // Adiciona o pool de conexões ao estado gerenciado do Tauri
                app_handle.manage(pool);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::subscribe_to_channel,
            commands::unsubscribe_from_channel,
            commands::get_chat_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}