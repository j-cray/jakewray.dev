# syntax=docker/dockerfile:1
FROM rust:bookworm as deps

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    nodejs \
    npm \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-binstall for faster tool installation
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Install cargo-leptos and sass (cached layer)
RUN cargo binstall cargo-leptos -y
RUN npm install -g sass

# Install sqlx-cli (binary install for speed)
RUN cargo binstall sqlx-cli -y --force

# Add WASM target
RUN rustup target add wasm32-unknown-unknown

FROM deps as planner
WORKDIR /app
# Copy only Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml ./backend/
COPY frontend/Cargo.toml ./frontend/
COPY shared/Cargo.toml ./shared/
COPY migration/Cargo.toml ./migration/

FROM deps as builder
WORKDIR /app

# Copy lockfiles first for better caching
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml ./backend/
COPY frontend/Cargo.toml ./frontend/
COPY shared/Cargo.toml ./shared/
COPY migration/Cargo.toml ./migration/

# Create dummy source files to cache dependencies
RUN mkdir -p backend/src frontend/src shared/src migration/src && \
    echo "fn main() {}" > backend/src/main.rs && \
    echo "fn main() {}" > migration/src/main.rs && \
    echo "pub fn dummy() {}" > shared/src/lib.rs && \
    echo "pub fn dummy() {}" > frontend/src/lib.rs

# Build dependencies only (this layer will be cached)
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cargo build --release --target wasm32-unknown-unknown -p frontend

# Now copy actual source code
COPY . .

# Touch files to trigger rebuild with real source
RUN touch backend/src/main.rs frontend/src/lib.rs shared/src/lib.rs

# Build the actual app with cached dependencies
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo leptos build --release -vv && \
    cp -r target/release/backend /tmp/ && \
    cp -r target/site /tmp/

# Runtime Stage
FROM debian:bookworm-slim as runtime
WORKDIR /app

# Install runtime dependencies (OpenSSL, ca-certificates)
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy artifacts from builder
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY --from=builder /tmp/backend /app/backend
COPY --from=builder /tmp/site /app/site

# Set environment
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="site"

EXPOSE 3000

CMD ["/app/backend"]
