FROM rust:1.95-slim AS builder

WORKDIR /usr/src/app
# Copy the entire project
COPY . .

# Build the release binary
RUN cargo build --release

# Create a lightweight runtime image
FROM debian:bookworm-slim

# Install ca-certificates required for rustls
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/pg-chaos-monkey /usr/local/bin/pg-chaos-monkey

# Run the binary
CMD ["pg-chaos-monkey"]
