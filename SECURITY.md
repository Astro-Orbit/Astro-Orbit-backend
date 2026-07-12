# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ |

## Reporting a Vulnerability

**Do not open public issues for security vulnerabilities.**

Instead, report them privately to the Astro Orbit maintainers at
**security@astro-orbit.dev**. You should expect an acknowledgement
within 48 hours and a detailed response within 5 business days.

## Security Model

### Authentication

- Wallet-based authentication using Stellar Ed25519 keypairs
- JWT access tokens (15-minute TTL) with refresh token rotation
- Sessions stored in Redis with automatic expiry
- Rate limiting on auth endpoints (10 req/min)

### Authorization

- Role-based access control (RBAC) at the organization level
- Built-in roles: owner, admin, member, viewer
- Resource-level permission checks on every request
- API keys with scoped permissions

### Data Protection

- All passwords hashed with Argon2id
- API keys hashed with SHA-256 before storage (shown once)
- Encryption key for sensitive data at rest
- TLS required for all network communication
- PII minimized in logs (structured logging with redaction)

### API Security

- Rate limiting: 100 req/min per user, 10 req/min for auth
- Request body size limit: 10 MB
- Request timeout: 60 seconds
- CORS configured for allowed origins only
- Input validation at handler boundary using `validator` crate
- No SQL injection: parameterized queries via SQLx

### Infrastructure

- PostgreSQL with connection pooling and query timeout
- Redis with authentication and TLS support
- Docker containers run as non-root user
- Health checks on all services
- Graceful shutdown handling

## Threat Model

| Threat | Mitigation |
|--------|------------|
| JWT theft | Short TTL, refresh rotation, rate limiting |
| API key leak | Keys shown once, hashed, scoped permissions |
| SQL injection | Parameterized queries (SQLx) |
| CSRF | Token-based auth, not cookie-based |
| XSS | JSON API, no HTML rendering |
| DDoS | Rate limiting, connection pool limits |
| Brute force | Rate limiting, exponential backoff |
