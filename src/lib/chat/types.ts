interface Identity {
  color: string;
  badges: any[];
}
export interface Sender {
  id: number;
  username: string;
  slug: string;
  identity: Identity;
  badges: any[];
}
export interface ChatMessage {
  id: string;
  chatId: number;
  content: string;
  createdAt: string;
  sender: Sender;
  type: string;
}

export interface Channel {
  id: number;
  name: string;
}
export interface ChatMessageWithHeight extends ChatMessage {
  _height: number;
}
