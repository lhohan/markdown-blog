build:
    cargo build --release --bin tt

# install:

# Check for warnings continuously
check-w:
    cargo watch -c -x check

# Run the server
run:
    cargo run -- --help

# Run the server with restart on changes
run-w:
    cargo watch -w src -w templates -w static -c -x 'test -- --nocapture' -x run

test:
    cargo test

# Run tests on change continuously
test-w:
    cargo watch -c -x test

# Run tests with coverage
# test-coverage:
# cargo tarpaulin -- --test-threads=1

# Run tests with coverage and open the report
# test-coverage-report:
# cargo tarpaulin --out Html && open ./tarpaulin-report.html

# Run extensive Clippy linter checks
run-clippy:
    cargo clippy --all-targets -- -D clippy::all -D clippy::pedantic

# Clean the build artifacts
clean:
    cargo clean

bench:
    cargo bench
