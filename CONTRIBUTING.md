# Contributing

## Regole generali

- Ogni commit e ogni pull request devono contenere **una sola modifica coerente**: un fix, una funzione, una checklist o una configurazione collegata. Non mescolare riscritture, fix CI, aggiornamenti di dipendenze e documentazione non correlata.
- Il titolo della PR deve essere breve e concreto; la descrizione deve spiegare cosa cambia e come è stata verificata.
- Tutti i commit devono essere firmati GPG e attributi a Light Yagami.
- Non includere tracce, watermark o attribuzioni di strumenti di generazione automatica, né righe `Co-authored-by`.
- Non aggiungere segreti o chiavi API al codice, ai test o ai log.

## Prima di aprire una PR

Esegui i controlli seguenti: devono terminare con exit 0, zero errori e zero warning.

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Regole di sicurezza

- Non sopprimere errori o warning: correggi sempre la causa.
- Non usare `unwrap`/`expect` su dati provenienti da rete o dall'utente.
- Ogni URL usato per scaricare immagini deve passare la validazione su HTTPS e host autorizzato.
- Aggiorna `Cargo.lock` quando cambiano le dipendenze; mantieni i pin SHA-256 delle GitHub Action.

Le PR vengono riviste prima del merge.
