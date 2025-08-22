use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State, Manager};
use tokio_util::sync::CancellationToken;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

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

// O tipo para nosso gerenciador de estado
pub type ConnectionManager = Mutex<HashMap<String, CancellationToken>>;

#[tauri::command]
pub async fn connect_chat(
    // MUDANÇA: Agora recebemos o ID da sala de chat diretamente do front-end
    chatroom_id: String,
    app: AppHandle,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> { // MUDANÇA: Não retorna mais o histórico de mensagens

    // --- LÓGICA DE CONEXÃO EM TEMPO REAL ---
    let ws_url = "wss://ws-us2.pusher.com/app/32cbd69e4b950bf97679?protocol=7".to_string();
    println!("[RUST]: Conectando ao WebSocket para o chatroom {}", &chatroom_id);

    let (ws_stream, _) = connect_async(&ws_url).await.map_err(|e| e.to_string())?;
    let (mut write, mut read) = ws_stream.split();
    
    let token = CancellationToken::new();
    manager.lock().unwrap().insert(chatroom_id.clone(), token.clone());
    
    let subscribe_message = PusherEvent { 
        event: "pusher:subscribe".to_string(),
        data: SubscriptionData {
            channel: format!("chatrooms.{}.v2", chatroom_id),
        },
    };
    let json_message = serde_json::to_string(&subscribe_message).map_err(|e| e.to_string())?;
    write.send(Message::Text(json_message.into())).await.map_err(|e| e.to_string())?;
    
    let chatroom_id_clone_for_task = chatroom_id.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    println!("[RUST]: Conexão para o chatroom {} cancelada.", &chatroom_id_clone_for_task);
                    break;
                }
                Some(message) = read.next() => {
                    match message {
                        Ok(msg) => { if handle_message(msg, &app).is_err() { break; } }
                        Err(e) => {
                            eprintln!("❌ Erro ao receber mensagem do chatroom {}: {}", &chatroom_id_clone_for_task, e);
                            break;
                        }
                    }
                }
            }
        }
        let manager = app.state::<ConnectionManager>();
        manager.lock().unwrap().remove(&chatroom_id_clone_for_task);
        println!("[RUST]: Tarefa para o chatroom {} finalizada e limpa.", &chatroom_id_clone_for_task);
    });
    
    Ok(())
}

// Função auxiliar para processar cada mensagem recebida do WebSocket
fn handle_message(msg: Message, app: &AppHandle) -> Result<(), ()> {
    if let Message::Text(text) = msg {
        if let Ok(outer_message) = serde_json::from_str::<ApiOuterMessage>(&text) {
            if outer_message.event == "App\\Events\\ChatMessageEvent" {
                if let Ok(api_message) = serde_json::from_str::<ApiChatMessage>(&outer_message.data) {
                    println!("💬 Recebido de [{}]: {}", api_message.sender.username, api_message.content);
                    let app_message: AppChatMessage = api_message.into();
                    app.emit("new-chat-message", app_message).unwrap();
                }
            }
        }
    } else if let Message::Close(_) = msg {
        println!("🔌 Conexão fechada pelo servidor.");
        return Err(());
    }
    Ok(())
}

// Comando para encerrar uma conexão de chat específica
#[tauri::command]
pub async fn disconnect_chat(
    chatroom_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    println!("[RUST]: Solicitando desconexão do chatroom {}", &chatroom_id);
    if let Some(token) = manager.lock().unwrap().get(&chatroom_id) {
        token.cancel();
        Ok(())
    } else {
        Err(format!("[RUST]: Não foi possível encontrar conexão para o chatroom {}", &chatroom_id))
    }
}
