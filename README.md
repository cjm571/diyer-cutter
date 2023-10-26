# Build Process

## Dev Container

### Windows 11

#### Prerequisites

* Ubuntu under WSL2
* Install and configure [USBIPD-WIN](https://learn.microsoft.com/en-us/windows/wsl/connect-usb#install-the-usbipd-win-project) in Ubuntu

#### Procedure

0. Connect BBC micro:bit to PC via USB
1. Ensure Ubuntu is running in WSL
2. Determine the Bus ID of the micro:bit
   1. See Device Manager -> CMSIS-DAP v2 Properties -> Details Tab -> Hardware IDs
   2. Match against the VID:PID from `usbipd wsl list`, note the Bus ID (e.g., `2-2``)
3. `usbipd wsl attach --busid 2-2` to mount the micro:bit into Ubuntu (see [this](https://github.com/dorssel/usbipd-win/wiki/WSL-support) if error occurs)
4. Execute `Dev Containers: Reopen in Container`
5. Open a terminal in the container and execute `cargo run`
    1. If 