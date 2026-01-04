# NASA Wallpaper for Debian/Linux

This directory contains a Bash implementation of nasa-wallpaper specifically optimized for Debian and other Linux distributions.

## Features

- ✨ **APOD Support**: Get Astronomy Picture of the Day
- 🔍 **NASA Image Library**: Search and download from NASA's vast image archive
- 📸 **Unsplash Integration**: Random high-quality NASA images
- 🎨 **Multi-DE Support**: Works with GNOME, KDE, XFCE, and more
- 🚀 **Lightweight**: Pure Bash with minimal dependencies

## Requirements

```bash
sudo apt install curl jq
```

For specific desktop environments:
- **GNOME**: `gsettings` (usually pre-installed)
- **KDE**: `qdbus` (usually pre-installed)
- **XFCE**: `xfconf-query` (usually pre-installed)
- **Others**: `feh` as fallback

## Installation

```bash
# Make the script executable
chmod +x nasa-wallpaper.sh

# Optional: Move to system path
sudo cp nasa-wallpaper.sh /usr/local/bin/nasa-wallpaper
```

## Usage

### APOD (Astronomy Picture of the Day)

```bash
# Get today's APOD
./nasa-wallpaper.sh apod

# Get APOD for a specific date
./nasa-wallpaper.sh apod --date 2023-12-25

# Use low resolution
./nasa-wallpaper.sh apod --low
```

### NASA Image Library

```bash
# Random NASA image
./nasa-wallpaper.sh nasa_image

# Search for specific topic
./nasa-wallpaper.sh nasa_image --query "apollo 11"

# Filter by NASA center
./nasa-wallpaper.sh nasa_image --query "mars" --center "JPL"

# Filter by year range
./nasa-wallpaper.sh nasa_image --query "hubble" --year-start 2020 --year-end 2024
```

### Unsplash

```bash
# Random NASA image from Unsplash
./nasa-wallpaper.sh unsplash
```

## Environment Variables

```bash
# Set your NASA API key (optional, defaults to DEMO_KEY)
export NASA_API_KEY="your_api_key_here"
```

Get your free API key at: https://api.nasa.gov/

## Automation

### Daily APOD with Cron

```bash
# Edit crontab
crontab -e

# Add this line to set APOD daily at 9 AM
0 9 * * * /usr/local/bin/nasa-wallpaper apod
```

### Systemd Timer

Create `/etc/systemd/user/nasa-wallpaper.service`:

```ini
[Unit]
Description=NASA Wallpaper Service

[Service]
Type=oneshot
ExecStart=/usr/local/bin/nasa-wallpaper apod
```

Create `/etc/systemd/user/nasa-wallpaper.timer`:

```ini
[Unit]
Description=Daily NASA Wallpaper

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```

Enable the timer:

```bash
systemctl --user enable nasa-wallpaper.timer
systemctl --user start nasa-wallpaper.timer
```

## Supported Desktop Environments

- GNOME (3.x and 40+)
- KDE Plasma
- XFCE
- Cinnamon
- MATE
- Any DE with `feh` support

## Troubleshooting

### Wallpaper not changing

1. Check if dependencies are installed:
   ```bash
   command -v curl && command -v jq && echo "Dependencies OK"
   ```

2. Verify desktop environment detection:
   ```bash
   echo $XDG_CURRENT_DESKTOP
   ```

3. Try manual wallpaper setting:
   ```bash
   # For GNOME
   gsettings set org.gnome.desktop.background picture-uri "file:///path/to/image.jpg"
   ```

### API rate limiting

If using DEMO_KEY, you're limited to 30 requests per hour. Get a free API key at https://api.nasa.gov/ for higher limits.

## Comparison with Rust Version

| Feature | Bash (Debian) | Rust (Cross-platform) |
|---------|---------------|----------------------|
| Dependencies | curl, jq | None (statically linked) |
| Performance | Good | Excellent |
| Binary Size | ~500 bytes | ~5-10 MB |
| Startup Time | ~50ms | ~10ms |
| Portability | Linux only | Windows, macOS, Linux |
| Ease of Modification | Very Easy | Requires Rust knowledge |

## License

Apache 2.0 - Same as the main nasa-wallpaper project
