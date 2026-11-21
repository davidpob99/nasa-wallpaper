#!/usr/bin/env bash
# NASA Wallpaper Setter for Linux & WSL
# Nessun segreto hardcoded, nessun errore soppresso, input validati.

set -euo pipefail

APOD_ENDPOINT='https://science.nasa.gov/wp-json/wp/v2/apod-basic'
SEARCH_ENDPOINT='https://images-api.nasa.gov/search'
CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/nasa-wallpaper"
TEMP_DIR="${TMPDIR:-/tmp}/nasa-wallpaper"
HTTP_TIMEOUT=20
DOWNLOAD_TIMEOUT=30

log_info() {
    local message="$1"
    printf '[INFO] %s\n' "$message"
}
log_success() {
    local message="$1"
    printf '[SUCCESS] %s\n' "$message"
}
log_warning() {
    local message="$1"
    printf '[WARNING] %s\n' "$message" >&2
}
log_error() {
    local message="$1"
    printf '[ERROR] %s\n' "$message" >&2
}

die() {
    local message="$1"
    log_error "$message"
    exit 1
}

check_dependencies() {
    local missing=()
    local cmd
    for cmd in curl jq date; do
        command -v "$cmd" >/dev/null 2>&1 || missing+=("$cmd")
    done
    if [[ ${#missing[@]} -gt 0 ]]; then
        die "Dipendenze mancanti: ${missing[*]}"
    fi
    mkdir -p "$CACHE_DIR" "$TEMP_DIR" || die 'Impossibile creare le directory di lavoro'
}

url_encode() {
    local value="$1"
    printf '%s' "$value" | jq -sRr @uri
}

validate_date() {
    local value="$1"
    [[ "$value" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || return 1
    if date -d "$value" +%Y-%m-%d >/dev/null 2>&1; then
        [[ "$(date -d "$value" +%Y-%m-%d)" == "$value" ]]
    elif date -jf '%Y-%m-%d' "$value" +%Y-%m-%d >/dev/null 2>&1; then
        [[ "$(date -jf '%Y-%m-%d' "$value" +%Y-%m-%d)" == "$value" ]]
    else
        return 1
    fi
}

validate_year() {
    local value="$1"
    [[ "$value" =~ ^[0-9]{4}$ ]] && (( 10#$value >= 1800 && 10#$value <= 2200 ))
}

validate_image_url() {
    local url="$1"
    [[ "$url" == https://* ]] || return 1
    local host
    host=$(printf '%s' "$url" | jq -r 'capture("^https://(?<h>[^/:]+)").h // empty')
    case "$host" in
        assets.science.nasa.gov|science.nasa.gov|images-assets.nasa.gov) return 0 ;;
        *) return 1 ;;
    esac
}

get_today_est() { TZ='America/New_York' date +%Y-%m-%d; }

date_code() {
    local data="$1"
    if date -d "$data" +%y%m%d >/dev/null 2>&1; then
        date -d "$data" +%y%m%d
    else
        date -jf '%Y-%m-%d' "$data" +%y%m%d
    fi
}

http_get() {
    local url="$1"
    curl --fail-with-body --silent --show-error --location \
        --max-time "$HTTP_TIMEOUT" "$url" || return 1
}

set_apod() {
    local data="${1:-}" low="${2:-false}"
    [[ -z "$data" ]] && data=$(get_today_est)
    validate_date "$data" || die "Data non valida: usare YYYY-MM-DD"

    local url
    url="${APOD_ENDPOINT}/$(date_code "$data")"
    log_info "Recupero APOD per $data..."
    local response
    response=$(http_get "$url") || die 'Recupero APOD non riuscito'

    local media_type title explanation image_url
    media_type=$(printf '%s' "$response" | jq -r 'if type=="array" then .[0].media_type else .media_type // empty end')
    title=$(printf '%s' "$response" | jq -r 'if type=="array" then .[0].title else .title // empty end')
    explanation=$(printf '%s' "$response" | jq -r 'if type=="array" then .[0].explanation else .explanation // empty end')

    if [[ -n "$media_type" && "$media_type" != "image" ]]; then
        log_warning "L'APOD di $data non è un'immagine (tipo: $media_type)"
        local link
        link=$(printf '%s' "$response" | jq -r 'if type=="array" then .[0].permalink else .url // empty end')
        log_info "Contenuto originale: $link"
        return 0
    fi

    if [[ "$low" == "true" ]]; then
        image_url=$(printf '%s' "$response" | jq -r 'if type=="array" then .[0].hdurl // .[0].url else .hdurl // .url end')
        if [[ "$image_url" == *"/dynamicimage/"* ]]; then
            image_url="${image_url%%\?*}?w=1024&fit=clip"
        fi
    else
        image_url=$(printf '%s' "$response" | jq -r 'if type=="array" then .[0].hdurl // .[0].url else .hdurl // .url end')
    fi
    [[ -n "$image_url" && "$image_url" != "null" ]] || die 'Nessun URL immagine nell APOD'

    printf '[INFO] Title: %s\n' "$title"
    printf '[INFO] Date: %s\n' "$data"
    printf '[INFO] Explanation: %s\n' "$explanation"
    set_wallpaper_from_url "$image_url" "apod-$data"
}

set_nasa_image() {
    local query="${1:-}" center="${2:-}" ystart="${3:-1900}" yend="${4:-}"
    [[ -z "$yend" ]] && yend=$(date +%Y)
    validate_year "$ystart" || die "Anno inizio non valido: $ystart"
    validate_year "$yend" || die "Anno fine non valido: $yend"

    local url="${SEARCH_ENDPOINT}?media_type=image"
    [[ -n "$query" ]] && url="$url&q=$(url_encode "$query")"
    [[ -n "$center" ]] && url="$url&center=$(url_encode "$center")"
    url="$url&year_start=$ystart&year_end=$yend"

    log_info 'Ricerca nella NASA Image Library...'
    local response total_hits
    response=$(http_get "$url") || die 'Ricerca non riuscita'
    total_hits=$(printf '%s' "$response" | jq -r '.collection.metadata.total_hits // 0')
    (( total_hits > 0 )) || die 'Nessun risultato per i criteri richiesti'

    local pages=$(( (total_hits + 99) / 100 ))
    (( pages > 1 )) || pages=1
    local page=$(( RANDOM % pages ))
    local page_response
    page_response=$(http_get "$url&page=$page") || die 'Recupero pagina non riuscito'

    local items_count
    items_count=$(printf '%s' "$page_response" | jq '.collection.items | length')
    (( items_count > 0 )) || die "Pagina $page vuota"
    local index=$(( RANDOM % items_count ))

    local item title description nasa_id collection_url
    item=$(printf '%s' "$page_response" | jq -c ".collection.items[$index]")
    title=$(printf '%s' "$item" | jq -r '.data[0].title // "NASA Image"')
    description=$(printf '%s' "$item" | jq -r '.data[0].description // ""')
    nasa_id=$(printf '%s' "$item" | jq -r '.data[0].nasa_id // "nasa"')
    collection_url=$(printf '%s' "$item" | jq -r '.href // empty')
    [[ -n "$collection_url" ]] || die 'URL collezione mancante'

    local collection_data image_url
    collection_data=$(http_get "$collection_url") || die 'Recupero collezione non riuscito'
    image_url=$(printf '%s' "$collection_data" | jq -r '.[0] // empty')
    [[ -n "$image_url" ]] || die 'URL immagine mancante'

    printf '[INFO] Title: %s\n' "$title"
    printf '[INFO] NASA ID: %s\n' "$nasa_id"
    printf '[INFO] Description: %s\n' "$description"
    set_wallpaper_from_url "$image_url" "nasa-$nasa_id"
}

set_wallpaper_from_url() {
    local url="$1" name="$2"
    validate_image_url "$url" || die 'URL immagine non valido o host non autorizzato'

    local ext='jpg'
    [[ "$url" == *.png ]] && ext='png'
    local filename
    filename="$TEMP_DIR/${name}-$(date +%s).${ext}"

    log_info 'Download immagine...'
    if ! curl --fail-with-body --silent --show-error --location \
        --max-time "$DOWNLOAD_TIMEOUT" --output "$filename" "$url"; then
        die 'Download immagine non riuscito'
    fi
    [[ -s "$filename" ]] || die 'File immagine vuoto'

    if grep -qi microsoft /proc/version 2>/dev/null && command -v powershell.exe >/dev/null 2>&1; then
        local winpath
        winpath=$(wslpath -w "$filename") || die 'Conversione percorso WSL non riuscita'
        if powershell.exe -NoProfile -Command "Add-Type -TypeDefinition 'using System.Runtime.InteropServices; public class W { [DllImport(\"user32.dll\")] public static extern bool SystemParametersInfo(int uAction, int uParam, string lpvParam, int fuWinIni); }'; [W]::SystemParametersInfo(0x0014, 0, \"$winpath\", 0x01 | 0x02)" >/dev/null 2>&1; then
            log_success "Sfondo impostato sull host Windows tramite WSL: $filename"
            return 0
        fi
        die 'Impostazione sfondo Windows non riuscita'
    fi

    local desktop="${XDG_CURRENT_DESKTOP:-}"
    if [[ -n "${GNOME_DESKTOP_SESSION_ID:-}" || "$desktop" =~ (GNOME|Budgie|Cinnamon) ]]; then
        if ! gsettings set org.gnome.desktop.background picture-uri "file://$filename"; then
            die 'Impostazione sfondo GNOME non riuscita'
        fi
        if ! gsettings set org.gnome.desktop.background picture-uri-dark "file://$filename"; then
            die 'Impostazione sfondo GNOME (tema scuro) non riuscita'
        fi
        log_success 'Sfondo impostato tramite GNOME'
        return 0
    fi

    if [[ "$desktop" == 'KDE' ]] && command -v qdbus >/dev/null 2>&1; then
        qdbus org.kde.plasmashell /PlasmaShell org.kde.PlasmaShell.evaluateScript "
            var allDesktops = desktops();
            for (i = 0; i < allDesktops.length; i++) {
                d = allDesktops[i];
                d.wallpaperPlugin = 'org.kde.image';
                d.currentConfigGroup = Array('Wallpaper', 'org.kde.image', 'General');
                d.writeConfig('Image', 'file://$filename');
            }" || die 'Impostazione sfondo KDE non riuscita'
        log_success 'Sfondo impostato tramite KDE Plasma'
        return 0
    fi

    if [[ "$desktop" == 'XFCE' ]] && command -v xfconf-query >/dev/null 2>&1; then
        xfconf-query -c xfce4-desktop -p /backdrop/screen0/monitor0/workspace0/last-image -s "$filename" \
            || die 'Impostazione sfondo XFCE non riuscita'
        log_success 'Sfondo impostato tramite XFCE'
        return 0
    fi

    if command -v swaybg >/dev/null 2>&1; then
        pkill swaybg || log_warning 'Nessun processo swaybg da terminare'
        swaybg -i "$filename" -m fill || die 'Impostazione sfondo swaybg non riuscita'
        log_success 'Sfondo impostato tramite swaybg'
        return 0
    fi

    if command -v feh >/dev/null 2>&1; then
        feh --bg-scale "$filename" || die 'Impostazione sfondo feh non riuscita'
        log_success 'Sfondo impostato tramite feh'
        return 0
    fi

    die "Desktop non rilevato. Immagine salvata in: $filename"
}

show_help() {
    cat <<'EOF'
NASA Wallpaper Setter for Linux & WSL

USO:
    ./nasa-wallpaper.sh <comando> [opzioni]

COMANDI:
    apod [-d DATA] [-l]     Imposta l'APOD di una data (predefinita: oggi)
    nasa_image [opzioni]    Immagine casuale dalla libreria NASA
    help                    Mostra questo messaggio

OPZIONI APOD:
    -d, --date DATA         Data YYYY-MM-DD
    -l, --low               Immagine a bassa risoluzione

OPZIONI NASA IMAGE:
    -q, --query TESTO       Termini di ricerca
    -c, --center CENTRO     Centro NASA (es. JPL, GSFC)
    --year-start ANNO       Anno inizio (1800-2200)
    --year-end ANNO         Anno fine (1800-2200)
EOF
}

main() {
    check_dependencies
    local cmd="${1:-help}"
    if (( $# > 0 )); then shift; fi

    case "$cmd" in
        apod)
            local data='' low='false'
            while (( $# > 0 )); do
                local option="$1"
                case "$option" in
                    -d|--date) (( $# >= 2 )) || die 'Valore mancante per --date'; data="$2"; shift 2 ;;
                    -l|--low) low='true'; shift ;;
                    *) die "Opzione sconosciuta: $option" ;;
                esac
            done
            set_apod "$data" "$low"
            ;;
        nasa_image|nasa-image)
            local q='' center='' ystart='1900' yend=''
            while (( $# > 0 )); do
                local option="$1"
                case "$option" in
                    -q|--query) (( $# >= 2 )) || die 'Valore mancante per --query'; q="$2"; shift 2 ;;
                    -c|--center) (( $# >= 2 )) || die 'Valore mancante per --center'; center="$2"; shift 2 ;;
                    --year-start) (( $# >= 2 )) || die 'Valore mancante per --year-start'; ystart="$2"; shift 2 ;;
                    --year-end) (( $# >= 2 )) || die 'Valore mancante per --year-end'; yend="$2"; shift 2 ;;
                    *) die "Opzione sconosciuta: $option" ;;
                esac
            done
            set_nasa_image "$q" "$center" "$ystart" "$yend"
            ;;
        help|--help|-h) show_help ;;
        *) die "Comando sconosciuto: $cmd" ;;
    esac
}

main "$@"
