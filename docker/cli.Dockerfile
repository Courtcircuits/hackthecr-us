FROM rust:1.95-slim AS chef
RUN cargo install cargo-chef
RUN apt-get update && apt-get install -y pkg-config musl-tools perl && rm -rf /var/lib/apt/lists/*
WORKDIR /app

FROM chef AS planner
ENV SQLX_OFFLINE=true
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV SQLX_OFFLINE=true
COPY --from=planner /app/recipe.json recipe.json
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo chef cook --target=x86_64-unknown-linux-musl --release --recipe-path recipe.json
COPY . .
RUN cargo build --target=x86_64-unknown-linux-musl --release -p cli

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/cli /usr/local/bin/cli
CMD ["/usr/local/bin/cli"]
