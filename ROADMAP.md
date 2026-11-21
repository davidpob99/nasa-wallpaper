# Roadmap

## Stato attuale

`nasa-wallpaper` è alla versione 2.1.3. I sottocomandi disponibili sono `apod`, `nasa_image`, `unsplash`, `license` e `status`; le shorthand legacy `-a` e `-n` restano supportate.

Le caratteristiche verificate nel sorgente sono:

- recupero APOD dall'endpoint Basic di NASA, con data opzionale e modalità a bassa risoluzione;
- ricerca nella NASA Image Library con parametri codificati e selezione casuale;
- validazione HTTPS/host prima di impostare lo sfondo;
- timeout HTTP esplicito e gestione centralizzata degli errori senza `unwrap`/`expect` su dati esterni;
- test unitari e test HTTP con server mock locale.

## Obiettivi

- Mantenere zero errori e zero warning in `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test`.
- Aggiornare le dipendenze alle ultime versioni stabili mantenendo `Cargo.lock` con checksum e migrando le API quando richiesto.
- Mantenere tutte le GitHub Action pinnate a SHA-256 e aggiornarle tramite Dependabot.
- Ampliare la copertura dei test ai percorsi di errore della ricerca immagini.

## Prossimi passi

1. Aggiungere test mock per la paginazione e per la selezione dell'indice in `get_nasa_image`.
2. Documentare il formato di output di `nasa_image` e i criteri di validazione degli anni.
3. Verificare e aggiornare gli hash SHA-256 dei pacchetti AUR alle release correnti, sostituendo gli MD5 storici.
4. Estendere il comando `status` con un controllo di raggiungibilità dell'endpoint APOD, senza inviare dati personali.
