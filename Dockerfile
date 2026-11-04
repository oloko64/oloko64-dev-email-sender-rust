# Build stage. Edition 2024 requires Rust >= 1.85; the full image includes perl/make for vendored OpenSSL.
FROM rust:1-bookworm AS builder

WORKDIR /app
COPY . .

RUN cargo build --release

# Runtime stage.
FROM debian:bookworm-slim AS final

# CA certificates are needed for HTTPS calls to SendGrid and Telegram.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/email-sender-rust /app/email-sender-rust
WORKDIR /app

ENV PORT=8080
EXPOSE 8080

CMD ["./email-sender-rust"]
