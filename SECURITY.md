# Security Policy

## Segnalare una vulnerabilità

Non aprire una issue pubblica. Scrivi a **maule2703@gmail.com** con descrizione, passi di riproduzione e impatto stimato. Non includere segreti o dati personali nella segnalazione.

## Versioni supportate

Solo l'ultima versione pubblicata riceve correzioni di sicurezza. Versione corrente: `2.1.3`.

## Modello di sicurezza

- **Validazione URL**: solo HTTPS e host autorizzati (`assets.science.nasa.gov`, `science.nasa.gov`, `images-assets.nasa.gov`, `source.unsplash.com`) possono essere usati come sorgente immagine.
- **Codifica output**: i parametri di ricerca sono codificati con `Url::query_pairs_mut`, evitando concatenazione e injection.
- **Timeout**: ogni richiesta HTTP ha un limite di 20 secondi.
- **Gestione errori**: nessun `unwrap`/`expect` su dati esterni; gli errori sono propagati e mostrati senza stack trace.
- **Nessun segreto**: l'opzione `--key` è deprecata e ignorata; nessuna chiave viene letta o registrata.
- **Catena di fornitura**: `Cargo.lock` è committato; le GitHub Action sono pinnate a SHA-256; dipendenze e workflow sono aggiornati tramite Dependabot.

## Ambito

L'applicazione scarica immagini e modifica lo sfondo del desktop: non espone servizi di rete in ascolto, non gestisce sessioni utente e non memorizza credenziali. Le raccomandazioni OWASP Top 10 2025 applicabili (A03 supply chain, A05 injection, A06 insecure design, A10 exceptional conditions) sono implementate nel codice; le sezioni relative a webapp, LLM e database non sono applicabili.
