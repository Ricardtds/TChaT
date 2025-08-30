export function parseEmotes(content: string): string {
  const emoteRegex = /\[emote:(\d+):(\w+)\]/g;
  return content.replace(emoteRegex, (match, emoteId, emoteName) => {
    const emoteUrl = `https://files.kick.com/emotes/${emoteId}/fullsize`;
    return `<img src="${emoteUrl}" alt="${emoteName}" class="emote" title="${emoteName}" style="max-height: 1.6em; vertical-align: middle;" />`;
  });
}
