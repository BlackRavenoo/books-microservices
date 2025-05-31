FROM lukemathwalker/cargo-chef:latest-rust-1.86.0-bookworm AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin ratings-service

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin ratings-service

COPY services/ratings-service/Cargo.toml services/ratings-service/
RUN mkdir -p services/ratings-service/src && \
    echo "fn main() {}" > services/ratings-service/src/main.rs
RUN cargo build --release --bin ratings-service
RUN rm -rf services/ratings-service/src

COPY crates crates
COPY services/ratings-service services/ratings-service
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin ratings-service

FROM debian:bookworm-slim AS runtime

WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ratings-service server/server
COPY services/ratings-service/configuration server/configuration
ENV APP_ENV=production
WORKDIR /app/server
ENTRYPOINT ["./server"]