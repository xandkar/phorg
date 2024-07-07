phorg
===============================================================================

[![test status](https://github.com/xandkar/phorg/actions/workflows/test.yml/badge.svg)](https://github.com/xandkar/phorg/actions)
[![dependencies status](https://deps.rs/repo/github/xandkar/phorg/status.svg)](https://deps.rs/repo/github/xandkar/phorg)

Photo organizer.

Given a `<src>` and `<dst>` directories:

1. finds photo-or-video files in `<src>`
2. fetches their [Exif](https://en.wikipedia.org/wiki/Exif) data
3. computes their hash digests
4. moves-or-copies them into
   `<dst>/<year>/<month>/<day>/<date>--<time>--<digest>[.<extension>]`

Example:

```text
2020/
├── 11
│   ├── 29
│   │   ├── 2020-11-29--15:23:10--crc32:c7d15ddf.heic
│   │   ├── 2020-11-29--15:23:13--crc32:70088557.heic
│   │   ├── 2020-11-29--15:23:14--crc32:ff6fdcb2.heic
│   └── 30
│       ├── 2020-11-30--08:20:00--crc32:08a5aa4a.heic
│       ├── 2020-11-30--08:23:41--crc32:bba07552.heic
│       ├── 2020-11-30--08:24:24--crc32:94c0f155.heic
```
