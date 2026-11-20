# NASA Wallpaper Helper for Debian / Linux / WSL

A lightweight Bash script to download and set NASA wallpapers on Linux desktop environments and Windows Subsystem for Linux (WSL).

## Requirements

- `bash` (v4+)
- `curl`
- `jq`

Install requirements on Debian/Ubuntu/WSL:
```bash
sudo apt update && sudo apt install -y curl jq
```

## Supported Desktops & Environments

- **WSL (Windows Subsystem for Linux)**: Automatically detects WSL and sets the host Windows desktop wallpaper via Windows PowerShell interop.
- **GNOME / Budgie / Cinnamon**: via `gsettings`
- **KDE Plasma**: via `qdbus`
- **XFCE**: via `xfconf-query`
- **Wayland (Sway / Hyprland)**: via `swaybg`
- **Generic X11**: via `feh`

## Usage

```bash
# Set today's APOD
./nasa-wallpaper.sh apod

# Set APOD for a specific date
./nasa-wallpaper.sh apod -d 2024-07-20

# Search NASA Image and Video Library
./nasa-wallpaper.sh nasa_image -q "james webb"
```
