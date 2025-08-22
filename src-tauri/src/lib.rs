// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::Mutex;

mod commands;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 2. Registra o estado para gerenciar as conexões.
        //    Isso disponibiliza o `ConnectionManager` para seus comandos.
        .manage(Mutex::new(HashMap::new()) as commands::ConnectionManager)
                .plugin(tauri_plugin_store::Builder::new().build()) // <-- ADICIONE ESTA LINHAS
        .plugin(tauri_plugin_opener::init()) // Seu plugin existente
        
        // 3. Registra TODOS os comandos necessários.
        .invoke_handler(tauri::generate_handler![
            commands::connect_chat,
            commands::disconnect_chat
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
