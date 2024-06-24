ARG LLVMTARGETARCH
FROM --platform=${BUILDPLATFORM} ghcr.io/randomairborne/cross-cargo:${LLVMTARGETARCH} AS server-builder
ARG LLVMTARGETARCH

WORKDIR /build

COPY . .

RUN cargo build --release --target ${LLVMTARGETARCH}-unknown-linux-musl

FROM scratch
ARG LLVMTARGETARCH

WORKDIR /

COPY --from=builder /build/target/${LLVMTARGETARCH}-unknown-linux-musl/release/dm-reports /usr/bin/dm-reports

EXPOSE 8080

ENTRYPOINT ["/usr/bin/dm-reports"]

