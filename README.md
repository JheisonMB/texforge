# TexForge

> Self-contained LaTeX to PDF compiler — zero external dependencies

TexForge is a command-line tool that compiles LaTeX documents to PDF without requiring TeX Live, MiKTeX, or any external LaTeX distribution.

## Features

- **Self-contained**: Single binary, no external dependencies
- **Clean errors**: Actionable error messages with line numbers and context
- **Template system**: Managed preambles, you only write document content
- **Built-in tools**: Formatter, linter, and build system in one CLI

## Installation

```bash
cargo install texforge
```

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

## Project Structure

```
mi-tesis/
├── project.toml          # Project configuration
├── main.tex              # Entry point
├── capitulos/            # Chapters
│   ├── 01-intro.tex
│   └── 02-metodologia.tex
├── refs.bib              # Bibliography
└── assets/               # Images and resources
```

## Commands

- `texforge new <name>` — Create new project from template
- `texforge build` — Compile to PDF
- `texforge fmt` — Format .tex files
- `texforge check` — Lint without compiling
- `texforge template` — Manage templates

## Roadmap

See [texforge-spec.md](texforge-spec.md) for the complete specification and roadmap.

## License

MIT — See [LICENSE](LICENSE)
