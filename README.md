# WorkyTerm

A friendly terminal application for non-programmers featuring animated pixel art AI workers in a virtual office.

```
  ╭─────────────────────────────────────────────────╮
  │              Welcome to WorkyTerm!               │
  │                                                   │
  │     ╭───────╮        Let me think about this...  │
  │     │ ≡ ≡ ≡ │        ○                           │
  │     └───┬───┘       o                            │
  │         │                                         │
  │       O     O!    O>>    \O/                     │
  │      /|\   /|\   /|\      |                      │
  │      / \   / \   / \     / \                     │
  │     Pixel  Byte  Nova   Chip                     │
  │    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━           │
  ╰─────────────────────────────────────────────────╯
```

## What is WorkyTerm?

WorkyTerm brings the power of multiple AI models to your terminal in a fun, accessible way. Watch your AI workers collaborate in a virtual office as they generate content, research topics, and solve problems.

**Built for:**
- Marketers creating content and campaigns
- Researchers gathering and synthesizing information
- Planners organizing projects and ideas
- Learners exploring new topics
- Anyone who wants AI assistance without the complexity

## Features

### Animated AI Office
Watch pixel art characters work on your tasks in real-time. Each worker has personality and specialization:
- **Pixel** - The Writer, crafts compelling content
- **Byte** - The Researcher, finds and verifies information
- **Nova** - The Analyst, breaks down complex problems
- **Chip** - The Creative, brings fresh ideas
- **Luna** - The Editor, polishes and refines output

### LLM Council Deliberation
Multiple AI models can collaborate on your tasks:
- Get diverse perspectives from different AI providers
- Multi-round deliberation for better results
- Automatic synthesis of the best ideas
- Works with Ollama (local), OpenAI, Anthropic, and more

### Beginner Friendly
- No coding required - just type what you need
- Visual progress with entertaining animations
- Clear, simple interface
- Helpful status messages
- Auto-saves your work

### Power User Ready
- Keyboard shortcuts for efficiency
- Configurable providers and models
- Council mode for multi-AI deliberation
- CLI arguments for automation
- TOML configuration file

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/marc-shade/workyterm.git
cd workyterm

# Build release version
cargo build --release

# Run
./target/release/workyterm
```

### Requirements

- Rust 1.70+
- An LLM provider:
  - [Ollama](https://ollama.ai) (recommended for local use)
  - OpenAI API key
  - Anthropic API key

## Quick Start

1. **Install Ollama** (optional but recommended):
   ```bash
   # macOS
   brew install ollama
   ollama serve
   ollama pull llama3.2
   ```

2. **Run WorkyTerm**:
   ```bash
   workyterm
   ```

3. **Type your task** and press Enter:
   ```
   Write a blog post about sustainable gardening for beginners
   ```

4. **Watch your AI workers** collaborate in the virtual office!

## Usage

### Interactive Mode

Just run `workyterm` and start typing:

```bash
workyterm
```

### With a Task

```bash
workyterm --task "Create a marketing email for our new product launch"
```

### Save Output

```bash
workyterm --task "Research the history of jazz" --output jazz-history.md
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Enter` | Submit task |
| `Tab` | Switch focus |
| `↑/↓` | Scroll output |
| `q` / `Esc` | Quit |

## Configuration

WorkyTerm uses a TOML configuration file at `~/.config/workyterm/config.toml`:

```toml
[providers.ollama]
endpoint = "http://localhost:11434"
model = "llama3.2"
enabled = true

[providers.openai]
endpoint = "https://api.openai.com/v1"
api_key = "$OPENAI_API_KEY"  # Uses environment variable
model = "gpt-4o-mini"
enabled = false

[providers.anthropic]
endpoint = "https://api.anthropic.com/v1"
api_key = "$ANTHROPIC_API_KEY"
model = "claude-3-5-sonnet-20241022"
enabled = false

default_provider = "ollama"

[council]
enabled = false  # Enable multi-AI deliberation
members = ["ollama", "openai"]
rounds = 2
consensus_threshold = 0.7

[ui]
animation_fps = 10
show_thoughts = true
worker_names = ["Pixel", "Byte", "Nova", "Chip", "Luna"]

[output]
directory = "~/Documents/WorkyTerm"
auto_save = true
format = "markdown"
```

## Architecture

WorkyTerm is built with:
- **[Ratatui](https://ratatui.rs)** - Terminal UI framework
- **[Tokio](https://tokio.rs)** - Async runtime
- **[Reqwest](https://docs.rs/reqwest)** - HTTP client for API calls
- **[Clap](https://docs.rs/clap)** - CLI argument parsing

### Project Structure

```
src/
├── main.rs           # Entry point and event loop
├── app.rs            # Application state management
├── config.rs         # Configuration handling
├── error.rs          # Error types
├── ui/
│   ├── mod.rs        # Main UI layout
│   ├── office.rs     # Virtual office rendering
│   └── widgets.rs    # Custom widgets
├── workers/
│   ├── mod.rs        # Worker management
│   ├── sprites.rs    # Pixel art definitions
│   └── animations.rs # Animation system
└── llm/
    ├── mod.rs        # LLM module
    ├── provider.rs   # Provider implementations
    └── council.rs    # Multi-LLM deliberation
```

## Roadmap

- [ ] Image generation support
- [ ] Voice input/output
- [ ] More worker personalities
- [ ] Custom sprite editor
- [ ] Plugin system
- [ ] Workflow templates
- [ ] Team collaboration mode

## Contributing

Contributions welcome! Please read our contributing guidelines first.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

Created by [Marc Shade](https://github.com/marc-shade) at 2 Acre Studios.

Inspired by the [LLM Council](https://github.com/marc-shade/llm-council) project and built with the excellent [Ratatui](https://github.com/ratatui/ratatui) framework.

---

*WorkyTerm: Making AI accessible, one pixel worker at a time.*
