# Use the official Rust image to create a build.
FROM rust:1.75 as builder

# Copy local code to the container image.
COPY . /app

# Set the working directory.
WORKDIR /app

# Build the release with cargo build, including optimizations.
RUN cargo build --release

# Use the official Debian slim image for a lean production container.
FROM debian:bullseye-slim AS final

# Copy the binary to the production image from the builder stage.
COPY --from=builder /app/target/release/email-sender-rust /app/email-sender-rust
WORKDIR /app

# Run the web service on container startup.
CMD ["./email-sender-rust"]

# Port 8080 is the default port for HTTP functions.
EXPOSE 8080
