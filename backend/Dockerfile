# Multi-stage build: compile in a full Rust image, run in a slim Debian image.
# Built by CI (.github/workflows/docker-publish.yml), not on the target VPS -
# that box also runs Elevia on a single vCPU and can't absorb a second
# `cargo build --release` on every deploy.

FROM rust:1-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release --bin derniere-chance-api

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/derniere-chance-api ./derniere-chance-api
COPY --from=builder /app/migrations ./migrations

EXPOSE 8080
CMD ["./derniere-chance-api"]
