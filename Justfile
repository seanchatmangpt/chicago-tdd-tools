test:
    cargo make test

build:
    cargo make build

polish:
    cargo make lint && cargo make fmt

clean:
    cargo make clean

test-full:
    cargo make test-all 2>/dev/null || cargo test --all-features

doc:
    cargo doc --no-deps

bench:
    cargo bench

ci: polish test-full
