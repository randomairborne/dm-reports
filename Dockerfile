ARG LLVMTARGETARCH
FROM --platform=${BUILDPLATFORM} ghcr.io/randomairborne/cross-cargo:${LLVMTARGETARCH} AS builder
ARG LLVMTARGETARCH
ARG BINARY

WORKDIR /build

COPY . .

RUN cargo build --release --target ${LLVMTARGETARCH}-unknown-linux-musl --bin ${BINARY}

FROM scratch
ARG LLVMTARGETARCH
ARG BINARY

WORKDIR /

COPY --from=builder /build/target/${LLVMTARGETARCH}-unknown-linux-musl/release/${BINARY} /usr/bin/${BINARY}

EXPOSE 8080

ENTRYPOINT ["/usr/bin/${BINARY}"]

