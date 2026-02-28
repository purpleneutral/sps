# Seglamater Privacy Standard (SPS)

[![SPS Score](https://seglamater.app/api/privacy/badge/seglamater.app.svg)](https://seglamater.app/privacy/scan/seglamater.app)

An open-source privacy scanner that evaluates websites against the [Seglamater Privacy Specification (SPS) v1.0](spec/v1.0.md). Scores sites from 0 to 100 across six categories and assigns a letter grade.

Available as a CLI tool for one-off scans and an HTTP API server with badge generation, scheduled scanning, and pluggable storage backends.

**Try it live:** [seglamater.app/privacy](https://seglamater.app/privacy) — scan any website for free, no account required.

## Quick Start

### Install from source

Requires **Rust 1.85+** (edition 2024).

```bash
git clone https://github.com/purpleneutral/sps.git
cd sps
cargo build --release
```

The binary is at `target/release/seglamater-scan`.

### Scan a site

```bash
seglamater-scan scan example.com
```

Output:

```
Seglamater Privacy Scan — example.com
Specification: SPS v1.0

Score: 78/100 (Grade: B)

TRANSPORT SECURITY                           16/20
  PASS  [8] TLS 1.3 supported
  PASS  [4] TLS 1.0/1.1 disabled
  PASS  [4] HSTS enabled
  FAIL  [0] HSTS max-age >= 1 year
  ...
```

### Start the API server

```bash
seglamater-scan serve
```

The server starts on `http://0.0.0.0:8080` with a SQLite database by default.

## What It Checks

SPS evaluates 24 checks across 6 categories. Every check is binary — pass or fail. No partial credit.

| Category | Points | What It Measures |
|----------|--------|------------------|
| Transport Security | 20 | TLS 1.3, legacy protocol rejection, HSTS configuration |
| Security Headers | 20 | CSP, Referrer-Policy, Permissions-Policy, X-Content-Type-Options, X-Frame-Options |
| Tracking & Third Parties | 30 | Analytics scripts, ad trackers, fingerprinting, third-party CDNs, mixed content |
| Cookie Behavior | 15 | Third-party cookies, Secure/HttpOnly/SameSite flags, expiration |
| Email & DNS Security | 10 | SPF, DKIM, DMARC, DNSSEC, CAA records |
| Best Practices | 5 | security.txt, privacy.json, JavaScript-free accessibility |

Everything checked is publicly observable — no cooperation required from the site being scanned. The full methodology is documented in the [SPS v1.0 specification](spec/v1.0.md).

### Grade Thresholds

| Grade | Score |
|-------|-------|
| A+ | 95-100 |
| A | 90-94 |
| B | 75-89 |
| C | 60-74 |
| D | 40-59 |
| F | 0-39 |

## CLI Reference

```
seglamater-scan <COMMAND>
```

### `scan` — Run a privacy scan

```bash
seglamater-scan scan <DOMAIN> [OPTIONS]
```

| Argument/Option | Description | Default |
|----------------|-------------|---------|
| `<DOMAIN>` | Domain to scan (e.g., `mozilla.org`) | Required |
| `--format` | Output format: `text` or `json` | `text` |

**Examples:**

```bash
# Human-readable output
seglamater-scan scan mozilla.org

# JSON for automation
seglamater-scan scan duckduckgo.com --format json
```

### `serve` — Start the HTTP API server

```bash
seglamater-scan serve [OPTIONS]
```

| Option | Description | Default |
|--------|-------------|---------|
| `--host` | Address to bind to | `0.0.0.0` |
| `--port` | Port to listen on | `8080` |
| `--database-url` | Database connection string | `sqlite://./scanner.db` |

The `--database-url` can also be set via the `DATABASE_URL` environment variable.

**Examples:**

```bash
# Default (SQLite, port 8080)
seglamater-scan serve

# Custom port with PostgreSQL
seglamater-scan serve --port 3000 \
  --database-url "postgres://user:pass@localhost/seglamater"

# Using environment variable
DATABASE_URL="postgres://user:pass@db:5432/scanner" seglamater-scan serve
```

## API Reference

All endpoints are available when running `seglamater-scan serve`.

### POST /api/scan

Trigger a scan, store the result, and return it.

**Request:**

```json
{ "domain": "example.com" }
```

**Response (200):**

```json
{
  "domain": "example.com",
  "spec_version": { "major": 1, "minor": 0 },
  "scanned_at": "2026-02-25T14:30:00Z",
  "categories": [ ... ],
  "total_score": 78,
  "grade": "B",
  "recommendations": [ ... ]
}
```

### GET /api/verify/:domain

Get the latest scan result for a domain.

**Response (200):** Full scan result (same shape as `/api/scan` response).

**Response (404):** `{ "error": "No scan found for this domain" }`

### GET /api/history/:domain

Get scan history for a domain, most recent first.

| Query Parameter | Description | Default |
|----------------|-------------|---------|
| `limit` | Max records to return | `50` |

**Response (200):**

```json
[
  { "id": 42, "domain": "example.com", "score": 78, "grade": "B", "scanned_at": "2026-02-25T14:30:00Z" },
  { "id": 41, "domain": "example.com", "score": 76, "grade": "B", "scanned_at": "2026-02-24T14:30:00Z" }
]
```

### GET /api/domains

List all scanned domains with their latest score.

| Query Parameter | Description | Default |
|----------------|-------------|---------|
| `limit` | Records per page | `50` |
| `offset` | Pagination offset | `0` |

### POST /api/domains

Register a domain for automatic scheduled re-scanning.

**Request:**

```json
{ "domain": "example.com", "interval_hours": 24 }
```

`interval_hours` defaults to `24` if omitted.

### GET /api/domains/search

Search domains by prefix.

| Query Parameter | Description | Default |
|----------------|-------------|---------|
| `q` | Search prefix | Required |
| `limit` | Max results | `50` |

### GET /api/stats

Aggregate statistics across all scans.

**Response (200):**

```json
{
  "total_domains": 150,
  "total_scans": 847,
  "average_score": 72.5,
  "grade_distribution": { "a_plus": 12, "a": 30, "b": 68, "c": 25, "d": 10, "f": 5 }
}
```

### GET /badge/:domain.svg

Dynamic SVG badge ([shields.io](https://shields.io) style). Returns `image/svg+xml` with 1-hour cache.

Returns an "unknown" badge if no scan exists for the domain.

**Embed in HTML:**

```html
<a href="https://seglamater.app/privacy/scan/example.com">
  <img src="https://seglamater.app/api/privacy/badge/example.com.svg" alt="SPS Score">
</a>
```

**Embed in Markdown:**

```markdown
[![SPS Score](https://seglamater.app/api/privacy/badge/example.com.svg)](https://seglamater.app/privacy/scan/example.com)
```

## Scoring Details

### Categories and Checks

#### Transport Security (20 points)

| Check | Points | Pass Criteria |
|-------|--------|---------------|
| TLS 1.3 supported | 8 | Server negotiates TLS 1.3 |
| TLS 1.0/1.1 disabled | 4 | Server rejects legacy TLS |
| HSTS enabled | 4 | `Strict-Transport-Security` header present |
| HSTS max-age >= 1 year | 2 | `max-age` >= 31536000 |
| HSTS includeSubDomains | 1 | Directive present |
| HSTS preload | 1 | Directive present |

#### Security Headers (20 points)

| Check | Points | Pass Criteria |
|-------|--------|---------------|
| Content-Security-Policy present | 6 | Header exists |
| CSP blocks unsafe-inline | 3 | No `'unsafe-inline'` in script-src |
| CSP blocks unsafe-eval | 3 | No `'unsafe-eval'` in script-src |
| Referrer-Policy set | 3 | Restrictive value (`no-referrer`, `same-origin`, `strict-origin`, `strict-origin-when-cross-origin`) |
| Permissions-Policy set | 3 | Restricts at least 1 sensitive API |
| X-Content-Type-Options | 1 | Set to `nosniff` |
| X-Frame-Options | 1 | Set to `DENY` or `SAMEORIGIN` |

#### Tracking & Third Parties (30 points)

| Check | Points | Pass Criteria |
|-------|--------|---------------|
| No third-party analytics | 10 | No scripts from known analytics domains |
| No advertising/tracking scripts | 10 | No resources from known tracker domains |
| No fingerprinting patterns | 5 | No Canvas, WebGL, AudioContext, or FingerprintJS signatures |
| No third-party CDNs | 3 | All resources from first-party domain |
| All resources over HTTPS | 2 | No mixed content |

#### Cookie Behavior (15 points)

| Check | Points | Pass Criteria |
|-------|--------|---------------|
| No third-party cookies | 5 | No `Set-Cookie` from third parties |
| Secure flag on all cookies | 3 | Every cookie has `Secure` |
| HttpOnly flag on all cookies | 3 | Every cookie has `HttpOnly` |
| SameSite on all cookies | 2 | `SameSite=Strict` or `SameSite=Lax` |
| Reasonable expiration | 2 | No cookie expires beyond 1 year |

If no cookies are set, all checks pass (ideal behavior).

#### Email & DNS Security (10 points)

| Check | Points | Pass Criteria |
|-------|--------|---------------|
| SPF record strict | 3 | `v=spf1 ... -all` (hard fail) |
| DKIM discoverable | 2 | DKIM TXT record found via common selectors |
| DMARC policy enforced | 3 | `p=quarantine` or `p=reject` |
| DNSSEC enabled | 1 | DNSKEY records present |
| CAA record present | 1 | At least 1 CAA record |

#### Best Practices (5 points)

| Check | Points | Pass Criteria |
|-------|--------|---------------|
| security.txt present | 2 | `/.well-known/security.txt` returns 200 |
| privacy.json present | 2 | `/.well-known/privacy.json` returns valid JSON |
| Accessible without JS | 1 | HTML contains 20+ words without JavaScript |

## Storage Backends

The server supports pluggable storage backends via Cargo feature flags. Tables are created automatically on startup.

### SQLite (default)

Zero-configuration file-based database. Enabled by default.

```bash
cargo build --release
seglamater-scan serve --database-url "sqlite://./scanner.db"
```

### PostgreSQL

For production deployments. Requires the `postgres` feature flag.

```bash
cargo build --release --features postgres
seglamater-scan serve --database-url "postgres://user:pass@localhost:5432/seglamater"
```

### Custom Storage Backend

Implement the `Storage` trait from `scanner_server::storage`:

```rust
use scanner_server::storage::{Storage, ScanRecord, AggregateStats};

impl Storage for MyStorage {
    async fn store_scan(&self, domain: &str, score: u32, grade: &str, scan_data: &str) -> Result<i64> { ... }
    async fn get_latest_scan(&self, domain: &str) -> Result<Option<ScanRecord>> { ... }
    async fn get_history(&self, domain: &str, limit: i64) -> Result<Vec<ScanRecord>> { ... }
    // ... see storage/traits.rs for the full trait
}
```

## Docker

### Build and run

```bash
docker build -t seglamater-scan .
docker run -p 8080:8080 -v scanner-data:/data seglamater-scan
```

### docker-compose

```bash
docker compose up -d
```

The default `docker-compose.yml` runs the server on port 8080 with a SQLite database persisted to a Docker volume.

### With PostgreSQL

Set `DATABASE_URL` and build with the `postgres` feature. Modify the Dockerfile build line:

```dockerfile
RUN find crates -name "*.rs" -exec touch {} + && \
    cargo build --release --features postgres --no-default-features
```

Then in `docker-compose.yml`:

```yaml
services:
  scanner:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://scanner:<your-secure-password>@db:5432/scanner
      - RUST_LOG=info
    depends_on:
      - db

  db:
    image: postgres:17
    environment:
      - POSTGRES_USER=scanner
      - POSTGRES_PASSWORD=<your-secure-password>
      - POSTGRES_DB=scanner
    volumes:
      - pg-data:/var/lib/postgresql/data

volumes:
  pg-data:
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | Database connection string | `sqlite://./scanner.db` |
| `RUST_LOG` | Log level (`debug`, `info`, `warn`, `error`) | `info` |
| `SPS_API_KEY` | API key for write endpoints (unset = open mode) | *(unset)* |
| `SPS_CORS_ORIGINS` | Comma-separated allowed origins | `https://seglamater.app,https://seglamater.com` |

## Background Scheduler

When running in server mode, a background scheduler automatically re-scans registered domains.

- Checks for due domains every 5 minutes
- Waits 2 seconds between scans to be respectful to target servers
- Register domains via `POST /api/domains` with a custom `interval_hours`

## Architecture

```
scanner-core           Core types, scoring, report formatting
  ^
scanner-{transport, headers, tracking, cookies, dns, bestpractices}
  ^                    Individual check implementations
scanner-engine         Scan orchestration, page fetching, recommendations
  ^
scanner-server         HTTP API, badge generation, storage, scheduler
scanner-cli            CLI interface (scan + serve subcommands)
```

| Crate | Purpose |
|-------|---------|
| `scanner-core` | Specification types, grade thresholds, check/category result types, text/JSON report formatting |
| `scanner-transport` | TLS version checks, HSTS header parsing |
| `scanner-headers` | CSP, Referrer-Policy, Permissions-Policy, X-Content-Type-Options, X-Frame-Options |
| `scanner-tracking` | HTML parsing, tracker/analytics domain matching, fingerprinting detection, CDN detection |
| `scanner-cookies` | Set-Cookie header parsing, attribute validation |
| `scanner-dns` | SPF, DKIM, DMARC, DNSSEC, CAA record checks |
| `scanner-bestpractices` | security.txt, privacy.json, JavaScript-free accessibility |
| `scanner-engine` | Scan orchestration: `run_scan()`, `fetch_page()`, `normalize_domain()`, recommendation generation |
| `scanner-server` | Axum HTTP server, Storage trait + SQLite/PostgreSQL backends, SVG badge generation, background scheduler |
| `scanner-cli` | Binary entry point with `scan` and `serve` subcommands |

## Scan Behavior

- **User-Agent:** `Mozilla/5.0 (compatible; SeglamaterScan/0.1; +https://seglamater.app/privacy)`
- **Timeout:** 30 seconds per request
- **Redirects:** Up to 10 followed
- **TLS:** Valid certificates required (no insecure connections)
- **Parallelism:** Transport and DNS checks run concurrently; header, tracking, cookie, and best practice checks run after the page is fetched

### Domain Normalization

Input domains are automatically normalized:

- `https://Example.COM/path` becomes `example.com`
- `http://site.org:8080/` becomes `site.org`
- Leading/trailing whitespace is trimmed

## Roadmap

- **Browser extension** -- Available at [purpleneutral/sps-extension](https://github.com/purpleneutral/sps-extension). Shows the SPS grade for every site in your toolbar. Chrome and Firefox, Manifest V3.
- **CI/CD integration** — GitHub Action for automated privacy regression testing
- **Spec v1.1** — Additional checks based on community feedback
- **Blocklist updates** — Automated tracker/analytics list refresh from upstream sources

## Contributing

Contributions are welcome. If you find a false positive, a missing tracker, or a check that should be scored differently, open an issue with details.

For code contributions:

1. Fork the repository
2. Create a feature branch
3. Run `cargo test` and `cargo clippy` before submitting
4. Open a pull request with a clear description of the change

If you think the specification itself should change, open a discussion issue first — spec changes affect every scan.

## Support

If you find this project useful, you can [buy me a coffee](https://buymeacoffee.com/uniqueuserg).

## License

GPL-3.0-only. See [LICENSE](LICENSE) for details.

The [SPS v1.0 specification](spec/v1.0.md) is licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
