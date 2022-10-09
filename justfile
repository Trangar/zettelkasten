default:
    just --list

check:
    cargo check --features front-web,data-sqlite,runtime-async-std

run_terminal:
    cargo run --features front-terminal,data-sqlite,runtime-async-std
    