FROM rust:1.89.0-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libsqlite3-dev \
    libssl-dev \
    libclang-dev \
    protobuf-compiler \
    nodejs \
    npm \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

RUN npm install -g pnpm@9

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY easytier/Cargo.toml easytier/
COPY easytier-rpc-build/Cargo.toml easytier-rpc-build/
COPY easytier-web/Cargo.toml easytier-web/
COPY vendor/ vendor/
COPY .cargo/ .cargo/

RUN mkdir -p easytier/src easytier-web/src easytier-rpc-build/src \
    && echo "fn main() {}" > easytier/src/main.rs \
    && echo "fn main() {}" > easytier/src/lib.rs \
    && echo "fn main() {}" > easytier-web/src/main.rs \
    && echo "fn main() {}" > easytier-rpc-build/src/lib.rs

RUN cargo build --release --package easytier-web 2>/dev/null || true

COPY . .

COPY pnpm-workspace.yaml pnpm-lock.yaml ./
WORKDIR /app
RUN pnpm install --frozen-lockfile || pnpm install

WORKDIR /app/easytier-web/frontend-lib
RUN pnpm build

WORKDIR /app/easytier-web/frontend
RUN pnpm build

WORKDIR /app

RUN cargo build --release --package easytier --package easytier-web --features embed

FROM debian:bookworm-slim

ENV TZ=Asia/Shanghai

RUN apt-get update && apt-get install -y ca-certificates sqlite3 python3 tzdata && rm -rf /var/lib/apt/lists/*

RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

COPY --from=builder /app/target/release/easytier-web /usr/local/bin/easytier-admin
COPY --from=builder /app/entrypoint.sh /entrypoint.sh
COPY --from=builder /app/target/release/easytier-core /usr/local/bin/easytier-core
RUN chmod +x /entrypoint.sh

ENV ET_ADMIN_USERNAME=admin
ENV ET_ADMIN_PASSWORD=admin123
ENV ET_ADMIN_SECRET=change-me-to-a-random-string

VOLUME ["/data"]

EXPOSE 11211 22020

CMD ["/entrypoint.sh"]
