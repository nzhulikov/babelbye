import type { Connection, UserProfile, WsEvent } from "./types";

const API_BASE = import.meta.env.VITE_API_URL ?? "http://localhost:8080";

const TOKEN_KEY = "babelbye_token";
const USER_ID_KEY = "babelbye_user_id";

export function getToken() {
  return localStorage.getItem(TOKEN_KEY) ?? "";
}

export function setToken(token: string) {
  localStorage.setItem(TOKEN_KEY, token);
}

export function getUserId() {
  return localStorage.getItem(USER_ID_KEY) ?? "";
}

export function setUserId(userId: string) {
  localStorage.setItem(USER_ID_KEY, userId);
}

function authHeaders() {
  const token = getToken();
  if (token) {
    return { Authorization: `Bearer ${token}` };
  }
  const userId = getUserId();
  return userId ? { "x-user-id": userId } : {};
}

async function apiFetch<T>(path: string, options: RequestInit = {}) {
  const headers: HeadersInit = {
    ...(options.headers ?? {}),
    ...authHeaders(),
  };

  const response = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers,
  });
  if (!response.ok) {
    const errorBody = await response.json().catch(() => ({}));
    throw new Error(errorBody.message ?? "request_failed");
  }
  if (response.status === 204) {
    return undefined as T;
  }
  return (await response.json()) as T;
}

export async function getProfile() {
  return apiFetch<UserProfile>("/api/profile");
}

export async function updateProfile(payload: {
  email?: string;
  phone?: string;
  nickname: string;
  tagline?: string;
  native_language: string;
  is_searchable: boolean;
}) {
  return apiFetch<UserProfile>("/api/profile", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
}

export async function searchUsers(query: string) {
  const params = new URLSearchParams({ query });
  return apiFetch<UserProfile[]>(`/api/search?${params.toString()}`);
}

export async function listConnections() {
  return apiFetch<Connection[]>("/api/connections");
}

export async function listPendingRequests() {
  return apiFetch<Connection[]>("/api/connections/requests");
}

export async function requestConnection(target_user_id: string) {
  return apiFetch<Connection>("/api/connections/request", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ target_user_id }),
  });
}

export async function respondConnection(requester_id: string, accept: boolean) {
  return apiFetch<Connection>("/api/connections/respond", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ requester_id, accept }),
  });
}

export async function deleteHistory(peerId?: string) {
  const path = peerId ? `/api/history/${peerId}` : "/api/history";
  return apiFetch<number>(path, { method: "DELETE" });
}

export async function submitFeedback(message: string) {
  return apiFetch<{ issue_url: string | null }>("/api/feedback", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ message }),
  });
}

export function wsUrl() {
  const url = new URL("/ws", API_BASE);
  const token = getToken();
  const userId = getUserId();
  if (token) {
    url.searchParams.set("token", token);
  } else if (userId) {
    url.searchParams.set("user_id", userId);
  }
  return url.toString().replace("http", "ws");
}

export type { WsEvent };
