# Security Policy

## Segnalare una vulnerabilità

Non aprire una issue pubblica. Scrivi a **@LightYagami28** con descrizione,
passi di riproduzione e impatto stimato. Non includere segreti o dati personali
nella segnalazione.

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

## HTTPS residuo

Tutte le richieste di rete dell'applicazione usano HTTPS e host autorizzati. I `http://` ancora presenti non sono URL di rete attivi:

- `LICENSE` (due righe): testo originale della licenza Apache 2.0, riferimento legale da non alterare.
- `src/tests.rs:298`: indirizzo loopback di test (`127.0.0.1:1`), nessuna richiesta esterna.
- `src/tests.rs:367`: asserzione negativa che verifica il rifiuto di una sorgente immagine `http://`.

## Dipendenze senza fix compatibile

- **`rand 0.7.3`** (RUSTSEC-2026-0097, bassa): presente solo come dipendenza
  transitiva di `imageproc 0.23.1`, che richiede `rand ^0.7.3`. La correzione
  upstream esiste solo nelle serie `0.8.6+`, `0.9.3+` e `0.10.1+`; per
  raggiungerla servirebbe aggiornare `imageproc` (e `image`) e migrare il
  rendering testuale da `rusttype` ad `ab_glyph`, una modifica dedicata non
  eseguibile senza rischio di regressioni visive.

  L'esposizione è nulla: il difetto richiede le feature `log` e `thread_rng`
  e un logger personalizzato che chiami `rand::rng()`. In questo progetto
  `rand 0.7.3` espone solo `alloc`, `std`, `default`, `getrandom`,
  `getrandom_package` e `libc` (verificato con
  `cargo tree -e features -i rand@0.7.3`) e non è definito alcun logger.
  Le altre copie nel lockfile (`0.9.5`, `0.10.4`) sono già patched.
