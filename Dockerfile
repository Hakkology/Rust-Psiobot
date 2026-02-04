# Build Stage
FROM rust:1.84-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev gcc make pkgconfig openssl-dev openssl-libs-static

WORKDIR /app

# Cache dependencies - bu layer sadece Cargo.toml değişirse rebuild olur
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Asıl kaynak kodu kopyala ve derle
COPY . .
RUN touch src/main.rs && cargo build --release

# Final Stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache libgcc openssl ca-certificates

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/psiobot /app/psiobot

# Run as non-root user
RUN adduser -D -u 1000 appuser
USER appuser

# Start the application
CMD ["./psiobot"]
