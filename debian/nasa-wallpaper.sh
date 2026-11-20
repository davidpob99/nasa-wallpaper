#!/usr/bin/env bash
# NASA Wallpaper Setter for Linux & WSL
# Modern, POSIX-compatible helper with multi-desktop & WSL support

set -euo pipefail

# Configuration
API_KEY="${NASA_API_KEY:-DEMO_KEY}"
CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/nasa-wallpaper"
TEMP_DIR="${TMPDIR:-/tmp}/nasa-wallpaper"

# Terminal Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

mkdir -p "$CACHE_DIR" "$TEMP_DIR"

log_info() {
    printf "${BLUE}[INFO]${NC} %s\n" "$1"
}

log_success() {
    printf "${GREEN}[SUCCESS]${NC} %s\n" "$1"
}

log_warning() {
    printf "${YELLOW}[WARNING]${NC} %s\n" "$1"
}

log_error() {
    printf "${RED}[ERROR]${NC} %s\n" "$1" >&2
}

check_dependencies() {
    local missing=()
    for cmd in curl jq; do
        if ! command -v "$cmd" &>/dev/null; then
            missing+=("$cmd")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing required dependencies: ${missing[*]}"
        log_info "Install with: sudo apt install -y ${missing[*]}"
        exit 1
    fi
}

get_today_est() {
    TZ='America/New_York' date +%Y-%m-%d
}

set_apod() {
    local date="${1:-$(get_today_est)}"
    local use_hd="${2:-true}"

    log_info "Fetching APOD for $date..."

    local url="https://science.nasa.gov/wp-json/wp/v2/apod-basic/$(date -d "$date" +%y%m%d 2>/dev/null || date -jf "%Y-%m-%d" "$date" +%y%m%d)"
    local response
    response=$(curl -sL --max-time 15 "$url" || true)

    if [[ -z "$response" || "$response" == *"code"* ]]; then
        url="https://api.nasa.gov/planetary/apod?api_key=$API_KEY&date=$date"
        response=$(curl -sL --max-time 15 "$url")
    fi

    if [[ -z "$response" ]]; then
        log_error "Failed to fetch APOD data"
        return 1
    fi

    local media_type title explanation image_url
    media_type=$(printf '%s' "$response" | jq -r 'if type=="array" then .[0].media_type else .media_type // empty end')
    title=$(printf '%s' "$response" | jq -r 'if type=="array" then .[0].title else .title // empty end')
    explanation=$(printf '%s' "$response" | jq -r 'if type=="array" then .[0].explanation else .explanation // empty end')

    if [[ "$media_type" != "image" && -n "$media_type" ]]; then
        log_warning "APOD for $date is not an image (type: $media_type)"
        local permalink
        permalink=$(printf '%s' "$response" | jq -r 'if type=="array" then .[0].permalink else .url // empty end')
        log_info "View online at: $permalink"
        return 0
    fi

    if [[ "$use_hd" = "true" ]]; then
        image_url=$(printf '%s' "$response" | jq -r 'if type=="array" then .[0].hdurl // .[0].url else .hdurl // .url end')
    else
        image_url=$(printf '%s' "$response" | jq -r 'if type=="array" then .[0].url else .url end')
    fi

    printf "${BLUE}Title:${NC} %s\n" "$title"
    printf "${BLUE}Date:${NC} %s\n" "$date"
    printf "${BLUE}Explanation:${NC} %s\n" "$explanation"

    set_wallpaper_from_url "$image_url" "apod-$date"
}

set_nasa_image() {
    local query="${1:-}"
    local center="${2:-}"
    local year_start="${3:-1900}"
    local year_end="${4:-$(date +%Y)}"

    log_info "Searching NASA Image Library..."

    local url="https://images-api.nasa.gov/search?media_type=image"
    if [[ -n "$query" ]]; then
        local enc_query
        enc_query=$(printf '%s' "$query" | jq -sRr @uri)
        url="$url&q=$enc_query"
    fi
    if [[ -n "$center" ]]; then
        url="$url&center=$center"
    fi
    url="$url&year_start=$year_start&year_end=$year_end"

    local response total_hits
    response=$(curl -sL --max-time 15 "$url")
    total_hits=$(printf '%s' "$response" | jq -r '.collection.metadata.total_hits // 0')

    if [[ "$total_hits" -le 0 ]]; then
        log_error "No images found for query: $query"
        return 1
    fi

    log_info "Found $total_hits results"

    local max_page=$((total_hits / 100))
    [[ $max_page -gt 100 ]] && max_page=100
    local random_page=$((RANDOM % (max_page + 1) + 1))

    local page_response
    page_response=$(curl -sL --max-time 15 "$url&page=$random_page")

    local items_count random_index
    items_count=$(printf '%s' "$page_response" | jq '.collection.items | length')
    if [[ "$items_count" -le 0 ]]; then
        log_error "No items retrieved from page $random_page"
        return 1
    fi
    random_index=$((RANDOM % items_count))

    local item title description nasa_id collection_url
    item=$(printf '%s' "$page_response" | jq ".collection.items[$random_index]")
    title=$(printf '%s' "$item" | jq -r '.data[0].title // "NASA Image"')
    description=$(printf '%s' "$item" | jq -r '.data[0].description // ""')
    nasa_id=$(printf '%s' "$item" | jq -r '.data[0].nasa_id // "nasa"')
    collection_url=$(printf '%s' "$item" | jq -r '.href')

    local collection_data image_url
    collection_data=$(curl -sL --max-time 15 "$collection_url")
    image_url=$(printf '%s' "$collection_data" | jq -r '.[0] // empty')

    if [[ -z "$image_url" ]]; then
        log_error "Failed to retrieve direct image URL"
        return 1
    fi

    printf "${BLUE}Title:${NC} %s\n" "$title"
    printf "${BLUE}NASA ID:${NC} %s\n" "$nasa_id"
    printf "${BLUE}Description:${NC} %s\n" "$description"

    set_wallpaper_from_url "$image_url" "nasa-$nasa_id"
}

