FROM rust:1.90-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev

COPY Cargo.toml Cargo.lock /app/
RUN mkdir /app/crates

RUN cargo new --lib /app/crates/stoat
COPY crates/stoat/Cargo.toml /app/crates/stoat

RUN cargo new --bin /app/crates/highlight
COPY crates/highlight/Cargo.toml /app/crates/highlight

WORKDIR /app/
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

COPY crates/stoat /app/crates/stoat
COPY crates/highlight /app/crates/highlight

RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOF
  set -e
  # update timestamps to force a new build
  touch /app/crates/stoat/src/lib.rs /app/crates/highlight/src/main.rs
  cargo build --release
EOF

CMD ["/app/target/release/highlight"]

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/highlight /highlight

CMD ["/highlight"]