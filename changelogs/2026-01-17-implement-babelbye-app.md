## Summary
- Implemented Rust Axum backend with Auth0-aware auth, connection flow, and WS messaging.
- Built React + Vite frontend with Telegram-like UI and IndexedDB caching.
- Added Docker compose, frontend Dockerfile, and GitHub Actions CI/CD.

## Notes
- Auth bypass uses `x-user-id` for HTTP and `user_id` query param for WebSocket.
- Feedback issues are created when `FEEDBACK_REPO` and `GITHUB_TOKEN` are set.
