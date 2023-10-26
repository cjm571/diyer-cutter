#!/usr/bin/env bash

# Give ownership of the cargo registry to the vscode user to allow writes and such to work
chown vscode:vscode /usr/local/cargo/registry

cargo install flip-link
cargo install probe-rs --features cli