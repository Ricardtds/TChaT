export type MessagePart =
  | { type: "text"; content: string }
  | { type: "emote"; id: string; name: string };

/**
 * Divide uma mensagem de chat em um array de partes de texto e emotes.
 * @param content - O conteúdo da mensagem.
 * @returns Um array de partes da mensagem.
 */
export function parseMessage(content: string): MessagePart[] {
  const emoteRegex = /\[emote:(\d+):(\w+)\]/g;
  const parts: MessagePart[] = [];
  let lastIndex = 0;
  let match;

  while ((match = emoteRegex.exec(content)) !== null) {
    if (match.index > lastIndex) {
      parts.push({
        type: "text",
        content: content.substring(lastIndex, match.index),
      });
    }
    parts.push({ type: "emote", id: match[1], name: match[2] });
    lastIndex = emoteRegex.lastIndex;
  }

  if (lastIndex < content.length) {
    parts.push({ type: "text", content: content.substring(lastIndex) });
  }

  return parts;
}
