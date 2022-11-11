default:
    just --list

lint: fmt clippy

clippy:
    cargo clippy --features front-terminal,front-web,data-sqlite,runtime-async-std -- -D warnings

fmt:
    cargo fmt --all

test: test_terminal test_web_sqlite



docs_terminal:
    cargo doc --features front-terminal,data-sqlite,runtime-async-std --open

check_terminal:
    cargo check --features front-terminal,data-sqlite,runtime-async-std

run_terminal:
    cargo run --features front-terminal,data-sqlite,runtime-async-std

test_terminal:
    cargo test --features front-terminal,data-sqlite,runtime-async-std --workspace


docs_web_sqlite:
    cargo doc --features front-web,data-sqlite,runtime-async-std --open

check_web_sqlite:
    cargo check --features front-web,data-sqlite,runtime-async-std

run_web_sqlite:
    cargo run --features front-web,data-sqlite,runtime-async-std

test_web_sqlite:
    cargo test --features front-web,data-sqlite,runtime-async-std



install_terminal:
    cargo install --path . --features front-terminal,data-sqlite,runtime-async-std

prepare_sqlite:
    cd data/sqlite && cargo sqlx prepare -- --features runtime-async-std

