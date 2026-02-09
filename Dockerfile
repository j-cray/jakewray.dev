# syntax=docker/dockerfile:1
FROM rust:bookworm as chef
WORKDIR /app
# Install cargo-binstall for faster tool installation
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
# Install cargo-chef
RUN cargo binstall cargo-chef -y
# Install cargo-leptos and sass (needed for build)
RUN cargo binstall cargo-leptos -y
# Install sqlx-cli (needed for pre-build prepare step)
RUN cargo binstall sqlx-cli -y --force
RUN apt-get update && apt-get install -y --no-install-recommends nodejs npm pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN npm install -g sass
# Add WASM target
RUN rustup target add wasm32-unknown-unknown

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --target wasm32-unknown-unknown --recipe-path recipe.json

# Build application
COPY . .
# Build the actual app
RUN cargo leptos build --release -vv && \
    cp -r target/release/backend /tmp/ && \
    cp -r target/site /tmp/
# Install sqlx-cli for runtime migrations/prep if needed (or just copy binary if available)
RUN cargo binstall sqlx-cli -y --force

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
COPY --from=builder /app/data /app/data

# Set environment
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="site"

EXPOSE 3000

CMD ["/app/backend"]
