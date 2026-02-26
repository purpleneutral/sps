FROM rust:1.87-bookworm AS builder

WORKDIR /app

# Install build dependencies for aws-lc-rs and SQLite
RUN apt-get update && apt-get install -y cmake clang pkg-config && rm -rf /var/lib/apt/lists/*

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/scanner-core/Cargo.toml crates/scanner-core/Cargo.toml
COPY crates/scanner-transport/Cargo.toml crates/scanner-transport/Cargo.toml
COPY crates/scanner-headers/Cargo.toml crates/scanner-headers/Cargo.toml
COPY crates/scanner-tracking/Cargo.toml crates/scanner-tracking/Cargo.toml
COPY crates/scanner-cookies/Cargo.toml crates/scanner-cookies/Cargo.toml
COPY crates/scanner-dns/Cargo.toml crates/scanner-dns/Cargo.toml
COPY crates/scanner-bestpractices/Cargo.toml crates/scanner-bestpractices/Cargo.toml
COPY crates/scanner-engine/Cargo.toml crates/scanner-engine/Cargo.toml
COPY crates/scanner-server/Cargo.toml crates/scanner-server/Cargo.toml
COPY crates/scanner-cli/Cargo.toml crates/scanner-cli/Cargo.toml

# Create stub lib.rs files for dependency caching
RUN for dir in crates/scanner-*/; do mkdir -p "$dir/src" && echo "" > "$dir/src/lib.rs"; done && \
    mkdir -p crates/scanner-cli/src && echo "fn main() {}" > crates/scanner-cli/src/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release 2>/dev/null || true

# Copy actual source code
COPY crates/ crates/
COPY blocklists/ blocklists/
COPY spec/ spec/

# Touch source files to invalidate cache and do a proper build
RUN find crates -name "*.rs" -exec touch {} + && cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/seglamater-scan /usr/local/bin/seglamater-scan

# Create data directory for SQLite
RUN mkdir -p /data
WORKDIR /data

ENV DATABASE_URL="sqlite:///data/scanner.db"

EXPOSE 8080

ENTRYPOINT ["seglamater-scan"]
CMD ["serve", "--host", "0.0.0.0", "--port", "8080"]
