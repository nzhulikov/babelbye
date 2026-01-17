# Babelbye

Chat app with automatic translations to break language barriers.

## Who it is for
Travelers and nomads in foreign countries.

## MVP scope
In scope:
- Web app only (React)
- Sign up and profile
- Search by exact email/phone, or by nickname/tag line (user can hide)
- Connection request required before messaging
- Text messages only
- Language detection and automatic translation (user sets native language)
- Paid quota for extra translations
- Delete specific chat history or wipe all
- Feedback reports create GitHub issues via a special feedback user

Out of scope:
- Mobile/desktop apps
- File transfers, voice/video, group chats
- Threads, emojis, reactions, GIFs, stickers
- Blocks, spam reports
- AI search in messages

## Happy path
1. Sign up
2. Find a friend by email/phone/nickname/tag line
3. Ask to connect
4. Send the first message

## Privacy and security
- The server does not store message content.
- Messages are de-personalized and sent to a third-party translator.
- Communication is encrypted.

## Architecture snapshot
- Monolith backend in Rust to keep overhead low
- Auth0 for authentication
- Postgres for user config
- Websocket for realtime messaging
- IndexedDB for local web storage
- Docker-friendly runtime

## Vibe coding workflow
- Agents + rapid iteration
- Keep scope tight and ship usable slices
- Prefer clarity over cleverness
