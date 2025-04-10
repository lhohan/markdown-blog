content_dir := "./content"

build:
    cargo build --release --bin tt

# install:

# Check for warnings continuously
check-w:
    cargo watch -c -x check

# Run the server
run: collect-deploy-assets
    shuttle run

# Run the server with restart on changes
run-w: collect-deploy-assets
    cargo watch -w src -w templates -w static -c -x 'test -- --nocapture' -x 'shuttle run'

test:
    cargo test

# Run tests on change continuously
test-w:
    cargo watch -c -x test

content_pages_dir := content_dir + "/pages"
content_posts_dir := content_dir + "/posts"

collect-deploy-assets:
    rm -rf {{content_dir}}

    mkdir -p {{content_posts_dir}}
    cp -r posts/*.md {{content_posts_dir}}

    mkdir -p {{content_pages_dir}}
    cp -r pages/*.md {{content_pages_dir}}

    cp -r static {{content_dir}}
    cp -r templates {{content_dir}}
    cp blog_config.yaml {{content_dir}}

deploy: test collect-deploy-assets
    shuttle deploy


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
