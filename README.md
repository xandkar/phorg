phorg
===============================================================================

[![test status](https://github.com/xandkar/phorg/actions/workflows/test.yml/badge.svg)](https://github.com/xandkar/phorg/actions)
[![dependencies status](https://deps.rs/repo/github/xandkar/phorg/status.svg)](https://deps.rs/repo/github/xandkar/phorg)

Photo organizer.

Given a `<src>` and `<dst>` directories:

1. finds photo files in `<src>`
2. fetches their [Exif](https://en.wikipedia.org/wiki/Exif) data
3. moves-or-copies them into
   `<dst>/<year>/<month>/<day>/<timestamp>-<original-name>.<orignal-extension>`
