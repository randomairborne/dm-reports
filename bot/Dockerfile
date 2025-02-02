FROM rust:alpine AS builder

RUN apk add musl-dev

WORKDIR /build

COPY . .

RUN cargo build --release

FROM scratch

WORKDIR /

COPY --from=builder /build/target/release/dm-reports /usr/bin/dm-reports

EXPOSE 8080

ENTRYPOINT ["/usr/bin/dm-reports"]

