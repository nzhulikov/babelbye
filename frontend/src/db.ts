import { openDB } from "idb";
import type { ChatMessage, UserProfile } from "./types";

const DB_NAME = "babelbye";
const DB_VERSION = 1;

export const dbPromise = openDB(DB_NAME, DB_VERSION, {
  upgrade(db) {
    if (!db.objectStoreNames.contains("messages")) {
      const store = db.createObjectStore("messages", { keyPath: "id" });
      store.createIndex("connection_id", "connection_id");
    }
    if (!db.objectStoreNames.contains("profiles")) {
      db.createObjectStore("profiles", { keyPath: "id" });
    }
  },
});

export type CachedMessage = ChatMessage;

export async function cacheProfile(profile: UserProfile) {
  const db = await dbPromise;
  await db.put("profiles", profile);
}

export async function loadProfile(id: string) {
  const db = await dbPromise;
  return db.get("profiles", id);
}

export async function cacheMessage(message: ChatMessage) {
  const db = await dbPromise;
    await db.put("messages", message);
}

export async function loadMessages(connectionId: string) {
  const db = await dbPromise;
  return db.getAllFromIndex("messages", "connection_id", connectionId);
}

export async function deleteMessages(connectionId: string) {
  const db = await dbPromise;
  const keys = await db.getAllKeysFromIndex("messages", "connection_id", connectionId);
  await Promise.all(keys.map((key) => db.delete("messages", key)));
}
