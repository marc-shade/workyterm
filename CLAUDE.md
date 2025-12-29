# CLAUDE.md - WorkyTerm

## Project Overview

WorkyTerm is a terminal UI application for non-programmers that provides AI-assisted content generation with animated pixel art workers. Built with Rust and Ratatui.

## Architecture

### Core Components

- **main.rs**: Entry point, terminal setup, event loop
- **app.rs**: Application state, input handling, task submission
- **config.rs**: TOML-based configuration with provider settings
- **error.rs**: Custom error types

### UI Module (`src/ui/`)

- **mod.rs**: Main layout with title, input, office, output, status bar
- **office.rs**: Virtual office rendering with animated workers
- **widgets.rs**: Custom Ratatui widgets (progress bar, sparkline)

### Workers Module (`src/workers/`)

- **mod.rs**: Worker struct, Office container, WorkerState enum
- **sprites.rs**: ASCII art sprite definitions for workers and furniture
- **animations.rs**: Animation controller, easing functions, particle system

### LLM Module (`src/llm/`)

- **mod.rs**: Module exports
- **provider.rs**: LlmProvider trait and implementations (Ollama, OpenAI, Anthropic)
- **council.rs**: Multi-model deliberation with synthesis

## Common Commands

```bash
# Build
cargo build

# Run in development
cargo run

# Build release
cargo build --release

# Run tests
cargo test

# Check for issues
cargo clippy

# Format code
cargo fmt
```

## Key Patterns

### Async/Await
All LLM operations are async using Tokio runtime. The main event loop uses `event::poll()` with timeout to allow animations while waiting for user input.

### Animation Tick
The `App::tick()` method is called ~20 times per second to update animations. Workers have frame-based sprite animations keyed off the tick counter.

### Provider Abstraction
The `LlmProvider` trait allows different AI backends. Providers implement `generate(&self, prompt: &str) -> Result<String>`.

### Configuration
Config loads from `~/.config/workyterm/config.toml` with defaults created on first run. API keys can reference environment variables with `$VAR_NAME` syntax.

## File Locations

- Config: `~/.config/workyterm/config.toml`
- Output: `~/Documents/WorkyTerm/` (configurable)

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

## Dependencies

Key dependencies:
- `ratatui` + `crossterm`: TUI framework
- `tokio`: Async runtime
- `reqwest`: HTTP client for API calls
- `serde` + `toml`: Configuration
- `clap`: CLI arguments
- `async-trait`: Async trait support

## Adding a New LLM Provider

1. Add struct in `src/llm/provider.rs`
2. Implement `LlmProvider` trait
3. Add to `create_provider()` factory function
4. Add config section in `src/config.rs`

## Adding New Worker Animations

1. Add new `WorkerState` variant in `src/workers/mod.rs`
2. Add sprite frames in `get_worker_sprite()` in `src/ui/office.rs`
3. Add color mapping in `get_worker_color()`
