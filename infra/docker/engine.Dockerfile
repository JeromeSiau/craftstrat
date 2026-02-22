FROM rust:1.85-bookworm AS builder

RUN apt-get update && apt-get install -y \
    cmake librdkafka-dev libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY engine/ .
RUN cargo build --release

# ---------------------------------------------------------------------------

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates librdkafka1 libssl3 libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/oddex-engine /usr/local/bin/oddex-engine

ENTRYPOINT ["oddex-engine"]
