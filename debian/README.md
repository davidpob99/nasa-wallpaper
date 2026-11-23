# NASA Wallpaper Helper per Debian / Linux / WSL

Script Bash che scarica e imposta come sfondo immagini NASA sui desktop Linux e su WSL. Non usa chiavi API: legge l'endpoint APOD Basic di `science.nasa.gov`.

## Requisiti

- `bash` 4 o successivo
- `curl`
- `jq`

```bash
sudo apt update && sudo apt install -y curl jq
```

## Ambienti supportati

- **WSL**: rileva l'ambiente e imposta lo sfondo dell'host Windows tramite PowerShell interop.
- **GNOME / Budgie / Cinnamon**: tramite `gsettings`.
- **KDE Plasma**: tramite `qdbus`.
- **XFCE**: tramite `xfconf-query`.
- **Wayland (Sway / Hyprland)**: tramite `swaybg`.
- **X11 generico**: tramite `feh`.

## Uso

```bash
./nasa-wallpaper.sh apod
./nasa-wallpaper.sh apod -d 2024-07-20
./nasa-wallpaper.sh apod --low
./nasa-wallpaper.sh nasa_image -q "james webb" --year-start 2020
./nasa-wallpaper.sh help
```

## Sicurezza

- Tutte le richieste hanno un timeout esplicito.
- I parametri di ricerca sono codificati con `jq @uri`.
- Gli URL delle immagini vengono validati: solo HTTPS e host NASA autorizzati.
- Nessun errore viene soppresso: le richieste fallite terminano con codice non nullo.
- Nessun segreto o chiave API è presente nello script.

Verifica sintassi: `bash -n nasa-wallpaper.sh`.
