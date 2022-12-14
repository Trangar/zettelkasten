# zettelkasten
Rust implementation of https://en.wikipedia.org/wiki/Zettelkasten

## Configuration

Zettelkasten is build up out of the following modules:
- `runtime`, what runtime is being used?
- `data`, how is the data stored?
- `front`, one or multiple frontends

The following confirmations are available:

|`runtime`  |`data`    |`front`   |
|-----------|----------|----------|
|`async-std`|`sqlite`  |`terminal`|
|           |`postgres`|          |

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

We highly recommend installing [`sqlx-cli`](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli):
```bash
cargo install sqlx-cli --no-default-features --features rustls,sqlite,postgres
```

Some configs require custom setup instructions

### `data-sqlite`

`data-sqlite` will look for, or create, a database in the following places:
- At the location of one of the following environment variables:
  - `DATABASE_URL`
  - `ZETTELKASTEN_DATABASE_URL`
  - Note that if there is a `.env` file present, this will be loaded
- `<DATA_DIR>/zettelkasten/database.db` where `<DATA_DIR>` is the value from [dirs::data_dir](https://docs.rs/dirs/latest/dirs/fn.data_dir.html)
  - Linux: `$XDG_DATA_HOME` or `$HOME/.local/share`
  - Windows: `C:/Users/<user>/AppData/Roaming/`
- The local directory, if the above fail (`./database.db`)

For local development where you also have a running zettelkasten system, we **highly** recommend setting `DATABASE_URL` to a temp database while working on this project.

To set a temp database you need to:
- add `DATABASE_URL=sqlite://<path>` to `.env`
  - example: `DATABASE_URL=sqlite://database.db`
- either:
  - with [sqlx-cli](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#sqlx-cli):
    - install `cargo install sqlx-cli --no-default-features --features sqlite,rustls`
    - run `sqlx database setup --source data/sqlite/migrations`
  - manually:
    - create a database file and run all the queries in `data/sqlite/migrations/*.up.sql`

### `data-postgres`

`data-postgres` will look for a database connection string in one of the following environment variables:
- `DATABASE_URL`
- `ZETTELKASTEN_DATABASE_URL`

If such an environment variable is not set, postgres will not be able to run.

For local development where you also have a running zettelkasten system, we **highly** recommend setting `DATABASE_URL` to a temp database while working on this project.

To set a temp database you need to:
- add `DATABASE_URL=postgres://<path>` to `.env`
- either:
  - with [sqlx-cli](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#sqlx-cli):
    - install `cargo install sqlx-cli --no-default-features --features postgres,rustls`
    - run `sqlx database setup --source data/postgres/migrations`
  - manually:
    - create a database file and run all the queries in `data/postgres/migrations/*.up.sql`

## Contributing

### Updating schema

When a change in the database schema is made, please follow the following steps to make sure the `sqlx-data.json` is properly updated:
- sqlite:
  - make sure an `.env` and updated `database.db` exist in the `data/sqlite` folder
  - run either:
    - `just prepare_sqlite` from this folder (not `data/sqlite`)
    - `cargo sqlx prepare -- --features runtime-async-std` from the `data/sqlite` folder
  - check in `data/sqlite/sqlx-data.json`
