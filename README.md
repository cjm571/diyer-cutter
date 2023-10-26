# Build Process

## Dev Container

### Windows 11

#### Prerequisites

* Ubuntu under WSL2
* Install and configure [USBIPD-WIN](https://learn.microsoft.com/en-us/windows/wsl/connect-usb#install-the-usbipd-win-project) in Ubuntu

#### Procedure

0. Connect BBC micro:bit to PC via USB
1. Run `tools/setup.ps1` to attach the micro:bit to WSL (see [this](https://github.com/dorssel/usbipd-win/wiki/WSL-support) if error occurs)
2. Execute `Dev Containers: Reopen in Container`
3. Open a terminal in the container and execute `cargo run`