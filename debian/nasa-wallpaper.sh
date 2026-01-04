#!/bin/bash
# NASA Wallpaper Setter for Debian/Linux
# Simplified version of the Rust implementation

set -e

# Configuration
API_KEY="${NASA_API_KEY:-DEMO_KEY}"
CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/nasa-wallpaper"
TEMP_DIR="/tmp/nasa-wallpaper"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create directories
mkdir -p "$CACHE_DIR" "$TEMP_DIR"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Get today's date in EST
get_today_est() {
    TZ='America/New_York' date +%Y-%m-%d
}

# Download and set APOD
set_apod() {
    local date="${1:-$(get_today_est)}"
    local use_hd="${2:-true}"
    
    log_info "Fetching APOD for $date..."
    
    local url="https://api.nasa.gov/planetary/apod?api_key=$API_KEY&date=$date"
    local response=$(curl -s "$url")
    
    if [ -z "$response" ]; then
        log_error "Failed to fetch APOD data"
        return 1
    fi
    
    # Parse JSON response
    local media_type=$(echo "$response" | jq -r '.media_type')
    local title=$(echo "$response" | jq -r '.title')
    local explanation=$(echo "$response" | jq -r '.explanation')
    
    if [ "$media_type" != "image" ]; then
        log_warning "APOD for $date is not an image (type: $media_type)"
        local url_link=$(echo "$response" | jq -r '.url')
        echo -e "${YELLOW}View it here: $url_link${NC}"
        return 1
    fi
    
    local image_url
    if [ "$use_hd" = "true" ]; then
        image_url=$(echo "$response" | jq -r '.hdurl // .url')
    else
        image_url=$(echo "$response" | jq -r '.url')
    fi
    
    echo -e "${BLUE}Title:${NC} $title"
    echo -e "${BLUE}Date:${NC} $date"
    echo -e "${BLUE}Explanation:${NC} $explanation"
    
    set_wallpaper_from_url "$image_url" "apod-$date"
}

# Search and set NASA Image Library
set_nasa_image() {
    local query="${1:-}"
    local center="${2:-}"
    local year_start="${3:-1900}"
    local year_end="${4:-$(date +%Y)}"
    
    log_info "Searching NASA Image Library..."
    
    local url="https://images-api.nasa.gov/search?media_type=image"
    [ -n "$query" ] && url="$url&q=$(echo "$query" | jq -sRr @uri)"
    [ -n "$center" ] && url="$url&center=$center"
    url="$url&year_start=$year_start&year_end=$year_end"
    
    local response=$(curl -s "$url")
    local total_hits=$(echo "$response" | jq -r '.collection.metadata.total_hits')
    
    if [ "$total_hits" = "0" ] || [ "$total_hits" = "null" ]; then
        log_error "No images found for query: $query"
        return 1
    fi
    
    log_info "Found $total_hits results"
    
    # Get random page and item
    local max_page=$((total_hits / 100))
    [ $max_page -gt 100 ] && max_page=100
    local random_page=$((RANDOM % (max_page + 1) + 1))
    
    local page_url="$url&page=$random_page"
    local page_response=$(curl -s "$page_url")
    
    local items_count=$(echo "$page_response" | jq '.collection.items | length')
    local random_index=$((RANDOM % items_count))
    
    local item=$(echo "$page_response" | jq ".collection.items[$random_index]")
    local title=$(echo "$item" | jq -r '.data[0].title')
    local description=$(echo "$item" | jq -r '.data[0].description')
    local nasa_id=$(echo "$item" | jq -r '.data[0].nasa_id')
    local date_created=$(echo "$item" | jq -r '.data[0].date_created' | cut -d'T' -f1)
    
    echo -e "${BLUE}Title:${NC} $title"
    echo -e "${BLUE}Date:${NC} $date_created"
    echo -e "${BLUE}NASA ID:${NC} $nasa_id"
    echo -e "${BLUE}Description:${NC} $description"
    
    local collection_url=$(echo "$item" | jq -r '.href')
    local collection_response=$(curl -s "$collection_url")
    local image_url=$(echo "$collection_response" | jq -r '.[0]')
    
    set_wallpaper_from_url "$image_url" "nasa-$nasa_id"
}

# Set wallpaper from Unsplash
set_unsplash() {
    log_info "Fetching random NASA image from Unsplash..."
    local image_url="https://source.unsplash.com/user/nasa/1920x1080"
    set_wallpaper_from_url "$image_url" "unsplash-nasa"
}

