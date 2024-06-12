FROM rust:alpine AS builder

WORKDIR /build
COPY . .

RUN apk add musl-dev

RUN cargo version

RUN \
    --mount=type=cache,target=/build/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release && cp /build/target/release/dm-reports /build/dm-reports

FROM alpine:latest

WORKDIR /

COPY --from=builder /build/dm-reports /usr/bin/dm-reports

EXPOSE 8080

CMD ["/usr/bin/dm-reports"]

