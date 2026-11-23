# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added

- `status` subcommand for a local health check of HTTP client, font and image URL configuration.
- Explicit HTTPS and host allow-list validation before setting any wallpaper.
- Query parameters encoded via `Url::query_pairs_mut` instead of string concatenation.
- Explicit 20-second HTTP timeout on all requests.

### Changed

- All external-data paths return `Result` instead of panicking on network, parsing or shape errors.
- Typed `NasaImageQuery` replaces the long argument list.
- APOD API error messages preserve the machine code and human message without exposing internal details.

### Fixed

- Page-index underflow and out-of-bounds random index in NASA Image Library search.
- Invalid font paths in `NASA_WALLPAPER_FONT` are now reported instead of silently ignored.
- GitHub Actions pinned consistently to SHA-256 and Dependabot covers both Cargo and Actions.

## [1.0.0] - 2026-10-05

### Added

- Initial version with APOD, NASA Image Library and Unsplash sources.