# Download and set wallpaper
set_wallpaper_from_url() {
    local url="$1"
    local name="$2"
    
    log_info "Downloading wallpaper..."
    
    # Determine file extension
    local ext="jpg"
    [[ "$url" =~ \.png$ ]] && ext="png"
    
    local timestamp=$(date +%s)
    local filename="$TEMP_DIR/${name}-${timestamp}.${ext}"
    
    if ! curl -L -o "$filename" "$url" 2>/dev/null; then
        log_error "Failed to download image from $url"
        return 1
    fi
    
    log_info "Setting wallpaper..."
    
    # Detect desktop environment and set wallpaper accordingly
    if [ -n "$GNOME_DESKTOP_SESSION_ID" ] || [ "$XDG_CURRENT_DESKTOP" = "GNOME" ]; then
        gsettings set org.gnome.desktop.background picture-uri "file://$filename"
        gsettings set org.gnome.desktop.background picture-uri-dark "file://$filename"
    elif [ "$XDG_CURRENT_DESKTOP" = "KDE" ]; then
        qdbus org.kde.plasmashell /PlasmaShell org.kde.PlasmaShell.evaluateScript "
            var allDesktops = desktops();
            for (i=0;i<allDesktops.length;i++) {
                d = allDesktops[i];
                d.wallpaperPlugin = 'org.kde.image';
                d.currentConfigGroup = Array('Wallpaper', 'org.kde.image', 'General');
                d.writeConfig('Image', 'file://$filename');
            }"
    elif [ "$XDG_CURRENT_DESKTOP" = "XFCE" ]; then
        xfconf-query -c xfce4-desktop -p /backdrop/screen0/monitor0/workspace0/last-image -s "$filename"
    elif command -v feh &> /dev/null; then
        feh --bg-scale "$filename"
    else
        log_warning "Could not detect desktop environment. Trying generic method..."
        if command -v gsettings &> /dev/null; then
            gsettings set org.gnome.desktop.background picture-uri "file://$filename"
        else
            log_error "No supported wallpaper setter found"
            return 1
        fi
    fi
    
    log_success "Wallpaper set successfully!"
    log_info "Image saved to: $filename"
}

# Show help
show_help() {
    cat << EOF
NASA Wallpaper Setter for Debian/Linux

USAGE:
    $0 <COMMAND> [OPTIONS]

COMMANDS:
    apod [OPTIONS]              Get Astronomy Picture of the Day
    nasa_image [OPTIONS]        Get random image from NASA Image Library
    unsplash                    Get random image from NASA's Unsplash
    help                        Show this help message

APOD OPTIONS:
    -d, --date <DATE>          Date in YYYY-MM-DD format (default: today)
    -l, --low                  Use low resolution image

NASA IMAGE OPTIONS:
    -q, --query <QUERY>        Search query
    -c, --center <CENTER>      NASA center
    --year-start <YEAR>        Start year (default: 1900)
    --year-end <YEAR>          End year (default: current year)

ENVIRONMENT VARIABLES:
    NASA_API_KEY               NASA API key (default: DEMO_KEY)

EXAMPLES:
    $0 apod
    $0 apod --date 2023-12-25
    $0 nasa_image --query "apollo 11"
    $0 unsplash

EOF
}

# Main command parser
main() {
    if [ $# -eq 0 ]; then
        show_help
        exit 1
    fi
    
    local command="$1"
    shift
    
    case "$command" in
        apod)
            local date=""
            local use_hd="true"
            
            while [ $# -gt 0 ]; do
                case "$1" in
                    -d|--date)
                        date="$2"
                        shift 2
                        ;;
                    -l|--low)
                        use_hd="false"
                        shift
                        ;;
                    *)
                        log_error "Unknown option: $1"
                        exit 1
                        ;;
                esac
            done
            
            set_apod "$date" "$use_hd"
            ;;
        
        nasa_image)
            local query=""
            local center=""
            local year_start="1900"
            local year_end=$(date +%Y)
            
            while [ $# -gt 0 ]; do
                case "$1" in
                    -q|--query)
                        query="$2"
                        shift 2
                        ;;
                    -c|--center)
                        center="$2"
                        shift 2
                        ;;
                    --year-start)
                        year_start="$2"
                        shift 2
                        ;;
                    --year-end)
                        year_end="$2"
                        shift 2
                        ;;
                    *)
                        log_error "Unknown option: $1"
                        exit 1
                        ;;
                esac
            done
            
            set_nasa_image "$query" "$center" "$year_start" "$year_end"
            ;;
        
        unsplash)
            set_unsplash
            ;;
        
        help|--help|-h)
            show_help
            ;;
        
        *)
            log_error "Unknown command: $command"
            show_help
            exit 1
            ;;
    esac
}

# Check dependencies
check_dependencies() {
    local missing=()
    
    command -v curl &> /dev/null || missing+=("curl")
    command -v jq &> /dev/null || missing+=("jq")
    
    if [ ${#missing[@]} -gt 0 ]; then
        log_error "Missing required dependencies: ${missing[*]}"
        log_info "Install with: sudo apt install ${missing[*]}"
        exit 1
    fi
}

# Run
check_dependencies
main "$@"
