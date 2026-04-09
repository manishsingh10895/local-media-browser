FROM rust:1.94-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/local-gallery-server /usr/local/bin/local-gallery-server

EXPOSE 6677

CMD ["local-gallery-server"]
