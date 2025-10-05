import { error } from "@sveltejs/kit";
import axios from "axios";

export async function getChatInfo(streammer: string) {
    let resp = await axios.get(`https://kick.com/api/v2/channels/${streammer}`)
    if (resp.status == 200) {
        return resp
    } else {
        error
    }
}

export async function getChatHistory(chatroom: number) {
    let resp = await axios.get(`https://web.kick.com/api/v1/chat/${chatroom}/history`)
    if (resp.status == 200) {
        return resp
    } else {
        error
    }
}