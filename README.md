# Kindler
Simple viewer of Kindle books written in Rust. Potentially will add more features such as book management.
![Screenshot](https://github.com/nickshulhin/kindler/blob/master/screenshots/0.1_home.png)

## How to build:
```bash
  cargo build --release
```
## How to run:
```bash
  kindler/target/release/kindler
```
Once running, Kindler will look up for connected Kindle book (tested on my own Paperwhite) which usually mounts on `/media/{HOME}/Kindle/documents` and tries to read all .MOBI file types recursively. Tested/built on PopOS.
## License
[**The MIT License (MIT)**](https://github.com/nickshulhin/kindler/blob/master/LICENSE)
## Thanks to
[mobi-rs](https://github.com/vv9k/mobi-rs) for mobi file parser.