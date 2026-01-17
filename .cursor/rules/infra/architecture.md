---
alwaysApply: true
---

# Babelbye Infra/Platform Architecture Rules

## Packaging and deployment
- All services must be Docker-friendly
- Compose-first dev setup; production ready for orchestration
- Containerize every service with Docker

## Runtime topology
- API + realtime transport services scale horizontally
- Translation/search services isolated for quota and cost control
- No server-side storage of message content

## Security and privacy
- Secrets managed outside code (env/secret store)
- Encrypted payloads only; no plaintext in logs or traces
- Rate limit auth and translation endpoints

## Observability
- Cloud ELK for logs, Sentry for errors
- Metrics for latency (translation, transport, search)
- Throughput metrics per user/device and total data passed

## Reliability
- Health checks and graceful shutdown
- Backpressure handling on realtime streams
- Circuit breakers for AI provider failures

## CI/CD
- GitHub Actions for CI/CD

## Scalability targets
- 50 concurrent at start, 5k after 1 month, 1M users at 1 year
- Horizontal scaling validated with load tests

## Future hooks (do not implement yet)
- Media pipeline storage/CDN
- Moderation/reporting services
