FROM lukemathwalker/cargo-chef:latest-rust-1.86.0-bookworm AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin api-gateway

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin api-gateway

COPY Cargo.toml Cargo.lock ./
COPY services/api-gateway/Cargo.toml services/api-gateway/
RUN mkdir -p services/api-gateway/src && \
    echo "fn main() {}" > services/api-gateway/src/main.rs
RUN cargo build --release --bin api-gateway
RUN rm -rf services/api-gateway/src

COPY . .
RUN cargo build --release --bin api-gateway

FROM debian:bookworm-slim AS runtime

WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/api-gateway server/server
COPY services/api-gateway/configuration server/configuration
ENV APP_ENV=production
WORKDIR /app/server
ENTRYPOINT ["./server"]