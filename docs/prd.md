# Product Requirements Document: Markdown Blog Server

## Overview
Create a lightweight, fast blog server that renders Markdown content from a local directory. The server will be built with Rust and Axum, focusing on simplicity and performance.

## Core Requirements

1. **Markdown Rendering**
   - Parse and render Markdown files with YAML front matter
   - Support standard Markdown features (headers, lists, code blocks, etc.)

2. **Content Organization**
   - Serve blog posts by slug in URL (e.g., `/post-slug`)
   - Display posts with title, date, and formatted content

3. **User Interface**
   - Provide clean, readable presentation of blog content
   - Ensure responsive design for various screen sizes

## Technical Specifications

- **Backend**: Rust with Axum web framework
- **Content Processing**: Pulldown-cmark for Markdown parsing
- **Metadata Handling**: Gray-matter for front matter extraction
- **Deployment**: Local server running on port 3000

## Key Features (MVP)

- Single post view by slug URL
- Simple navigation
- Markdown rendering with basic styling
- Markdown rendering of code blocks
- Minimal overhead and fast response times

## Future Enhancements

- Index page showing all posts
- Tag-based navigation
- Post search functionality
- Static asset handling for custom CSS/JS
- RSS feed generation

## Success Criteria

- Server successfully renders Markdown content from local files
- Blog posts are accessible via URL slugs
- Content is presented in a clean, readable format
- Server startup and response times are fast

## More technical details


Here's a summary of the minimal Markdown blog server we created with Axum:

## Project Structure
```
blog-server/
├── src/
│   └── main.rs
├── content/
│   └── posts/        <- Your Markdown files go here
└── Cargo.toml
```

## Dependencies in Cargo.toml
```toml
[dependencies]
axum = "0.6.20"
tokio = { version = "1.32.0", features = ["full"] }
pulldown-cmark = "0.9.3"
gray_matter = "0.2.6"
```

## Main Features
1. Reads Markdown files with YAML front matter
2. Serves individual posts by their slug (e.g., `/fizzbuzz-functional-fun-in-scala`)
3. Converts Markdown to HTML for display
4. Simple, responsive styling included directly in the HTML response

## Future Enhancements
1. Add a proper index page listing all posts
2. Add templating with Tera
3. Add static file serving for CSS/JS
4. Add tag support
5. Add navigation between posts

## How to Run
1. Copy your Markdown files to `content/posts/`
2. Run with `cargo run`
3. Access at `http://localhost:3000/{slug}`

When you're ready to continue building, this minimal foundation makes it easy to incrementally add features while maintaining a working blog server at each step.

# Future Enhancements

## Task 003 Add tags support

## Task 004 Add paging support

## Task 005 Performance: stop reading all files on each request

## Task 006: Cache HTML instead of rendering each time

## Task 007: Simplify Cache setup?

## Task 008: Minimize Shuttle dependencies

- Be able to run locally without Shuttle?
