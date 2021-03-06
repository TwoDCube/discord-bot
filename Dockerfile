FROM rust:alpine as builder

RUN apk add musl-dev
WORKDIR /builder
RUN cargo new --bin app
WORKDIR /builder/app
COPY ["Cargo.toml", "Cargo.lock", "./"]
RUN cargo build --release && \
    rm -rf ./src

COPY src ./src
RUN rm target/release/deps/discord_bot* && \
    cargo build --release

FROM alpine
WORKDIR /app
COPY --from=builder /builder/app/target/release/discord-bot ./
CMD ["./discord-bot"]