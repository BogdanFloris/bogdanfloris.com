# AGENTS.md

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
- **Run tests**: `make test`

## Architecture

### Tech Stack
- **Backend**: Rust with Axum web framework
- **Templating**: Askama (type-safe templates in `/templates`)
- **Styling**: Tailwind CSS with custom Gruvbox theme
- **Development**: Nix flake for reproducible environment

### Project Structure
- `/src/main.rs`: Entry point with route handlers and CLI argument parsing
- `/src/blog.rs`: AppState, the page handler (with its redirects) and the 404 page
- `/src/content.rs`: Markdown rendering, frontmatter, essays and scribbles
- `/src/rss.rs`: RSS feed of essays
- `/templates/`: Askama HTML templates with partials for reusable components
- `/content/`: Pages as markdown with frontmatter, plus `scribbles.md` (formats in README.md)
- `/dist/`: Static assets served at `/dist` route

### Key Patterns
1. **Route Handlers**: Each page has an async handler in main.rs that creates a PageMeta struct
2. **Content**: Markdown files in `/content/`, loaded at startup into AppState. Every page is an essay at `/essays/<slug>` (the old `/post/<slug>` redirects there); `scribbles.md` is the scribbles stream
3. **Templates**: Each page has a corresponding template struct that derives `Template`

### Adding Features
- **New Route**: Add handler in main.rs, create template struct, add to router
- **New Page or Scribble**: See "Writing" in README.md; no code changes needed
- **Static Assets**: Place in `/dist/` directory, accessible via `/dist/` URL path

### Deployment
- Docker multi-stage build creates minimal runtime image
- Deployed on Fly.io (config in fly.toml)
- Runs on port 8080 internally, configurable via CLI args
