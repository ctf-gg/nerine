FROM lukemathwalker/cargo-chef@sha256:cf4bd956000c0b18613ce4e485e4a0c7719921fcc5e34ba0e7e08e3dfcff8964 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,id=cargo,target=/app/target cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN --mount=type=cache,id=cargo,target=/app/target cargo build --release && cp /app/target/release/sctf /app/sctf

FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/sctf /usr/local/bin/sctf
CMD ["/usr/local/bin/sctf"]
