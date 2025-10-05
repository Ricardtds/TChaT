use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub chatroom_id: i32,
    pub content: String,
    pub created_at: String,
    pub sender: Sender,
    pub r#type: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct Sender {
    pub id: i32,
    pub username: String,
    pub identity: Identity,
}

#[derive(Serialize, Clone, Debug)]
pub struct Identity {
    pub color: String,
    pub badges: Vec<Option<Badge>>,
}


#[derive(Serialize, Clone, Debug)]
pub struct Badge {
    pub r#type: String,
    pub text: String,
    pub count: Option<i32>,
}
