FROM rust:1.73.0

# Install apt dependencies
RUN apt update && apt install -y \
    libudev-dev \
&& rm -rf /var/lib/apt/lists/*

WORKDIR /workspaces/diyer-cutter

# Copy Cargo.toml into workdir during build so Cargo doesn't puke
COPY Cargo.toml /workspaces/diyer-cutter/Cargo.toml

# Add appropriate toolchain for micro:bit development
RUN rustup target add thumbv7em-none-eabihf

# Install cargo dependencies
RUN cargo install flip-link probe-run