use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::StreamExt;
use futures_util::SinkExt;
use serde::{Serialize, Deserialize};


#[derive(Deserialize, Debug)]
struct OuterMessage {
    event: String,
    // `data` é lido como uma String, pois é um JSON aninhado
    data: String, 
    channel: String,
}

#[derive(Deserialize, Debug)]
struct ChatMessage {
    id: String,
    chatroom_id: u64,
    content: String,
    #[serde(rename = "type")] // Renomeia o campo, pois `type` é uma palavra reservada
    message_type: String,
    created_at: String,
    sender: Sender,
}

#[derive(Deserialize, Debug)]
struct Sender {
    id: u64,
    username: String,
    slug: String,
    identity: Identity,
}

#[derive(Deserialize, Debug)]
struct Identity {
    color: String,
    badges: Vec<serde_json::Value>, 
}

#[derive(Serialize)]
struct SubscriptionData {
    channel: String,
}

#[derive(Serialize)]
struct PusherEvent {
    event: String,
    data: SubscriptionData,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Definindo a URL do WebSocket diretamente no código.
    // Use `wss` para uma conexão segura.
    let ws_url = format!("wss://ws-us2.pusher.com/app/32cbd69e4b950bf97679?protocol=7");

    // 2. Passe a REFERÊNCIA da string para connect_async
    println!("Conectando ao WebSocket em: {}", ws_url);
    let (ws_stream, _) = connect_async(&ws_url).await?; // <- Corrigido aqui!
    let (mut write,mut read) = ws_stream.split();

    let subscribe_message = PusherEvent {
        event: "pusher:subscribe".to_string(),
        data: SubscriptionData {
            channel: "chatrooms.66973867.v2".to_string(),
        },
    };

    let json_message = serde_json::to_string(&subscribe_message)?;

    println!("➡️  Enviando JSON: {}", json_message);
    write.send(tokio_tungstenite::tungstenite::Message::Text(json_message.into())).await?;
    println!("✅ Mensagem de inscrição enviada com sucesso.");


    println!("Conexão estabelecida com sucesso!");


    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                println!("Mensagem recebida: {}", text);
            }
            Message::Close(_) => {
                println!("Conexão fechada pelo servidor.");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}