FROM rust:1.80-bookworm as builder
# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    nodejs \
    npm \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-binstall for faster tool installation
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Install cargo-leptos and sass
RUN cargo binstall cargo-leptos -y
RUN npm install -g sass

# Install sqlx-cli (compile from source for reliability)
RUN cargo install sqlx-cli --no-default-features --features postgres,rustls

# Add WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app
COPY . .

# Build the app (Release mode)
RUN cargo leptos build --release -vv

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
COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/target/site /app/site

# Set environment
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="site"

EXPOSE 3000

CMD ["/app/backend"]