set_wallpaper_from_url() {
    local url="$1"
    local name="$2"

    log_info "Downloading wallpaper..."

    local ext="jpg"
    [[ "$url" =~ \.png$ ]] && ext="png"

    local filename="$TEMP_DIR/${name}-$(date +%s).${ext}"

    if ! curl -sL --max-time 30 -o "$filename" "$url"; then
        log_error "Failed to download image from $url"
        return 1
    fi

    log_info "Setting wallpaper..."

    # 1. WSL integration: set Windows host desktop background directly
    if grep -qi "microsoft" /proc/version 2>/dev/null && command -v powershell.exe &>/dev/null; then
        local winpath
        winpath=$(wslpath -w "$filename")
        if powershell.exe -NoProfile -Command "Add-Type -TypeDefinition 'using System.Runtime.InteropServices; public class W { [DllImport(\"user32.dll\")] public static extern bool SystemParametersInfo(int uAction, int uParam, string lpvParam, int fuWinIni); }'; [W]::SystemParametersInfo(0x0014, 0, \"$winpath\", 0x01 | 0x02)" &>/dev/null; then
            log_success "Wallpaper set successfully on Windows host via WSL!"
            log_info "Image saved to: $filename"
            return 0
        fi
    fi

    # 2. GNOME / Budgie / Cinnamon
    if [[ -n "${GNOME_DESKTOP_SESSION_ID:-}" ]] || [[ "${XDG_CURRENT_DESKTOP:-}" =~ (GNOME|Budgie|Cinnamon) ]]; then
        gsettings set org.gnome.desktop.background picture-uri "file://$filename" 2>/dev/null || true
        gsettings set org.gnome.desktop.background picture-uri-dark "file://$filename" 2>/dev/null || true
        log_success "Wallpaper set via GNOME settings"
        return 0
    fi

    # 3. KDE Plasma
    if [[ "${XDG_CURRENT_DESKTOP:-}" = "KDE" ]] && command -v qdbus &>/dev/null; then
        qdbus org.kde.plasmashell /PlasmaShell org.kde.PlasmaShell.evaluateScript "
            var allDesktops = desktops();
            for (i=0;i<allDesktops.length;i++) {
                d = allDesktops[i];
                d.wallpaperPlugin = 'org.kde.image';
                d.currentConfigGroup = Array('Wallpaper', 'org.kde.image', 'General');
                d.writeConfig('Image', 'file://$filename');
            }"
        log_success "Wallpaper set via KDE Plasma"
        return 0
    fi

    # 4. XFCE
    if [[ "${XDG_CURRENT_DESKTOP:-}" = "XFCE" ]] && command -v xfconf-query &>/dev/null; then
        xfconf-query -c xfce4-desktop -p /backdrop/screen0/monitor0/workspace0/last-image -s "$filename"
        log_success "Wallpaper set via XFCE"
        return 0
    fi

    # 5. Wayland / Sway / Hyprland
    if command -v swaybg &>/dev/null; then
        pkill swaybg || true
        swaybg -i "$filename" -m fill &
        log_success "Wallpaper set via swaybg"
        return 0
    fi

    if command -v feh &>/dev/null; then
        feh --bg-scale "$filename"
        log_success "Wallpaper set via feh"
        return 0
    fi

    log_warning "Could not detect active desktop environment. Image downloaded to: $filename"
    return 0
}

show_help() {
    cat << EOF
NASA Wallpaper Setter for Linux & WSL

USAGE:
    ./nasa-wallpaper.sh <COMMAND> [OPTIONS]

COMMANDS:
    apod [OPTIONS]              Get Astronomy Picture of the Day
    nasa_image [OPTIONS]        Get random image from NASA Image Library
    help                        Show this help message

APOD OPTIONS:
    -d, --date <DATE>          Date in YYYY-MM-DD format (default: today)
    -l, --low                  Use lower resolution image

NASA IMAGE OPTIONS:
    -q, --query <QUERY>        Search terms
    -c, --center <CENTER>      NASA center (e.g. JPL, GSFC, KSC)
    --year-start <YEAR>        Start year (default: 1900)
    --year-end <YEAR>          End year (default: current year)

EXAMPLES:
    ./nasa-wallpaper.sh apod
    ./nasa-wallpaper.sh apod -d 2024-07-20
    ./nasa-wallpaper.sh nasa_image -q "james webb"
EOF
}

main() {
    check_dependencies
    local cmd="${1:-help}"
    shift || true

    case "$cmd" in
        apod)
            local date="" low="false"
            while [[ $# -gt 0 ]]; do
                case "$1" in
                    -d|--date) date="$2"; shift 2 ;;
                    -l|--low) low="true"; shift ;;
                    *) shift ;;
                esac
            done
            set_apod "$date" "$low"
            ;;
        nasa_image|nasa-image)
            local q="" center="" ystart="1900" yend=""
            while [[ $# -gt 0 ]]; do
                case "$1" in
                    -q|--query) q="$2"; shift 2 ;;
                    -c|--center) center="$2"; shift 2 ;;
                    --year-start) ystart="$2"; shift 2 ;;
                    --year-end) yend="$2"; shift 2 ;;
                    *) shift ;;
                esac
            done
            set_nasa_image "$q" "$center" "$ystart" "$yend"
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            log_error "Unknown command: $cmd"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
