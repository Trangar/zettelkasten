default:
    just --list

check_terminal:
    cargo check --features front-terminal,data-sqlite,runtime-async-std

run_terminal:
    cargo run --features front-terminal,data-sqlite,runtime-async-std

fmt:
    cargo fmt --all
