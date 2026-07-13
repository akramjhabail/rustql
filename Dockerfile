# Build stage
FROM rust:latest as builder

WORKDIR /app
COPY . .

# Offline mode use karo
ENV SQLX_OFFLINE=true
ENV DATABASE_URL=postgres://rustql:rustql123@db/rustqldb

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/rustql-cli .

EXPOSE 4000

CMD ["./rustql-cli"]