# nasa-wallpaper
Change your desktop background with a NASA image. You can set an image from both the APOD (Astronomical Picture of the Day) and the 
NASA Image Library.

![iss040e008244](https://images-assets.nasa.gov/image/iss040e008244/iss040e008244~small.jpg)

# Installation
## Debian and derivates: 
See [releases](https://github.com/davidpob99/nasa-wallpaper/releases) and download the `nasa-wallpaper_1.0_all.deb` file

`$ sudo dpkg -i nasa-wallpaper_1.0_all.deb`

## Arch Linux
Coming soon in the AUR!

## With the code
`git clone https://github.com/davidpob99/nasa-wallpaper`

`cd nasa-wallpaper`

`chmod -x ./nasa-wallpaper`

`./nasa-wallpaper`

# Getting started
Set the APOD image as wallpaper: `nasa-wallpaper -a`

Set the APOD image of the 27th March 1999: `nasa-wallpaper -d 1999-03-27 -a`

Set a random image from the NASA Image Library: `nasa-wallpaper -n`

Set a random image with the `earth` keyword: `nasa-wallpaper -w earth -n`

Read the manual: `man nasa-wallpaper` or `nasa-wallpaper -h`

**You can read the all reference on the [Wiki](https://github.com/davidpob99/nasa-wallpaper/wiki) section**

# License

Code available under the Apache 2.0 License.
