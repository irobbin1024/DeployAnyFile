# syntax=docker/dockerfile:1

# ---------- Stage 1: build frontend ----------
FROM node:20-alpine AS frontend
WORKDIR /app/frontend
COPY frontend/package.json ./
RUN npm install
COPY frontend/ ./
RUN npm run build

# ---------- Stage 2: build backend ----------
FROM rust:1-bookworm AS backend
WORKDIR /app
# Cache dependencies first
COPY backend/Cargo.toml ./backend/Cargo.toml
COPY backend/migrations ./backend/migrations
RUN mkdir -p backend/src && echo "fn main() {}" > backend/src/main.rs
WORKDIR /app/backend
RUN cargo build --release || true
# Now copy real sources and build
COPY backend/src ./src
RUN touch src/main.rs && cargo build --release

# ---------- Stage 3: runtime ----------
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=backend /app/backend/target/release/deployanyfile /app/deployanyfile
COPY --from=frontend /app/frontend/dist /app/static

ENV BIND_ADDR=0.0.0.0:8080 \
    DATA_DIR=/app/data \
    STATIC_DIR=/app/static \
    MAX_UPLOAD_MB=100

RUN mkdir -p /app/data/uploads
VOLUME ["/app/data"]
EXPOSE 8080

CMD ["/app/deployanyfile"]
