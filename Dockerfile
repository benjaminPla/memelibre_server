# Stage 1: Build
FROM rust:latest AS builder

WORKDIR /usr/src/app

# Copy Cargo files and fetch deps early (cache optimization)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Copy actual source and build
COPY ./src ./src

# Copy templates and static assets into builder
COPY templates ./templates
COPY src/public ./public

RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

RUN useradd -m appuser
USER appuser

WORKDIR /home/appuser

# Copy compiled binary
COPY --from=builder /usr/src/app/target/release/memelibre .

# Copy templates and static files from builder
COPY --from=builder /usr/src/app/templates ./templates
COPY --from=builder /usr/src/app/public ./public

EXPOSE 3000

CMD ["./memelibre"]

