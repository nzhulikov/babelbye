export type UserProfile = {
  id: string;
  email?: string | null;
  phone?: string | null;
  nickname: string;
  tagline?: string | null;
  native_language: string;
  is_searchable: boolean;
  translation_quota_remaining: number;
  created_at: string;
};

export type Connection = {
  id: string;
  requester_id: string;
  addressee_id: string;
  status: "pending" | "accepted" | "declined";
  created_at: string;
};

export type ChatMessage = {
  id: string;
  connection_id: string;
  from: string;
  to: string;
  text: string;
  original: string;
  translated: boolean;
  created_at: string;
  client_id?: string;
};

export type WsEvent =
  | {
      type: "message";
      from: string;
      text: string;
      original: string;
      translated: boolean;
      client_id?: string | null;
    }
  | {
      type: "delivery";
      to: string;
      status: "sent" | "typing";
      client_id?: string | null;
    }
  | {
      type: "error";
      message: string;
    };
