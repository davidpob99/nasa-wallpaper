# nasa-wallpaper
![AUR Version](https://img.shields.io/aur/version/nasa-wallpaper)

Change your desktop background with a NASA image. You can set an image from the [APOD (Astronomical Picture of the Day)](https://apod.nasa.gov/apod/), the [NASA Image Library](https://images.nasa.gov/) or the [NASA's account in Unsplash](https://unsplash.com/@nasa).

![example](https://images-assets.nasa.gov/image/iss040e008244/iss040e008244~small.jpg)

The supported desktops are:
- Windows
- macOS
- GNOME
- KDE
- Cinnamon
- Unity
- Budgie
- XFCE
- LXDE
- MATE
- Deepin
- Most Wayland compositors (set only, requires swaybg)
- i3 (set only, requires feh)

## Install
Just download the executable that fits your OS and architecture from the [Releases](https://github.com/davidpob99/nasa-wallpaper/releases) section, open a terminal and run it.

If you are a Arch Linux user, you can download and install it from the [AUR](https://aur.archlinux.org/packages/nasa-wallpaper/).

## Build
You need to have [Rust](https://www.rust-lang.org/) installed, version 2021 or above. As soon as you have it, run `cargo build` and check in the `target` folder.

## Getting started
Set the APOD image as wallpaper: `nasa-wallpaper -a`

Set the APOD image of the 27th March 1999: `nasa-wallpaper -a -d 1999-03-27`

Set a random image from the NASA Image Library: `nasa-wallpaper -n`

Set a random image with the _earth_ keyword: `nasa-wallpaper -n -q earth`

Read the help: `nasa-wallpaper --help` or `nasa-wallpaper -h`

**You can read the complete reference on the [Wiki](https://github.com/davidpob99/nasa-wallpaper/wiki/Command%E2%80%90Line-Help)**
