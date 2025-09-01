import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

interface FetchedEmote {
  mimeType: string;
  data: number[];
}

export const emoteCache = writable<Map<string, string>>(new Map());

/**
 * Pega a URL de um emote, usando uma estratégia de cache em 3 níveis:
 * 1. Cache em Memória (rápido)
 * 2. Cache em Disco (persistente, via Rust)
 * 3. Fetch da Web (via Rust, para contornar o CORS)
 */

export async function getEmoteDataURL(emoteId: string): Promise<string> {
  const inMemoryCache = get(emoteCache); // 2. Use 'get' para ler o valor da store

  if (inMemoryCache.has(emoteId)) {
    return inMemoryCache.get(emoteId)!;
  }

  if (inMemoryCache.has(emoteId)) {
    return inMemoryCache.get(emoteId)!; // Retorno instantâneo!
  }

  try {
    const cachedDataUrl: string | null = await invoke("read_emote_from_cache", {
      emoteId: emoteId,
    });
    if (cachedDataUrl) {
      emoteCache.update((c) => c.set(emoteId, cachedDataUrl));
      return cachedDataUrl;
    }
  } catch (e) {
    console.error("Falha ao ler cache do Rust:", e);
  }

  const remoteUrl = `https://files.kick.com/emotes/${emoteId}/fullsize`;
  try {
    const result: FetchedEmote = await invoke("fetch_emote_from_web", {
      emoteId,
    });
    console.log("Objeto completo recebido do Rust:", result);

    await invoke("cache_emote_locally", {
      emoteId: emoteId,
      imageData: result.data,
      imagemMime: result.mimeType,
    });

    const base64String = btoa(String.fromCharCode(...result.data));
    const dataUrl = `data:${result.mimeType};base64,${base64String}`;

    emoteCache.update((c) => c.set(emoteId, dataUrl));
    return dataUrl;
  } catch (error) {
    console.error(`Erro ao carregar emote ${emoteId}:`, error);
    return remoteUrl;
  }
}
