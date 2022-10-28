set dotenv-load

test:
    cargo test

test-print:
    cargo test -- --nocapture

build:
    cargo build

release:
    cargo build --release

default: test

clean:
    cargo clean
    rm *.txt
