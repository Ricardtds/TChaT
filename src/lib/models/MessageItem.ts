export interface MessageItem {
    sender: Sender;
    time: string;
    text: string;
}

interface Sender {
    name: string;
    color: string;
}

interface Badge {
    name: string,
    src: string
}