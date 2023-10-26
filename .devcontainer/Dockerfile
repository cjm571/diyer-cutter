FROM rust:1.73.0

# Install apt dependencies
RUN apt update && apt install -y \
    libudev-dev \
&& rm -rf /var/lib/apt/lists/*

WORKDIR /workspaces/diyer-cutter

# Add appropriate toolchain for micro:bit development
RUN rustup target add thumbv7em-none-eabihf