FROM lukemathwalker/cargo-chef@sha256:cf4bd956000c0b18613ce4e485e4a0c7719921fcc5e34ba0e7e08e3dfcff8964 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,id=cargo,target=/app/target cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN --mount=type=cache,id=cargo,target=/app/target cargo build --bin nerine-api --release && cp /app/target/release/nerine-api /app/nerine-api

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install libssl3 ca-certificates -y
COPY --from=builder /app/nerine-api /usr/local/bin/nerine-api
CMD ["/usr/local/bin/nerine-api"]
