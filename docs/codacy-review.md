# Revisione finding Codacy — nasa-wallpaper

Data: 2026-11-22. Fonte: API Codacy v3, `gh/LightYagami28/nasa-wallpaper`.
Metodo: `POST .../issues/overview` e `POST .../issues/search` con cursori, livelli Error/High/Warning/Info.

## Conteggi

| Livello | Totale |
| --- | ---: |
| High | 3 |
| Warning | 5 |
| Error | 3 |
| Info | 70 |
| **Totale** | **81** |
| belowThreshold | 81 |
## Falsi positivi accertati

- **`Semgrep_yaml.semgrep.duplicate-id.duplicate-id` (3)** — `.gitlab-ci.yml:18-20`: le righe segnalate sono `rules:` e voci `- if: ...`. Il rule interpreta chiavi YAML ripetute come campi `id`; il file non contiene alcun campo `id`.
- **`Lizard_file-nloc-critical`** — `Cargo.lock:1`: file generato da Cargo (2720 righe), non codice sorgente manutenuto.
- **`markdownlint_MD043` (9)** — il messaggio e' `Expected: [None]; Actual: # ...`: la configurazione della regola richiede documenti senza titoli. Ogni file Markdown con un titolo viene segnalato. Blocco di configurazione, non correggibile nel codice senza rimuovere tutti i titoli.
- **`Agentlinter_clarity_undefined-term` (2)** — regola per file di istruzioni agentiche applicata alla documentazione ordinaria.

## Finding reali

- `markdownlint_MD013` (46), `MD012` (1), `MD024` (1), `MD034` (1): formattazione della documentazione, correzioni meccaniche.
- `Lizard_ccn-minor` (3), `ccn-medium` (2), `nloc-minor` (6): complessita'/lunghezza di funzioni in `src/main.rs`, da trattare con refactor dedicati e `cargo test`.

Nessuna modifica a configurazioni o soglie; nessuna soppressione.

