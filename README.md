# Blog Engine

A fast, flexible blog engine built with Rust and Axum that can run standalone or deploy to Shuttle.

## Features

- **Framework Independent**: Core engine uses standard Axum with no cloud dependencies
- **Flexible Deployment**: Run locally for development or deploy to Shuttle for production
- **Markdown Support**: Write posts and pages in Markdown with frontmatter
- **Template Engine**: Tera-based templating system
- **Static File Serving**: Built-in static asset handling
- **Caching**: Intelligent caching system for better performance
- **Configuration**: YAML-based configuration system

## Quick Start

### Local Development

Run the blog engine locally for development:

```bash
# Run with default settings (content directory: ./content)
cargo run --bin blog-engine-main

# Run with custom directories via environment variables
BLOG_CONTENT_DIR=./my-content BLOG_DIR=./my-blog cargo run --bin blog-engine-main

# Run on custom host/port
HOST=0.0.0.0 PORT=8080 cargo run --bin blog-engine-main

# Auto-reload during development
cargo watch -x "run --bin blog-engine-main"
```

The server will start on `http://127.0.0.1:3000` by default.

### Shuttle Deployment

Deploy to Shuttle for production hosting:

```bash
# Deploy to Shuttle
cargo shuttle deploy --features shuttle

# Run Shuttle locally for testing
cargo shuttle run --features shuttle
```

## Project Structure

```
blog-engine/
├── src/
│   ├── bin/
│   │   ├── main.rs       # Standalone Axum server
│   │   └── shuttle.rs    # Shuttle deployment wrapper
│   ├── lib.rs            # Core blog engine logic
│   ├── blog_repository.rs
│   ├── cache.rs
│   ├── config.rs
│   ├── model.rs
│   └── renderer.rs
├── content/              # Your blog content (default)
│   ├── posts/           # Blog posts (.md files)
│   ├── pages/           # Static pages (.md files)
│   ├── templates/       # Tera templates
│   ├── static/         # Static assets (CSS, JS, images)
│   └── blog_config.yaml # Blog configuration
└── Cargo.toml
```

## Content Structure

### Blog Posts

Create Markdown files in `content/posts/`:

```markdown
---
title: "My First Post"
datePublished: "2024-12-06"
slug: "my-first-post"
---

# My First Post

This is the content of my first blog post!
```

### Pages

Create Markdown files in `content/pages/`:

```markdown
---
title: "About Me"
---

# About Me

This is my about page.
```

### Configuration

Create `content/blog_config.yaml`:

```yaml
site_title: "My Awesome Blog"
site_description: "A blog about awesome things"
```

### Templates

Create Tera templates in `content/templates/`:

- `index.html` - Blog post listing
- `post.html` - Individual blog post
- `page.html` - Static pages

## Rust Features Used

This project demonstrates several advanced Rust concepts:

1. **Conditional Compilation**: Uses `#[cfg(feature = "...")]` to enable different code paths
2. **Feature Flags**: Cargo features separate main vs. Shuttle dependencies
3. **Multiple Binaries**: Separate entry points sharing the same core library
4. **Trait Objects**: Runtime polymorphism with `dyn Trait`
5. **Smart Pointers**: `Arc<T>` for shared ownership across threads
6. **Async Traits**: `#[async_trait]` for async trait methods
7. **Zero-Cost Abstractions**: Compile-time optimizations with no runtime overhead

## Development Commands

```bash
# Check code without building
cargo check

# Check main binary only
cargo check --bin blog-engine-main

# Check shuttle binary only
cargo check --bin blog-engine-shuttle --features shuttle

# Run tests
cargo test

# Run tests with shuttle features
cargo test --features shuttle

# Build for main use (default)
cargo build --bin blog-engine-main

# Build for shuttle deployment
cargo build --bin blog-engine-shuttle --features shuttle

# Run benchmarks
cargo bench
```

## Environment Variables

For the main binary:

- `BLOG_CONTENT_DIR`: Directory containing your blog content (default: "content")
- `BLOG_DIR`: Directory containing templates and static files (default: "content")
- `HOST`: Server host address (default: "127.0.0.1")
- `PORT`: Server port (default: "3000")

## API Endpoints

- `GET /` - Blog post index
- `GET /{slug}` - Individual blog post
- `GET /p/{slug}` - Static page
- `GET /static/*` - Static assets
- `GET /health` - Health check endpoint

## Architecture Benefits

**Independence**: The core blog engine has no cloud provider dependencies. It uses standard Rust web ecosystem tools (Axum, Tokio, Tera) that work anywhere.

**Flexibility**: Run locally for development with hot reloading, then deploy to Shuttle (or any other platform) without code changes.

**Performance**: Compile-time optimizations mean you only pay for what you use. Main builds exclude Shuttle dependencies entirely.

**Maintainability**: Clean separation between core logic and deployment concerns makes the codebase easier to understand and modify.

## License

This project is open source. See the license file for details.