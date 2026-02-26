# Seglamater Privacy Scanner

An open-source privacy scanner that evaluates websites against the [Seglamater Privacy Specification (SPS) v1.0](spec/v1.0.md). Scores sites from 0 to 100 across six categories and assigns a letter grade.

Available as a CLI tool for one-off scans and an HTTP API server with badge generation, scheduled scanning, and pluggable storage backends.

## Quick Start

### Install from source

```bash
git clone https://github.com/purpleneutral/seglamater-scan.git
cd seglamater-scan
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

**Response (200):**

```json
[
  { "domain": "example.com", "score": 78, "grade": "B", "scanned_at": "2026-02-25T14:30:00Z" },
  { "domain": "mozilla.org", "score": 83, "grade": "B", "scanned_at": "2026-02-25T14:25:00Z" }
]
```

### POST /api/domains

Register a domain for automatic scheduled re-scanning.

**Request:**

```json
{ "domain": "example.com", "interval_hours": 24 }
```

`interval_hours` defaults to `24` if omitted.

**Response (200):**

```json
{ "domain": "example.com", "interval_hours": 24, "status": "registered" }
```

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

Dynamic SVG badge (shields.io style). Returns `image/svg+xml` with 1-hour cache.

```
GET /badge/mozilla.org.svg
```

Returns an "unknown" badge with gray background if no scan exists for the domain.

**Embed in HTML:**

```html
<a href="https://your-server/api/verify/example.com">
  <img src="https://your-server/badge/example.com.svg" alt="SPS Score" height="20">
</a>
```

**Embed in Markdown:**

```markdown
![SPS Score](https://your-server/badge/example.com.svg)
```

## Scoring

The SPS v1.0 specification defines 24 checks across 6 categories, totaling 100 points.

### Grade Thresholds

| Grade | Score | Badge Color |
|-------|-------|-------------|
| A+ | 95-100 | ![#4c1](https://placehold.co/12/44cc11/44cc11) Bright green |
| A | 90-94 | ![#97ca00](https://placehold.co/12/97ca00/97ca00) Green |
| B | 75-89 | ![#007ec6](https://placehold.co/12/007ec6/007ec6) Blue |
| C | 60-74 | ![#dfb317](https://placehold.co/12/dfb317/dfb317) Yellow |
| D | 40-59 | ![#fe7d37](https://placehold.co/12/fe7d37/fe7d37) Orange |
| F | 0-39 | ![#e05d44](https://placehold.co/12/e05d44/e05d44) Red |

Only grades A+ and A are eligible for the "Seglamater Verified" designation.

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
# Build (SQLite included by default)
cargo build --release

# Run
seglamater-scan serve --database-url "sqlite://./scanner.db"
```

### PostgreSQL

For production deployments. Requires the `postgres` feature flag.

```bash
# Build with PostgreSQL support
cargo build --release --features postgres

# Or with both backends
cargo build --release --features "sqlite,postgres"

# Run
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
    async fn list_domains(&self, limit: i64, offset: i64) -> Result<Vec<ScanRecord>> { ... }
    async fn search_domains(&self, query: &str, limit: i64) -> Result<Vec<ScanRecord>> { ... }
    async fn register_domain(&self, domain: &str, interval_hours: i32) -> Result<()> { ... }
    async fn get_due_domains(&self) -> Result<Vec<String>> { ... }
    async fn mark_scanned(&self, domain: &str) -> Result<()> { ... }
    async fn get_stats(&self) -> Result<AggregateStats> { ... }
}
```

### Database Schema

The server creates two tables:

**scans**

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER/BIGSERIAL | Primary key |
| `domain` | TEXT | Scanned domain |
| `score` | INTEGER | Score 0-100 |
| `grade` | TEXT | Letter grade (A+, A, B, C, D, F) |
| `scan_data` | TEXT | Full JSON scan result |
| `scanned_at` | TIMESTAMP | When the scan was performed |

**registered_domains**

| Column | Type | Description |
|--------|------|-------------|
| `domain` | TEXT | Primary key |
| `registered_at` | TIMESTAMP | When registered |
| `scan_interval_hours` | INTEGER | Hours between scans (default: 24) |
| `last_scanned_at` | TIMESTAMP | Last scan timestamp (nullable) |

## Docker

### Build and run

```bash
docker build -t seglamater-scan .
docker run -p 8080:8080 -v scanner-data:/data seglamater-scan
```

### docker-compose

```bash
docker-compose up -d
```

The default `docker-compose.yml` runs the server on port 8080 with a SQLite database persisted to a Docker volume.

### With PostgreSQL

To use PostgreSQL instead of SQLite, set `DATABASE_URL` and build with the `postgres` feature. Modify the Dockerfile build line:

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
      - DATABASE_URL=postgres://scanner:password@db:5432/scanner
      - RUST_LOG=info
    depends_on:
      - db

  db:
    image: postgres:17
    environment:
      - POSTGRES_USER=scanner
      - POSTGRES_PASSWORD=password
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

### Crate Overview

| Crate | Purpose |
|-------|---------|
| `scanner-core` | Specification types, grade thresholds, check/category result types, text/JSON report formatting |
| `scanner-transport` | TLS version checks, HSTS header parsing |
| `scanner-headers` | CSP, Referrer-Policy, Permissions-Policy, X-Content-Type-Options, X-Frame-Options |
| `scanner-tracking` | HTML parsing, tracker/analytics domain matching, fingerprinting detection, CDN detection |
| `scanner-cookies` | Set-Cookie header parsing, attribute validation |
| `scanner-dns` | SPF, DKIM, DMARC, DNSSEC, CAA record checks |
| `scanner-bestpractices` | security.txt, privacy.json, JavaScript-free accessibility |
| `scanner-engine` | Shared scan logic: `run_scan()`, `fetch_page()`, `normalize_domain()`, recommendation generation |
| `scanner-server` | Axum HTTP server, Storage trait + SQLite/PostgreSQL backends, SVG badge generation, background scheduler |
| `scanner-cli` | Binary entry point with `scan` and `serve` subcommands |

## Scan Behavior

- **User-Agent:** `Mozilla/5.0 (compatible; SeglamaterScan/0.1; +https://seglamater.com/scan)`
- **Timeout:** 30 seconds per request
- **Redirects:** Up to 10 followed
- **TLS:** Valid certificates required (no insecure connections)
- **Parallelism:** Transport and DNS checks run concurrently; header, tracking, cookie, and best practice checks run after the page is fetched

### Domain Normalization

Input domains are automatically normalized:

- `https://Example.COM/path` becomes `example.com`
- `http://site.org:8080/` becomes `site.org`
- Leading/trailing whitespace is trimmed

## License

GPL-3.0-only. See [LICENSE](LICENSE) for details.

The [SPS v1.0 specification](spec/v1.0.md) is licensed under CC BY 4.0.
