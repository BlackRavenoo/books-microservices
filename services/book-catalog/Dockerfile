FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin book-catalog

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin book-catalog
WORKDIR /app/server
COPY . .
RUN cargo build --release --bin book-catalog

FROM debian:bookworm-slim AS runtime

WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/server/target/release/book-catalog server/server
COPY services/book-catalog/configuration server/configuration
ENV APP_ENV=production
WORKDIR /app/server
ENTRYPOINT ["./server"]