# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
- **Start development server with hot-reload**: `make dev` (runs cargo-watch)
- **Run Tailwind CSS watcher**: `make tailwind` (required for CSS changes)
- **Run both in parallel**: Run the above commands in separate terminals

### Build & Run
- **Build for development**: `make build`
- **Build for production**: `make build-release`
- **Run server**: `make run` or `cargo run -- -H 0.0.0.0 -p 3000` (supports custom host/port)

### Code Quality
- **Format code**: `make fmt` or `cargo fmt`
- **Lint with Clippy**: `cargo clippy`
- **Check types**: `cargo check`

### Testing
- **Run tests**: `make test` (Note: No tests currently exist)

## Architecture

### Tech Stack
- **Backend**: Rust with Axum web framework
- **Templating**: Askama (type-safe templates in `/templates`)
- **Styling**: Tailwind CSS with custom Gruvbox theme
- **Development**: Nix flake for reproducible environment

### Project Structure
- `/src/main.rs`: Entry point with route handlers and CLI argument parsing
- `/src/blog.rs`: Blog library with Post struct and AppState management
- `/templates/`: Askama HTML templates with partials for reusable components
- `/blog_posts/`: HTML content files for blog posts
- `/dist/`: Static assets served at `/dist` route

### Key Patterns
1. **Route Handlers**: Each page has an async handler in main.rs that creates a PageMeta struct
2. **Blog Posts**: Stored as HTML files in `/blog_posts/`, loaded at startup into AppState
3. **Templates**: Each page has a corresponding template struct that derives `Template`
4. **Theme Toggle**: Implemented client-side with localStorage persistence

### Adding Features
- **New Route**: Add handler in main.rs, create template struct, add to router
- **New Blog Post**: Create HTML file in `/blog_posts/`, add to `add_posts()` function
- **Static Assets**: Place in `/dist/` directory, accessible via `/dist/` URL path

### Deployment
- Docker multi-stage build creates minimal runtime image
- Deployed on Fly.io (config in fly.toml)
- Runs on port 8080 internally, configurable via CLI args