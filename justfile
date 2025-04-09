assets_dir := "./assets"

build:
    cargo build --release --bin tt

# install:

# Check for warnings continuously
check-w:
    cargo watch -c -x check

# Run the server
run:
    shuttle run

# Run the server with restart on changes
run-w:
    cargo watch -w src -w templates -w static -c -x 'test -- --nocapture' -x 'shuttle run'

test:
    cargo test

# Run tests on change continuously
test-w:
    cargo watch -c -x test

assets_pages_dir := assets_dir + "/pages"
assets_posts_dir := assets_dir + "/posts"

collect-deploy-assets:
    rm -rf {{assets_dir}}

    mkdir -p {{assets_posts_dir}}
    cp -r posts/*.md {{assets_posts_dir}}

    mkdir -p {{assets_pages_dir}}
    cp -r pages/*.md {{assets_pages_dir}}

    cp -r static {{assets_dir}}
    cp -r templates {{assets_dir}}
    cp blog_config.yaml {{assets_dir}}

deploy: collect-deploy-assets
    shuttle deploy --debug


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
