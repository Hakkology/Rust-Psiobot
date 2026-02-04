# Build Stage
FROM rust:1.81-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev gcc make pkgconfig openssl-dev

WORKDIR /app
COPY . .

# Build the application
RUN cargo build --release

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
