$ErrorActionPreference = "Stop"

# Start WSL
wsl --exec cat /dev/null

# Ensure WSL remains running with a detached, neverending tail
wsl --exec tail -f /dev/null &

# Determine the bus ID of the micor:bit
$microbit_entry = usbipd wsl list -u | Select-String -Pattern "NXP, ARM mbed" | Out-String -NoNewline
$microbit_bus_id = (-split $microbit_entry)[0]

# Use USBIPD-win to attach the micro:bit to the WSL distro
usbipd wsl attach --auto-attach --busid $microbit_bus_id &