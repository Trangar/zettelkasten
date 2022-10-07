# zettelkasten
Rust implementation of https://en.wikipedia.org/wiki/Zettelkasten

## Trangar ideas

- Able to self-host this.
- Multiple users but registering new accounts can be disabled
- Web interface (maybe more)
  - hotkey based
    - `N` new page
    - `E` edit current page
    - `S` search in all pages
    - `F` highlight all links with unique 1-2 character codes. If you type those codes you'll follow the links
    - `?` popup hotkeys
  - Edit page is a split page
    - Left is a text input
    - Right is a markdown preview
  - Links in pages `[Page]` will auto-link to other pages, if they don't exist a red link will be shown
  - Going 1 page back will actually go a page back (properly push browser history)
- Database backed (postgres/sqlite)
