phorg
===============================================================================

[![test status](https://github.com/xandkar/phorg/actions/workflows/test.yml/badge.svg)](https://github.com/xandkar/phorg/actions)
[![dependencies status](https://deps.rs/repo/github/xandkar/phorg/status.svg)](https://deps.rs/repo/github/xandkar/phorg)

Idempotent photo/video file organizer.

Given a `<src>` and `<dst>` directories:

1. finds photo-or-video files in `<src>`
2. fetches their [Exif](https://en.wikipedia.org/wiki/Exif) data
3. computes their hash digests
4. moves-or-copies them into
   `<dst>/{<img-dir>,<vid-dir>}/<year>/<month>/<day>/<date>--<time>--<digest>[.<extension>]`
   where `<img-dir>` and `<vid-dir>` default to "img" and "vid", respectively,
   and are customizable via CLI.

Example:

```sh
$ phorg /mnt/usb-drive $dst move
$ cd $dst
$ tree .
.
├── img
│   ├── 2020
│   │   ├── 11
│   │   │   ├── 29
│   │   │   │   └── Hike on Suffern-Bear Mountain Trail
│   │   │   │       ├── 2020-11-29--15:23:10--crc32:c7d15ddf.heic
│   │   │   │       ├── 2020-11-29--15:29:40--crc32:b4f4e4e0.heic
│   │   │   │       ├── 2020-11-29--15:30:07--crc32:3b5aa617.heic
│   │   │   │       └── 2020-11-29--15:38:30--crc32:514c9b0c.heic
│   │   │   └── 30
│   │   │       ├── 2020-11-30--08:20:00--crc32:08a5aa4a.heic
│   │   │       ├── 2020-11-30--08:23:41--crc32:bba07552.heic
│   │   │       ├── 2020-11-30--08:24:24--crc32:94c0f155.heic
```
