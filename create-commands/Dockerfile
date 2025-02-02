FROM rust:alpine AS builder

RUN apk add musl-dev

WORKDIR /build

COPY . .

RUN cargo build --release

FROM scratch

WORKDIR /

COPY --from=builder /build/target/release/dm-reports-create-commands /usr/bin/dm-reports-create-commands

EXPOSE 8080

ENTRYPOINT ["/usr/bin/dm-reports-create-commands"]

