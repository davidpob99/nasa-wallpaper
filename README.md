# nasa-wallpaper

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Donate](https://img.shields.io/badge/Donate-PayPal-green.svg)](https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=GRXHT9CGJ4L7G)
[![Website](https://img.shields.io/website-up-down-green-red/http/shields.io.svg?label=my-website)](https://davidpob99.github.io/PolenCyL/)


Change your desktop background with a NASA image. You can set an image from both the APOD (Astronomical Picture of the Day) and the 
NASA Image Library.

![iss040e008244](https://images-assets.nasa.gov/image/iss040e008244/iss040e008244~small.jpg)

# Installation
## Debian and derivatives

1. `sudo apt-get install jq curl git wget`
2. `git clone https://github.com/davidpob99/nasa-wallpaper`
3. `cd nasa-wallpaper`
4. `chmod -x INSTALL`
5. `sudo sh INSTALL`

## Arch Linux
[AUR](https://aur.archlinux.org/packages/nasa-wallpaper/)

## With the code
`git clone https://github.com/davidpob99/nasa-wallpaper`

`cd nasa-wallpaper`

`chmod -x ./nasa-wallpaper`

`./nasa-wallpaper`

# Getting started
Set the APOD image as wallpaper (GNOME): `nasa-wallpaper -T gnome -a`

Set the APOD image of the 27th March 1999 (MATE): `nasa-wallpaper -d 1999-03-27 -T mate -a`

Set a random image from the NASA Image Library (LXDE): `nasa-wallpaper -T lxde -n`

Set a random image with the `earth` keyword (GNOME): `nasa-wallpaper -w earth -T gnome -n`

Read the manual: `man nasa-wallpaper` or `nasa-wallpaper -h`

**You can read the all reference on the [Wiki](https://github.com/davidpob99/nasa-wallpaper/wiki/Reference) section**

# Uninstall
## Debian and derivatives

1. `git clone https://github.com/davidpob99/nasa-wallpaper`
2. `cd nasa-wallpaper`
3. `chmod -x UNINSTALL`
4. `sudo sh UNINSTALL`

## Arch Linux

`sudo pacman -Rs nasa-wallpaper`

# License

Code available under the Apache 2.0 License.
