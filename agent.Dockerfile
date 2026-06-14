FROM rust:1.89.0-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    build-essential pkg-config libssl-dev libclang-dev protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY easytier/Cargo.toml easytier/
COPY easytier-rpc-build/Cargo.toml easytier-rpc-build/
COPY vendor/ vendor/
COPY .cargo/ .cargo/

RUN mkdir -p easytier/src/bin easytier/src easytier-rpc-build/src \
    && echo "fn main() {}" > easytier/src/lib.rs \
    && echo "fn main() {}" > easytier/src/easytier-core.rs \
    && echo "fn main() {}" > easytier/src/easytier-cli.rs \
    && echo "fn main() {}" > easytier/src/bin/whitelist_sync.rs \
    && echo "fn main() {}" > easytier-rpc-build/src/lib.rs

RUN cargo build --release --bin easytier-core --bin easytier-whitelist-sync 2>/dev/null || true
COPY . .
RUN cargo build --release --bin easytier-core --bin easytier-whitelist-sync

FROM debian:bookworm-slim
ENV TZ=Asia/Shanghai
RUN apt-get update && apt-get install -y ca-certificates sqlite3 tzdata iproute2 \
    && rm -rf /var/lib/apt/lists/*
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

COPY --from=builder /app/target/release/easytier-core /usr/local/bin/easytier-core
COPY --from=builder /app/target/release/easytier-whitelist-sync /usr/local/bin/easytier-whitelist-sync
COPY --from=builder /app/easytier/scripts/agent-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

VOLUME ["/data"]

CMD ["/entrypoint.sh"]
