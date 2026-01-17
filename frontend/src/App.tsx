import { useEffect, useMemo, useRef, useState } from "react";
import {
  deleteHistory,
  getProfile,
  getToken,
  getUserId,
  listConnections,
  listPendingRequests,
  requestConnection,
  respondConnection,
  searchUsers,
  setToken,
  setUserId,
  submitFeedback,
  updateProfile,
  wsUrl,
} from "./api";
import { cacheMessage, cacheProfile, deleteMessages, loadMessages } from "./db";
import type { ChatMessage, Connection, UserProfile, WsEvent } from "./types";

const DEFAULT_LANGUAGE = "en";

type MessageMap = Record<string, ChatMessage[]>;
type TypingMap = Record<string, boolean>;

function App() {
  const [tokenInput, setTokenInput] = useState("");
  const [userIdInput, setUserIdInput] = useState("");
  const [token, setTokenState] = useState(() => getToken());
  const [userId, setUserIdState] = useState(() => getUserId());
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [profileDraft, setProfileDraft] = useState<UserProfile | null>(null);
  const [connections, setConnections] = useState<Connection[]>([]);
  const [pending, setPending] = useState<Connection[]>([]);
  const [selectedConnection, setSelectedConnection] = useState<Connection | null>(null);
  const [messages, setMessages] = useState<MessageMap>({});
  const [typing, setTyping] = useState<TypingMap>({});
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<UserProfile[]>([]);
  const [messageDraft, setMessageDraft] = useState("");
  const [showOriginal, setShowOriginal] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const typingTimer = useRef<number | null>(null);

  const selectedMessages = useMemo(() => {
    if (!selectedConnection) {
      return [];
    }
    return messages[selectedConnection.id] ?? [];
  }, [messages, selectedConnection]);

  useEffect(() => {
    if (!token && !userId) {
      return;
    }
    setToken(token);
    setUserId(userId);
    refreshAll();
    connectWs();
    return () => {
      wsRef.current?.close();
    };
  }, [token, userId]);

  useEffect(() => {
    if (!selectedConnection) {
      return;
    }
    loadMessages(selectedConnection.id).then((cached) => {
      if (cached.length === 0) {
        return;
      }
      setMessages((prev) => ({
        ...prev,
        [selectedConnection.id]: cached.sort((a, b) =>
          a.created_at.localeCompare(b.created_at)
        ),
      }));
    });
  }, [selectedConnection]);

  async function refreshAll() {
    try {
      const [connectionsData, pendingData] = await Promise.all([
        listConnections(),
        listPendingRequests(),
      ]);
      setConnections(connectionsData.filter((item) => item.status === "accepted"));
      setPending(pendingData);
      try {
        const profileData = await getProfile();
        setProfile(profileData);
        setProfileDraft(profileData);
        cacheProfile(profileData);
        if (!userId) {
          setUserIdState(profileData.id);
        }
      } catch {
        if (userId) {
          setProfileDraft({
            id: userId,
            nickname: "Traveler",
            native_language: DEFAULT_LANGUAGE,
            is_searchable: true,
            translation_quota_remaining: 1000,
            created_at: new Date().toISOString(),
          });
        }
      }
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "failed_to_load");
    }
  }

  function connectWs() {
    const ws = new WebSocket(wsUrl());
    ws.onmessage = async (event) => {
      const payload = JSON.parse(event.data) as WsEvent;
      if (payload.type === "message") {
        const connectionId = resolveConnectionId(payload.from);
        if (!connectionId || !userId) {
          return;
        }
        const message: ChatMessage = {
          id: payload.client_id ?? `${connectionId}-${payload.from}-${Date.now()}`,
          connection_id: connectionId,
          from: payload.from,
          to: userId,
          text: payload.text,
          original: payload.original,
          translated: payload.translated,
          created_at: new Date().toISOString(),
          client_id: payload.client_id ?? undefined,
        };
        await cacheMessage(message);
        setMessages((prev) => appendMessage(prev, connectionId, message));
      } else if (payload.type === "delivery") {
        if (payload.status === "typing") {
          const connectionId = resolveConnectionId(payload.to);
          if (!connectionId) {
            return;
          }
          setTyping((prev) => ({ ...prev, [connectionId]: true }));
          setTimeout(() => {
            setTyping((prev) => ({ ...prev, [connectionId]: false }));
          }, 1200);
        }
      } else if (payload.type === "error") {
        setStatus(payload.message);
      }
    };
    ws.onclose = () => {
      wsRef.current = null;
    };
    ws.onerror = () => {
      setStatus("ws_error");
    };
    wsRef.current = ws;
  }

  function appendMessage(
    current: MessageMap,
    connectionId: string,
    message: ChatMessage
  ) {
    const list = current[connectionId] ?? [];
    if (list.some((item) => item.id === message.id)) {
      return current;
    }
    return { ...current, [connectionId]: [...list, message] };
  }

  function resolveConnectionId(peerId: string) {
    const match = connections.find(
      (connection) =>
        (connection.requester_id === peerId && connection.addressee_id === userId) ||
        (connection.addressee_id === peerId && connection.requester_id === userId)
    );
    return match?.id ?? null;
  }

  function handleLogin() {
    const tokenValue = tokenInput.trim();
    const userIdValue = userIdInput.trim();
    if (tokenValue) {
      setTokenState(tokenValue);
    }
    if (userIdValue) {
      setUserIdState(userIdValue);
    }
    setTokenInput("");
    setUserIdInput("");
  }

  function handleGenerateUserId() {
    const id = crypto.randomUUID();
    setUserIdInput(id);
  }

  async function handleSearch() {
    if (!searchQuery.trim()) {
      return;
    }
    try {
      const results = await searchUsers(searchQuery.trim());
      setSearchResults(results);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "search_failed");
    }
  }

  async function handleRequestConnection(targetId: string) {
    try {
      await requestConnection(targetId);
      setStatus("request_sent");
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "request_failed");
    }
  }

  async function handleRespond(requesterId: string, accept: boolean) {
    try {
      await respondConnection(requesterId, accept);
      await refreshAll();
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "respond_failed");
    }
  }

  async function handleSend() {
    if (!selectedConnection || !messageDraft.trim() || !userId) {
      return;
    }
    const text = messageDraft.trim();
    setMessageDraft("");

    const peerId =
      selectedConnection.requester_id === userId
        ? selectedConnection.addressee_id
        : selectedConnection.requester_id;
    const clientId = crypto.randomUUID();
    const optimistic: ChatMessage = {
      id: clientId,
      connection_id: selectedConnection.id,
      from: userId,
      to: peerId,
      text,
      original: text,
      translated: false,
      created_at: new Date().toISOString(),
      client_id: clientId,
    };

    await cacheMessage(optimistic);
    setMessages((prev) => appendMessage(prev, selectedConnection.id, optimistic));

    wsRef.current?.send(
      JSON.stringify({ type: "message", to: peerId, text, client_id: clientId })
    );
  }

  async function handleDeleteHistory() {
    if (!selectedConnection || !userId) {
      return;
    }
    const peerId =
      selectedConnection.requester_id === userId
        ? selectedConnection.addressee_id
        : selectedConnection.requester_id;
    try {
      await deleteHistory(peerId);
      await deleteMessages(selectedConnection.id);
      setMessages((prev) => ({ ...prev, [selectedConnection.id]: [] }));
      setStatus("history_deleted");
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "history_failed");
    }
  }

  async function handleProfileSave() {
    if (!profileDraft) {
      return;
    }
    try {
      const updated = await updateProfile({
        email: profileDraft.email ?? undefined,
        phone: profileDraft.phone ?? undefined,
        nickname: profileDraft.nickname,
        tagline: profileDraft.tagline ?? undefined,
        native_language: profileDraft.native_language ?? DEFAULT_LANGUAGE,
        is_searchable: profileDraft.is_searchable,
      });
      setProfile(updated);
      setProfileDraft(updated);
      cacheProfile(updated);
      setStatus("profile_saved");
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "profile_failed");
    }
  }

  async function handleFeedbackSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = event.currentTarget;
    const data = new FormData(form);
    const message = String(data.get("message") ?? "");
    if (!message) {
      return;
    }
    try {
      await submitFeedback(message);
      form.reset();
      setStatus("feedback_sent");
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "feedback_failed");
    }
  }

  function handleTyping() {
    if (!selectedConnection || !userId) {
      return;
    }
    const peerId =
      selectedConnection.requester_id === userId
        ? selectedConnection.addressee_id
        : selectedConnection.requester_id;
    if (typingTimer.current) {
      window.clearTimeout(typingTimer.current);
    }
    wsRef.current?.send(JSON.stringify({ type: "typing", to: peerId }));
    typingTimer.current = window.setTimeout(() => {
      typingTimer.current = null;
    }, 800);
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">Babelbye</div>
        {!token && !userId && (
          <div className="card">
            <h3>Sign in</h3>
            <p>Use Auth0 token or dev user id when auth bypass is enabled.</p>
            <input
              placeholder="Auth0 token (optional)"
              value={tokenInput}
              onChange={(event) => setTokenInput(event.target.value)}
            />
            <div className="inline">
              <input
                placeholder="Dev user id"
                value={userIdInput}
                onChange={(event) => setUserIdInput(event.target.value)}
              />
              <button type="button" onClick={handleGenerateUserId}>
                Generate
              </button>
            </div>
            <button onClick={handleLogin}>Continue</button>
          </div>
        )}

        {(token || userId) && (
          <>
            <div className="card">
              <h3>Profile</h3>
              <label>
                Nickname
                <input
                  value={profileDraft?.nickname ?? ""}
                  onChange={(event) =>
                    setProfileDraft((prev) =>
                      prev ? { ...prev, nickname: event.target.value } : prev
                    )
                  }
                />
              </label>
              <label>
                Tagline
                <input
                  value={profileDraft?.tagline ?? ""}
                  onChange={(event) =>
                    setProfileDraft((prev) =>
                      prev ? { ...prev, tagline: event.target.value } : prev
                    )
                  }
                />
              </label>
              <label>
                Native language
                <input
                  value={profileDraft?.native_language ?? DEFAULT_LANGUAGE}
                  onChange={(event) =>
                    setProfileDraft((prev) =>
                      prev ? { ...prev, native_language: event.target.value } : prev
                    )
                  }
                />
              </label>
              <label className="checkbox">
                <input
                  type="checkbox"
                  checked={profileDraft?.is_searchable ?? true}
                  onChange={(event) =>
                    setProfileDraft((prev) =>
                      prev ? { ...prev, is_searchable: event.target.checked } : prev
                    )
                  }
                />
                Show in search
              </label>
              <button onClick={handleProfileSave}>Save</button>
            </div>

            <div className="card">
              <h3>Find people</h3>
              <div className="inline">
                <input
                  placeholder="Email, phone, nickname"
                  value={searchQuery}
                  onChange={(event) => setSearchQuery(event.target.value)}
                />
                <button onClick={handleSearch}>Search</button>
              </div>
              <div className="list">
                {searchResults.map((user) => (
                  <div className="list-item" key={user.id}>
                    <div>
                      <strong>{user.nickname || "Unknown"}</strong>
                      <span>{user.tagline ?? "No tagline"}</span>
                    </div>
                    <button onClick={() => handleRequestConnection(user.id)}>Connect</button>
                  </div>
                ))}
                {!searchResults.length && <p className="muted">No results yet.</p>}
              </div>
            </div>

            <div className="card">
              <h3>Requests</h3>
              <div className="list">
                {pending.map((request) => (
                  <div className="list-item" key={request.id}>
                    <span>Request from {request.requester_id.slice(0, 8)}</span>
                    <div className="inline">
                      <button onClick={() => handleRespond(request.requester_id, true)}>
                        Accept
                      </button>
                      <button onClick={() => handleRespond(request.requester_id, false)}>
                        Decline
                      </button>
                    </div>
                  </div>
                ))}
                {!pending.length && <p className="muted">No pending requests.</p>}
              </div>
            </div>

            <div className="card">
              <h3>Feedback</h3>
              <form onSubmit={handleFeedbackSubmit} className="feedback-form">
                <textarea name="message" placeholder="Your feedback" />
                <button type="submit">Send</button>
              </form>
            </div>
          </>
        )}
      </aside>

      <main className="chat-area">
        <header className="chat-header">
          <div>
            <h2>{selectedConnection ? "Conversation" : "Select a chat"}</h2>
            {profile && (
              <p>
                Translation quota remaining: {profile.translation_quota_remaining}
              </p>
            )}
          </div>
          <div className="inline">
            <label className="checkbox">
              <input
                type="checkbox"
                checked={showOriginal}
                onChange={(event) => setShowOriginal(event.target.checked)}
              />
              Show original text
            </label>
            <button className="ghost" onClick={handleDeleteHistory}>
              Clear history
            </button>
          </div>
        </header>

        <section className="chat-body">
          {!connections.length && <p className="muted">No connections yet.</p>}
          {connections.map((connection) => (
            <button
              key={connection.id}
              className={`conversation-button ${
                selectedConnection?.id === connection.id ? "active" : ""
              }`}
              onClick={() => setSelectedConnection(connection)}
            >
              <span>
                Chat {connection.id.slice(0, 6)}{" "}
                {typing[connection.id] ? "• typing..." : ""}
              </span>
              <small>{new Date(connection.created_at).toLocaleDateString()}</small>
            </button>
          ))}
        </section>

        <section className="messages">
          {selectedMessages.map((message) => (
            <div
              key={message.id}
              className={`bubble ${message.from === userId ? "outgoing" : "incoming"}`}
            >
              <p>{showOriginal ? message.original : message.text}</p>
              <span>
                {message.translated ? "translated" : "original"} •{" "}
                {new Date(message.created_at).toLocaleTimeString()}
              </span>
            </div>
          ))}
        </section>

        <footer className="composer">
          <input
            placeholder="Type a message"
            value={messageDraft}
            onChange={(event) => {
              setMessageDraft(event.target.value);
              handleTyping();
            }}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                handleSend();
              }
            }}
          />
          <button onClick={handleSend}>Send</button>
        </footer>
        {status && <div className="status">{status}</div>}
      </main>
    </div>
  );
}

export default App;
