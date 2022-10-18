# zettelkasten
Rust implementation of https://en.wikipedia.org/wiki/Zettelkasten

## Trangar ideas

- Able to self-host this.
- Multiple users but registering new accounts can be disabled
- Web interface (maybe more)
  - hotkey based
    - `N` new zettel
    - `E` edit current zettel
    - `S` search in all zettels
    - `F` highlight all links with unique 1-2 character codes. If you type those codes you'll follow the links
    - `?` popup hotkeys
  - Edit zettel is a split zettel
    - Left is a text input
    - Right is a markdown preview
  - Links in zettels `[Zettel]` will auto-link to other zettels, if they don't exist a red link will be shown
  - Going 1 zettel back will actually go a zettel back (properly push browser history)
  - Ability to upload images with drag+drop or ctrl+V
- Database backed (postgres/sqlite)
