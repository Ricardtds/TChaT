interface Identity {
  color: string;
  badges: any[];
}
export interface Sender {
  id: number;
  username: string;
  slug: string;
  identity: Identity;
}
export interface ChatMessage {
  id: string;
  chatroomId: number;
  content: string;
  messageType: string;
  createdAt: string;
  sender: Sender;
}

export interface Channel {
  id: number;
  name: string;
}
export interface ChatMessageWithHeight extends ChatMessage {
  _height: number;
}
