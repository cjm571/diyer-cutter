FROM rust:1.71.0

WORKDIR /workspaces/test-app

# Copy Cargo.toml into workdir during build so Cargo doesn't puke
COPY Cargo.toml /workspaces/test-app/Cargo.toml

# Add appropriate toolchain for micro:bit development
RUN rustup +nightly target add thumbv7em-none-eabihf

# Install apt dependencies
RUN apt update && apt install -y \
    libudev-dev \
&& rm -rf /var/lib/apt/lists/*

# Install cargo dependencies
RUN cargo install flip-link probe-run