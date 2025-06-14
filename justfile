content_dir := "./content"

# Build the main (standalone) binary
build:
    cargo build --release --bin blog-engine-main

# Build the shuttle binary
build-shuttle:
    cargo build --release --bin blog-engine-shuttle --features shuttle

# Build both binaries
build-all: build build-shuttle

# Check for warnings continuously
check-w:
    cargo watch -c -x check

# Check shuttle-specific code
check-shuttle:
    cargo check --bin blog-engine --features shuttle

# Run the main server locally
run: collect-deploy-assets
    cargo run --bin blog-engine-main

# Run the shuttle server locally
run-shuttle: collect-deploy-assets
    shuttle run

# Run the main server with restart on changes
run-w: collect-deploy-assets
    cargo watch -w src -w content -c -x 'test -- --nocapture' -x 'run --bin blog-engine-main'

# Run the shuttle server with restart on changes
run-shuttle-w: collect-deploy-assets
    cargo watch -w src -w content -c -x 'test -- --nocapture' -x 'shuttle run'

# Run tests
test:
    cargo test

# Run tests with shuttle features
test-shuttle:
    cargo test --features shuttle

# Run all tests (both main and shuttle)
test-all: test test-shuttle

# Run tests on change continuously
test-w:
    cargo watch -c -x test

content_pages_dir := content_dir + "/pages"
content_posts_dir := content_dir + "/posts"

# Collect and organize assets for deployment/running
collect-deploy-assets:
    rm -rf {{content_dir}}

    mkdir -p {{content_posts_dir}}
    cp -r posts/*.md {{content_posts_dir}}

    mkdir -p {{content_pages_dir}}
    cp -r pages/*.md {{content_pages_dir}}

    cp -r static {{content_dir}}
    cp -r templates {{content_dir}}
    cp blog_config.yaml {{content_dir}}

# Deploy to Shuttle
deploy: test-all collect-deploy-assets
    shuttle deploy

# Run extensive Clippy linter checks
run-clippy:
    cargo clippy --all-targets -- -D clippy::all -D clippy::pedantic

# Clean the build artifacts
clean:
    cargo clean

# Run benchmarks
bench:
    cargo bench
