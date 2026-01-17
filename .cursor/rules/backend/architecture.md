---
alwaysApply: true
---

# Babelbye Backend Architecture Rules

## Scope
- Rust services for API, realtime transport, translation, and search
- Message content must never be stored on the server
- End-to-end encrypted payloads only; server is blind
- Use Postgres for backend metadata and system data

## Clean Architecture
- Layers: `domain`, `use_cases`, `ports`, `adapters`, `delivery`
- Domain and use-cases never depend on delivery/adapters
- All vendor integrations live behind port traits

## Provider-agnostic AI
- `TranslationPort` abstracts OpenAI (swappable)
- `SearchPort` uses lower-cost LLMs
- No direct OpenAI calls from delivery or domain layers

## Realtime + translation flow
- WebSocket relay for typing streams and final messages
- Translate on receive per recipient locale
- Stream sentence-by-sentence; finalize on send
- Max end-to-end latency target: <= 1s
- If translation quota exhausted, deliver original text only

## Data handling & privacy
- Store only minimal metadata (routing, quotas, delivery status)
- No plaintext content in logs, traces, or analytics
- Device-to-device history sync after explicit user approval

## Auth & identity
- Auth0 for authentication (web-only for now)
- Token-based auth for API and realtime
- Device binding supported

## Observability
- Metrics for translation/search quotas and latency
- Transport latency and throughput per user/device
- Cloud ELK for logs and Sentry for errors

## Scalability targets
- 50 concurrent at start, 5k after 1 month, 1M users at 1 year
- Horizontal scaling ready from day one

## Testing
- Unit tests for domain/use-cases
- Contract tests for ports/adapters
- Load tests for realtime and translation latency

## Future hooks (do not implement yet)
- Media pipeline for images/files/video
- Moderation/reporting flows
- Threads and reactions schema extensions
