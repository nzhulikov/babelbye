---
alwaysApply: true
---

# Babelbye Frontend Architecture Rules

## Platforms
- Web (React) now; Mobile (React Native) now; Desktop later
- Shared UI primitives and translation rendering logic

## UX and i18n
- Full Unicode support; RTL layouts required
- Context menu action to show original message
- Stream translated text while typing; finalize on send
- If quota exhausted, show original text with clear indicator

## Features (current)
- 1:1 chat and group chat
- Text + voice messages only
- Local encrypted cache for offline access
- Use IndexedDB for web storage

## State and data
- Feature-first module structure
- Separate transport, translation, and UI layers
- Avoid leaking content to logs or analytics

## Audio
- Voice capture/playback as first-class components
- Consistent waveform/playing state across platforms

## Auth
- Email/password, phone/password, Google sign-in
- Token refresh handled transparently; stale token allows cached reads

## Observability
- Track latency for transport and translation
- Track translation/search quotas used
- Sentry for runtime errors

## Testing
- Component tests for translation UI
- RTL layout checks
- Offline cache behavior tests

## Future hooks (do not implement yet)
- Media messages (images/files/video)
- Reactions and threads
