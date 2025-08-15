# -----------------------------
# Builder image
# -----------------------------
FROM rust:1-bookworm AS builder

WORKDIR /app

# Cache deps
# Copy manifest first to leverage Docker layer caching
COPY Cargo.toml ./
# Create a dummy src to build deps
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

# Now copy the real source
COPY . .

# Build the actual binary
RUN cargo build --release

# -----------------------------
# Runtime image
# -----------------------------
FROM debian:bookworm-slim AS runtime

# Minimal runtime dependencies
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates tzdata \
  && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -u 10001 -m appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/actix-todo-api /app/actix-todo-api

# Create data directory for sqlite file db (if used)
RUN mkdir -p /app/data && chown -R appuser:appuser /app
USER appuser

ENV PORT=8080
ENV DATABASE_URL=sqlite:data/todos.db
EXPOSE 8080

# Healthcheck curl may not be present; rely on container orchestrator
ENTRYPOINT ["/app/actix-todo-api"]