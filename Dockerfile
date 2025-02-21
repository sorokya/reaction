# Builder
FROM rust:1.84-bookworm as builder

WORKDIR /usr/src

# Create blank project
RUN USER=root cargo new reaction

# We want dependencies cached, so copy those first.
COPY Cargo.toml Cargo.lock /usr/src/reaction/

# Set the working directory
WORKDIR /usr/src/reaction

# Install target platform (Cross-Compilation) --> Needed for Alpine
RUN apt update && apt install -y musl-tools musl-dev && \
    update-ca-certificates && \
    rustup target add x86_64-unknown-linux-musl

# This is a dummy build to get the dependencies cached.
RUN cargo build --target x86_64-unknown-linux-musl --release

# Now copy in the rest of the sources
COPY src /usr/src/reaction/src/

# Touch main.rs to prevent cached release build
RUN touch /usr/src/reaction/src/main.rs

# This is the actual application build.
RUN cargo build --target x86_64-unknown-linux-musl --release

# Runtime
FROM alpine:3.21.3 as runtime

# Copy application binary from builder image
COPY --from=builder /usr/src/reaction/target/x86_64-unknown-linux-musl/release/reaction /usr/bin/

EXPOSE 8080

WORKDIR /reaction

# Run the application
CMD ["/usr/bin/reaction"]
