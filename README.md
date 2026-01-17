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

## Local development
Backend:
- Copy `infra/env.example` to your environment or export values.
- Run `cargo run` in `backend` (migrations run automatically).

Frontend:
- Copy `frontend/env.example` into `.env` in `frontend` if you want a custom API URL.
- Run `npm install` then `npm run dev` in `frontend`.
- If `AUTH_BYPASS=true`, use a dev user id in the UI (Generate button).
- For Auth0, configure a Regular Web App and set `AUTH0_DOMAIN`, `AUTH0_AUDIENCE`, `AUTH0_ISSUER`.

## Docker compose
Run `docker compose -f infra/docker-compose.yml up --build`.

## Environment variables
- `DATABASE_URL`: Postgres connection string.
- `AUTH_BYPASS`: `true` to allow `x-user-id` header or `user_id` WS query param.
- `ALLOWED_ORIGINS`: comma-separated list for CORS.
- `AUTH0_DOMAIN`, `AUTH0_AUDIENCE`, `AUTH0_ISSUER`: Auth0 settings.
- `OPENAI_API_URL`, `OPENAI_API_KEY`, `OPENAI_MODEL`: OpenAI translation settings.
- `FEEDBACK_REPO`, `GITHUB_TOKEN`: optional GitHub issue feedback integration.
Examples are provided in `infra/env.example` and `frontend/env.example`.

## CI/CD
GitHub Actions builds backend/frontend on PRs and pushes Docker images to GHCR on `main`.

## Troubleshooting
- `CORS` errors: ensure `ALLOWED_ORIGINS` includes `http://localhost:5173`.
- `Auth0` errors: confirm the issuer and audience match your Auth0 tenant settings.
- Port conflicts: change `8080` or `5173` in `infra/docker-compose.yml` if already in use.

## Vibe coding workflow
- Agents + rapid iteration
- Keep scope tight and ship usable slices
- Prefer clarity over cleverness
