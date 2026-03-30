# TexForge

> Self-contained LaTeX to PDF compiler — one curl, zero friction

TexForge is a command-line tool that compiles LaTeX documents to PDF without requiring TeX Live, MiKTeX, or any external LaTeX distribution. A single install script sets up everything you need.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/JheisonMB/texforge/main/install.sh | sh
```

This installs both `texforge` and `tectonic` (the LaTeX engine). Nothing else needed.

## Quick Start

```bash
# Create a new project from a template
texforge new mi-tesis

# Check for errors without compiling
texforge check

# Format your .tex files
texforge fmt

# Build to PDF
texforge build
```

## Features

- **One-command install**: Single curl sets up the full LaTeX toolchain
- **Clean errors**: Actionable error messages with line numbers and context
- **Template system**: Managed preambles, you only write document content
- **Built-in tools**: Formatter, linter, and build system in one CLI

## Project Structure

```
mi-tesis/
├── project.toml          # Project configuration
├── main.tex              # Entry point
├── sections/             # Document sections
│   └── body.tex
├── bib/
│   └── references.bib    # Bibliography
└── assets/
    └── images/           # Images and resources
```

## Commands

- `texforge new <name>` — Create new project from template
- `texforge new <name> -t <template>` — Create with specific template
- `texforge build` — Compile to PDF
- `texforge fmt` — Format .tex files
- `texforge fmt --check` — Check formatting without modifying
- `texforge check` — Lint without compiling
- `texforge template list` — List installed templates
- `texforge template add <name>` — Download template from registry
- `texforge template remove <name>` — Remove installed template

## Available Templates

| Template | Description |
|----------|-------------|
| `general` | Generic article (default) |
| `apa-general` | APA 7th edition report |
| `apa-unisalle` | Universidad de La Salle thesis |
| `ieee` | IEEE journal paper |
| `letter` | Formal Spanish correspondence |

## Roadmap

See [texforge-spec.md](texforge-spec.md) for the complete specification and roadmap.

## License

MIT — See [LICENSE](LICENSE)
