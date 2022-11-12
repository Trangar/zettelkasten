default:
    just --list

docs_terminal:
    cargo doc --features front-terminal,data-sqlite,runtime-async-std --open

check_terminal:
    cargo check --features front-terminal,data-sqlite,runtime-async-std

run_terminal:
    cargo run --features front-terminal,data-sqlite,runtime-async-std

install_terminal:
    cargo install --path . --features front-terminal,data-sqlite,runtime-async-std

prepare_sqlite:
    cd data/sqlite && cargo sqlx prepare -- --features runtime-async-std

prepare_postgres:
    cd data/postgres && cargo sqlx prepare -- --features runtime-async-std

lint: fmt clippy

clippy:
    cargo clippy --features front-terminal,data-sqlite,runtime-async-std -- -D warnings

fmt:
    cargo fmt --all

test: test_async_std

test_async_std:
    cargo test --features front-terminal,data-sqlite,runtime-async-std --workspace
