FROM rust:alpine AS builder
ARG BINARY=dm-reports

RUN apk add musl-dev

WORKDIR /build

COPY . .

RUN cargo build --release --bin ${BINARY}

FROM scratch
ARG BINARY=dm-reports

WORKDIR /

COPY --from=builder /build/target/release/${BINARY} /usr/bin/${BINARY}

EXPOSE 8080

ENTRYPOINT ["/usr/bin/${BINARY}"]

