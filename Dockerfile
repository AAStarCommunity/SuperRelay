# Optimized multi-stage Dockerfile for Rundler deployment on Fly.io
# Adapted from https://github.com/paradigmxyz/reth/blob/main/Dockerfile
# syntax=docker/dockerfile:1.4

FROM rust:1.87.0 AS chef-builder

# Install system dependencies
RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add - && echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list
RUN mkdir -p /etc/apt/keyrings
RUN curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg
RUN echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_20.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list
RUN apt-get update && apt-get -y upgrade && apt-get install -y libclang-dev pkg-config protobuf-compiler nodejs yarn rsync

SHELL ["/bin/bash", "-c"]
RUN curl -L https://foundry.paradigm.xyz | bash
ENV PATH="/root/.foundry/bin:${PATH}"
RUN foundryup -i v0.3.0

RUN cargo install cargo-chef --locked

WORKDIR /app

# Builds a cargo-chef plan
FROM chef-builder AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef-builder AS builder
COPY --from=planner /app/recipe.json recipe.json

# Set the build profile to be release
ARG BUILD_PROFILE=release
ENV BUILD_PROFILE=$BUILD_PROFILE

# Builds dependencies
RUN cargo chef cook --profile $BUILD_PROFILE --recipe-path recipe.json

# Undo the source file changes made by cargo-chef.
# rsync invalidates the cargo cache for the changed files only, by updating their timestamps.
# This makes sure the fake empty binaries created by cargo-chef are rebuilt.
COPY --from=planner /app recipe-original
RUN rsync --recursive --checksum --itemize-changes --verbose recipe-original/ .
RUN rm -r recipe-original

RUN cargo build --profile $BUILD_PROFILE --locked --bin rundler

# Use Ubuntu as the release image optimized for Fly.io
FROM ubuntu:22.04 AS runtime
WORKDIR /app

# Install system dependencies for the runtime
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

# Create non-root user for security
RUN useradd -r -s /bin/false -u 1001 rundler

# Copy rundler binary from the build stage
COPY --from=builder /app/target/release/rundler /usr/local/bin/rundler

# Copy chain specifications for various networks
COPY --from=builder /app/bin/rundler/chain_specs ./chain_specs

# Create data directory for logs and potential state
RUN mkdir -p /data && chown rundler:rundler /data

# Set environment variables with defaults for Fly.io
ENV RUST_LOG=info
ENV METRICS_HOST=0.0.0.0
ENV METRICS_PORT=8080

# Health check for Fly.io monitoring
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Switch to non-root user
USER rundler

# Expose ports: 3000 for RPC, 8080 for metrics
EXPOSE 3000 8080

# Default command runs node mode (all services)
# Can be overridden with fly.toml or fly deploy --dockerfile-args
ENTRYPOINT ["/usr/local/bin/rundler"]
CMD ["node", \
     "--rpc.port", "3000", \
     "--rpc.host", "0.0.0.0", \
     "--metrics.port", "8080", \
     "--metrics.host", "0.0.0.0"]
