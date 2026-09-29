# nasa-wallpaper
![GitHub Release](https://img.shields.io/github/v/release/davidpob99/nasa-wallpaper)
![Crates.io Version](https://img.shields.io/crates/v/nasa-wallpaper)
![AUR Version](https://img.shields.io/aur/version/nasa-wallpaper)
![GitHub Downloads](https://img.shields.io/github/downloads/davidpob99/nasa-wallpaper/total)
![License](https://img.shields.io/github/license/davidpob99/nasa-wallpaper)

A lightweight tool to automatically set your desktop wallpaper with stunning NASA images.
You can choose images from:

* [APOD (Astronomical Picture of the Day)](https://science.nasa.gov/apod/)
* [NASA Image Library](https://images.nasa.gov/)
* [NASA on Unsplash](https://unsplash.com/@nasa)

![Example](https://images-assets.nasa.gov/image/iss040e008244/iss040e008244~small.jpg)

---

## 🌍 Supported Platforms

* **Windows**
* **macOS**
* **GNOME**, **KDE**, **Cinnamon**, **Unity**, **Budgie**, **XFCE**, **LXDE**, **MATE**, **Deepin**
* **i3**

---

## ⚡ Installation
- **Crates**: `cargo install nasa-wallpaper`
- **Manual**: download the executable that matches your operating system and architecture from the [Releases](https://github.com/davidpob99/nasa-wallpaper/releases) page or downloa d
Open a terminal and simply run it.
- **Arch Linux**: you can install it directly from the [AUR](https://aur.archlinux.org/packages/nasa-wallpaper/).

---

## 🔧 Build from Source

Requirements: [Rust](https://www.rust-lang.org/) **2021 edition or newer**.

Clone the repository and run:

```bash
cargo build --release
```

The compiled binary will be available under `target/release/`.

---

## 🚀 Quick Usage

* Set today’s APOD as wallpaper:

  ```bash
  nasa-wallpaper apod
  ```

* Set the APOD for **March 27, 1999**:

  ```bash
  nasa-wallpaper apod -d 1999-03-27
  ```

* Set a random image from the NASA Image Library:

  ```bash
  nasa-wallpaper nasa_image
  ```

* Set a random image with the keyword *earth*:

  ```bash
  nasa-wallpaper nasa_image -q earth
  ```

* Show help:

  ```bash
  nasa-wallpaper help
  ```

📖 **Full documentation:** [Wiki – Command Line Help](https://github.com/davidpob99/nasa-wallpaper/wiki/Command%E2%80%90Line-Help)

---

## 🤝 Contributing

Contributions are welcome! 🎉

This project follows the [Gitflow](https://nvie.com/posts/a-successful-git-branching-model/) branching model:

| Branch                | Purpose                                                   | Branches off | Merges into          |
|-----------------------|-----------------------------------------------------------|--------------|----------------------|
| `master`              | Production-ready code. Every commit is a tagged release   | —            | —                    |
| `develop`             | Integration branch for the next release                   | `master`     | —                    |
| `feature/<issue>-<short-description>` | New features                              | `develop`    | `develop`            |
| `release/<version>`   | Release preparation (version bump, final fixes)           | `develop`    | `master` and `develop` |
| `hotfix/<issue>-<short-description>`  | Urgent fixes for production               | `master`     | `master` and `develop` |

If you’d like to contribute:

1. Fork this repository
2. Create an issue describing the feature or bug
3. Create a branch that corresponds to the issue, following the naming conventions above:
   - Features: branch off `develop` as `feature/[issue number]-[short description]` (e.g. `feature/37-automatise-upload-of-versions`)
   - Hotfixes: branch off `master` as `hotfix/[issue number]-[short description]`
4. Commit your changes
5. Push the branch
6. Open a Pull Request against `develop` (features) or `master` (hotfixes). Please ensure that an issue exists before submitting your contribution as a pull request

> `master` and `develop` are never committed to directly. `release/*` branches are created by the maintainers.

---

## 📜 License

This project is licensed under the [APACHE 2.0 License](LICENSE).
