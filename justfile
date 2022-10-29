default:
    just --list

check_terminal:
    cargo check --features front-terminal,data-sqlite,runtime-async-std

run_terminal:
    cargo run --features front-terminal,data-sqlite,runtime-async-std

lint: fmt clippy

clippy:
    cargo clippy --features front-terminal,data-sqlite,runtime-async-std -- -D warnings

fmt:
    cargo fmt --all
