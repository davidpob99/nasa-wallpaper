# Observability

The application is a local CLI: it exposes no HTTP server, metrics endpoint or distributed tracing. Observability consists of terminal progress messages and exit codes.

## Progress messages

| Message | Meaning |
| --- | --- |
| `Changing wallpaper...` | Download and wallpaper change started |
| `Done` | Wallpaper changed successfully |
| `Error: ...` | Operation failed; process exits with code 1 |
| `Error: invalid date (...)` | Invalid `--date` input; process exits with code 2 |

Messages about a non-image APOD point to the original content URL and exit with code 0.

## Health check

`nasa-wallpaper status` verifies that the HTTP client can be built, a system font is available and the APOD URL passes the HTTPS/host validation. It exits 0 on success and 1 with a generic error otherwise. No secrets, tokens or personal data are printed.

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | success |
| 1 | network, parsing or wallpaper backend failure |
| 2 | invalid command-line input |

## Local verification

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
