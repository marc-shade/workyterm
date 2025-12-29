# WorkyTerm

A Claude Code-style CLI assistant that uses installed AI CLI tools (claude, codex, gemini) or Ollama - no API keys required.

```
WorkyTerm 0.2.0
✓ 3 providers ready via gemini, codex, claude

Type your message, or /help for commands.

> Write a blog post about Rust programming
● Writing → Iris

Rust is a systems programming language that emphasizes safety,
performance, and concurrency...

(2.3s)
```

## What is WorkyTerm?

WorkyTerm provides multi-model AI access through a single CLI. It automatically detects and routes requests to available providers, making it ideal for:

- **Programmatic use** - JSON output mode for automation
- **Multi-model routing** - Different models for different tasks
- **Cost optimization** - Use fast/cheap models when appropriate
- **Privacy options** - Ollama for local-only processing

## Features

### Multi-Provider Support
- **Gemini CLI** - Fast, web-grounded responses
- **Codex CLI** - Code execution capabilities
- **Claude CLI** - Advanced reasoning
- **Ollama** - Local/private processing

### Claude Code-Style Interface
- Slash commands (`/help`, `/status`, `/team`, `/doctor`)
- File references with `@file.txt`
- Shell commands with `!command`
- Streaming responses
- Markdown rendering

### Programmatic Mode
```bash
# JSON output for parsing
workyterm -j -m gemini "What is Rust?"
{"success":true,"response":"...","model":"gemini-cli","elapsed_ms":7089,"cached":false}

# Quiet mode - just the response
workyterm -q -m codex "Explain this code"

# Task-based routing
workyterm -j -t research "Latest AI news"
```

### Response Caching
```bash
# First call hits API (~7s)
workyterm -j -m gemini "What is 2+2?"

# Second call from cache (~0ms)
workyterm -j -m gemini "What is 2+2?"

# Bypass cache
workyterm --no-cache -m gemini "What is 2+2?"

# Clear all cached responses
workyterm --clear-cache
```

## Installation

### From Source

```bash
git clone https://github.com/marc-shade/workyterm.git
cd workyterm
cargo build --release
./target/release/workyterm
```

### Requirements

At least one of:
- [Claude CLI](https://github.com/anthropics/claude-cli) - `claude`
- [Codex CLI](https://github.com/openai/codex-cli) - `codex`
- [Gemini CLI](https://github.com/google/gemini-cli) - `gemini`
- [Ollama](https://ollama.ai) running locally

## Usage

### Interactive Mode

```bash
workyterm
```

### One-Shot Queries

```bash
# Print mode - process and exit
workyterm -p "Explain quantum computing"

# JSON mode - for programmatic use
workyterm -j "What is machine learning?"

# Quiet mode - minimal output
workyterm -q "Hello world"
```

### CLI Options

| Flag | Description |
|------|-------------|
| `-p, --print` | Print mode (process and exit) |
| `-j, --json` | JSON output for parsing |
| `-q, --quiet` | Minimal output |
| `-m, --model` | Force model: gemini, codex, claude, ollama |
| `-t, --task` | Hint task type: research, code, write, analyze |
| `--no-cache` | Bypass response cache |
| `--cache-ttl` | Cache TTL in seconds (default: 3600) |
| `--clear-cache` | Clear cache and exit |
| `-v, --verbose` | Enable debug logging |

### Slash Commands

| Command | Description |
|---------|-------------|
| `/help` | Show help |
| `/status` | Session status |
| `/team` | Show team members |
| `/model` | Available models |
| `/doctor` | Diagnostic checks |
| `/cost` | Token usage |
| `/context` | Context usage |
| `/clear` | Clear history |
| `/exit` | Exit |

### Special Syntax

```bash
# Include file contents
> Explain @src/main.rs

# Run shell command
> !ls -la
```

## Task Routing

WorkyTerm automatically routes requests to appropriate providers:

| Task Type | Keywords | Default Provider |
|-----------|----------|------------------|
| Research | research, find, search, what is | Gemini (fast) |
| Analysis | analyze, debug, code, review | Codex |
| Writing | write, draft, compose, blog | Gemini |
| Creative | brainstorm, ideas, design | Claude |
| Editing | edit, improve, fix, rewrite | Claude |

Override with `-m` flag:
```bash
workyterm -m claude "Research quantum physics"
```

## Configuration

Config file: `~/.config/workyterm/config.toml`

```toml
[providers.ollama]
endpoint = "http://localhost:11434"
model = "llama3.2"
enabled = true
```

## Architecture

```
src/
├── main.rs           # CLI entry point
├── config.rs         # Configuration
├── cache.rs          # Response caching
├── team/
│   ├── mod.rs        # Support team orchestration
│   ├── analyzer.rs   # Request analysis
│   ├── members.rs    # Team member definitions
│   └── workflow.rs   # Task workflow
└── llm/
    ├── mod.rs        # LLM module
    ├── provider.rs   # CLI providers (claude, codex, gemini, ollama)
    └── council.rs    # Multi-model deliberation
```

## Use with Claude Code

WorkyTerm is designed to extend Claude Code's capabilities:

```bash
# In Claude Code, spawn WorkyTerm for Gemini research
result=$(workyterm -j -m gemini "Latest Rust 2024 features")

# Use Codex for code execution
workyterm -q -m codex "Run pytest and summarize results"

# Local processing with Ollama
workyterm -q -m ollama "Summarize this confidential doc"
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

Created by [Marc Shade](https://github.com/marc-shade) at 2 Acre Studios.
