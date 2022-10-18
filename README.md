# zettelkasten
Rust implementation of https://en.wikipedia.org/wiki/Zettelkasten

## Configuration

Zettelkasten is build up out of the following modules:
- `runtime`, what runtime is being used?
- `data`, how is the data stored?
- `front`, one or multiple frontends

The following confirmations are available:

|`runtime`  |`data`  |`front`   |
|-----------|--------|----------|
|`async-std`|`sqlite`|`terminal`|
|           |        |`web`     |

Note that these modules can be mixed and matched in any way you want.

To run a particular combination, pass the set you want as feature flags. e.g.:
```sh
# runtime: async-std
# data: sqlite
# front: terminal
cargo run --features runtime-async-std,data-sqlite,front-terminal
```

Additionally you can use [just](https://github.com/casey/just):
```sh
cargo install just
```

And then run one of:

|`command`          |`runtime`  |`data`  |`front`   |
|-------------------|-----------|--------|----------|
|`just run_terminal`|`async-std`|`sqlite`|`terminal`|

## Setup

Some configs require custom setup instructions

### `data-sqlite`

To use a sqlite database you need to:
- add `DATABASE_URL=sqlite://<path>` to `.env`
  - example: `DATABASE_URL=sqlite://database.db`
- either:
  - with [sqlx-cli](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#sqlx-cli):
    - install `cargo install sqlx-cli --no-default-features --features sqlite,rustls`
    - run `sqlx database setup --source data/sqlite/migrations`
  - manually:
    - create a database file and run all the queries in `data/sqlite/migrations/*.up.sql`
