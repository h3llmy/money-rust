FROM rust:1.93-bookworm AS chef
RUN cargo install cargo-chef 
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# Install system dependencies for Diesel/Postgres
RUN apt-get update && apt-get install -y libpq-dev pkg-config && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --bin mobile-money-backend # force-rebuild-25

# Runtime stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install required runtime packages
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
        libpq5 \
        ca-certificates \
        curl \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/mobile-money-backend /usr/local/bin/mobile-money-backend

ENV APP_ENVIRONMENT=production
EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/mobile-money-backend"]