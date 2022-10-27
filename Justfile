set dotenv-load

test:
    cargo test

test-print:
    cargo test -- --nocapture

build:
    cargo build

default: test

clean:
    cargo clean
