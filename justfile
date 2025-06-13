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
    cargo check --bin blog-engine-shuttle --features shuttle

# Run the main server locally
run: collect-deploy-assets
    cargo run --bin blog-engine-main

# Run the main server with custom settings
run-custom host="127.0.0.1" port="3000":
    HOST="{{host}}" PORT="{{port}}" cargo run --bin blog-engine-main

# Run the shuttle server locally
run-shuttle: collect-deploy-assets
    cargo shuttle run

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

# Run tests with shuttle features on change
test-shuttle-w:
    cargo watch -c -x 'test --features shuttle'

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
    cargo shuttle deploy

# Quick development setup (collect assets and run main server)
dev: collect-deploy-assets run

# Quick development with watch (collect assets and run with auto-reload)
dev-w: collect-deploy-assets run-w

# Development with custom host/port
dev-custom host="0.0.0.0" port="8080": collect-deploy-assets
    HOST="{{host}}" PORT="{{port}}" cargo run --bin blog-engine-main

# Run extensive Clippy linter checks
run-clippy:
    cargo clippy --all-targets -- -D clippy::all -D clippy::pedantic

# Run clippy on shuttle-specific code
run-clippy-shuttle:
    cargo clippy --all-targets --features shuttle -- -D clippy::all -D clippy::pedantic

# Run clippy on all code
run-clippy-all: run-clippy run-clippy-shuttle

# Clean the build artifacts
clean:
    cargo clean

# Run benchmarks
bench:
    cargo bench
