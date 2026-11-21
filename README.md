# nasa-wallpaper

![GitHub Release](https://img.shields.io/github/v/release/davidpob99/nasa-wallpaper)
![Crates.io Version](https://img.shields.io/crates/v/nasa-wallpaper)
![AUR Version](https://img.shields.io/aur/version/nasa-wallpaper)
![GitHub Downloads](https://img.shields.io/github/downloads/davidpob99/nasa-wallpaper/total)
![License](https://img.shields.io/github/license/davidpob99/nasa-wallpaper)

Strumento leggero che imposta come sfondo del desktop un'immagine NASA. Le sorgenti disponibili sono:

- [APOD (Astronomical Picture of the Day)](https://science.nasa.gov/apod/)
- [NASA Image Library](https://images.nasa.gov/)
- [NASA su Unsplash](https://unsplash.com/@nasa)

![Example](https://images-assets.nasa.gov/image/iss040e008244/iss040e008244~small.jpg)

## Piattaforme supportate

- Windows, macOS
- GNOME, KDE, Cinnamon, Unity, Budgie, XFCE, LXDE, MATE, Deepin
- i3

## Installazione

- **Crates.io**: `cargo install nasa-wallpaper`
- **Manualmente**: scarica l'eseguibile dalla pagina [Releases](https://github.com/davidpob99/nasa-wallpaper/releases) e avvialo da terminale.
- **Arch Linux**: pacchetto [AUR](https://aur.archlinux.org/packages/nasa-wallpaper/).

## Build dal sorgente

Richiede Rust con edizione 2021 o successiva.

```bash
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
```

Il binario viene creato in `target/release/`.

## Uso

```bash
# APOD di oggi
nasa-wallpaper apod

# APOD di una data specifica
nasa-wallpaper apod -d 1999-03-27

# Versione a risoluzione ridotta
nasa-wallpaper apod -l

# Immagine casuale dalla libreria NASA
nasa-wallpaper nasa_image -q "james webb"

# Immagine dall'account NASA su Unsplash
nasa-wallpaper unsplash

# Verifica locale (client HTTP, font, URL)
nasa-wallpaper status

# Testo di licenza
nasa-wallpaper license
```

I dati APOD provengono dall'endpoint [APOD Basic](https://science.nasa.gov/wp-json/wp/v2/apod-basic), che non richiede chiave API. L'opzione legacy `-k/--key` è ancora accettata ma ignorata e non appare nell'help.

Le shorthand storiche `-a` (apod) e `-n` (nasa_image) restano supportate.

## Sicurezza

- Le richieste HTTP hanno un timeout esplicito di 20 secondi.
- I parametri di ricerca sono codificati tramite URL, senza concatenazione manuale.
- Prima di impostare lo sfondo, l'URL viene validato: solo HTTPS e host NASA/Unsplash autorizzati.
- Nessun segreto viene letto, scritto o registrato: la vecchia chiave API non è più necessaria.
- Gli errori vengono gestiti senza `unwrap`/`expect` su dati esterni e senza stack trace.

Per le segnalazioni di sicurezza vedi [SECURITY.md](SECURITY.md).

## Licenza

Apache License 2.0. Vedi [LICENSE](LICENSE) e [NOTICE.md](NOTICE.md).

## Altri documenti

[CONTRIBUTING.md](CONTRIBUTING.md), [ROADMAP.md](ROADMAP.md), [CHANGELOG.md](CHANGELOG.md), [SECURITY.md](SECURITY.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [OBSERVABILITY.md](OBSERVABILITY.md), [CITATION.cff](CITATION.cff), [debian/README.md](debian/README.md).
