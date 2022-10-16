default:
    just --list

check_web:
    cargo check --features front-web,data-sqlite,runtime-async-std

check_terminal:
    cargo check --features front-terminal,data-sqlite,runtime-async-std

run_terminal:
    cargo run --features front-terminal,data-sqlite,runtime-async-std
    