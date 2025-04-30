FROM lukemathwalker/cargo-chef:latest-rust-1.86.0-bookworm AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin auth-service

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin auth-service

COPY Cargo.toml Cargo.lock ./
COPY services/auth-service/Cargo.toml services/auth-service/
RUN mkdir -p services/auth-service/src && \
    echo "fn main() {}" > services/auth-service/src/main.rs
RUN cargo build --release --bin auth-service
RUN rm -rf services/auth-service/src

COPY . .
RUN cargo build --release --bin auth-service

FROM debian:bookworm-slim AS runtime

WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/auth-service server/server
COPY services/auth-service/configuration server/configuration
ENV APP_ENV=production
WORKDIR /app/server
ENTRYPOINT ["./server"]