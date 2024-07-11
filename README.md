phorg
===============================================================================

Idempotent photo/video file organizer.

[![test status](https://github.com/xandkar/phorg/actions/workflows/test.yml/badge.svg)](https://github.com/xandkar/phorg/actions)
[![dependencies status](https://deps.rs/repo/github/xandkar/phorg/status.svg)](https://deps.rs/repo/github/xandkar/phorg)

Overview
-------------------------------------------------------------------------------

Given a `<src>` and `<dst>` directories:

1. finds photo/video files in `<src>`
2. fetches their [Exif](https://en.wikipedia.org/wiki/Exif) data
3. computes their hash digests
4. moves/copies them into
   `<dst>/{<img>,<vid>}/<year>/<month>/<day>/<date>--<time>--<digest>[.<ext>]`
   where:
    - `<img>` and `<vid>` default to "img" and "vid", respectively, and are
      customizable via CLI
    - date and time are extracted from Exif metadata, from whichever of the
      following tags is found first, tried in order:
      + `DateTimeOriginal`
      + `DateTimeCreated`
      + `CreateDate`
      + `DateCreated`
      + `Datecreate`
      + `CreationDate`
      + `TrackCreateDate`
5. optionally, you can (manually) add semantically-named subdirectories
   underneath the `<day>` directory and (manually) move the media files into
   them, these subdirectories will then be preserved on subsequent
   reprocessings, i.e. when this `<dst>` is later used as `<src>`

Example
-------------------------------------------------------------------------------

(note the semantic subdirectory on 2020-11-29)

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

Install
-------------------------------------------------------------------------------

0. Ensure a Rust `1.75.0`+ toolchain is installed: <https://www.rust-lang.org/tools/install>
1. `cargo install phorg`
2. Ensure `~/.cargo/bin/` is in your `PATH`
3. `phorg help`
