# Use the official Rust image as the base image
FROM --platform=linux/amd64 ubuntu@sha256:bbf3d1baa208b7649d1d0264ef7d522e1dc0deeeaaf6085bf8e4618867f03494 as deps

# Set non-interactive frontend for apt-get
ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo
ENV PATH=/opt/cargo/bin:$PATH

# Install necessary packages and dependencies
RUN apt-get update && apt-get install -y \
    curl \
    npm \
    qemu-user-static \
    tzdata \
    && rm -rf /var/lib/apt/lists/*

# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

# Manually install dfx
# Set the default Rust toolchain to stable
RUN rustup default stable

# Set the working directory
WORKDIR /usr/src/app

# Copy the source code into the Docker image
COPY . .

# Install Rust target
# Run cargo build first to download the necessary dependencies
RUN cargo build
#  RUN cargo test --test integration_tests
# Add docker ignore file to ignore the target directory
# Copy over the toml and lock file and then run cargo build. 



