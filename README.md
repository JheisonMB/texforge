# texforge

[![CI](https://github.com/JheisonMB/texforge/actions/workflows/ci.yml/badge.svg)](https://github.com/JheisonMB/texforge/actions/workflows/ci.yml)
[![Release](https://github.com/JheisonMB/texforge/actions/workflows/release.yml/badge.svg)](https://github.com/JheisonMB/texforge/actions/workflows/release.yml)
[![Crates.io](https://img.shields.io/crates/v/texforge)](https://crates.io/crates/texforge)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Self-contained LaTeX to PDF compiler — one curl, zero friction. No TeX Live, no MiKTeX, no Perl, no Node. A single install sets up everything you need.

---

## Installation

### Quick install (recommended)

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/JheisonMB/texforge/main/install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/JheisonMB/texforge/main/install.ps1 | iex
```

This downloads and installs `texforge`. No Rust toolchain required. Tectonic (the LaTeX engine) is installed automatically on first build.

You can customize the install:

```bash
# Pin a specific version
VERSION=0.1.0 curl -fsSL https://raw.githubusercontent.com/JheisonMB/texforge/main/install.sh | sh

# Install to a custom directory
INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/JheisonMB/texforge/main/install.sh | sh
```

```powershell
# Pin a specific version (PowerShell)
$env:VERSION="0.1.0"; irm https://raw.githubusercontent.com/JheisonMB/texforge/main/install.ps1 | iex
```

### Via cargo

```bash
cargo install texforge
```

Tectonic (the LaTeX engine) is installed automatically on first build. No extra steps needed.

Available on [crates.io](https://crates.io/crates/texforge).

### From source

```bash
git clone https://github.com/JheisonMB/texforge.git
cd texforge
cargo build --release
# Binary at target/release/texforge
```

### GitHub Releases

Check the [Releases](https://github.com/JheisonMB/texforge/releases) page for precompiled binaries (Linux x86_64, macOS x86_64/ARM64, Windows x86_64).

---

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

# Remove build artifacts
texforge clean
```

---

## Commands

| Command | Description |
|---|---|
| `texforge new <name>` | Create new project from template |
| `texforge new <name> -t <template>` | Create with specific template |
| `texforge init` | Initialize texforge in an existing LaTeX project |
| `texforge build` | Compile to PDF |
| `texforge clean` | Remove build artifacts |
| `texforge fmt` | Format .tex files |
| `texforge fmt --check` | Check formatting without modifying |
| `texforge check` | Lint without compiling |
| `texforge template list` | List installed templates |
| `texforge template list --all` | List installed + available in registry |
| `texforge template add <name>` | Download template from registry |
| `texforge template remove <name>` | Remove installed template |
| `texforge template validate <name>` | Verify template compatibility |

---

## Project Structure

`texforge new` generates this structure:

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

### `project.toml`

```toml
[documento]
titulo = "mi-tesis"
autor = "Author"
template = "general"

[compilacion]
entry = "main.tex"
bibliografia = "bib/references.bib"
```

---

## Templates

Templates are managed through the [texforge-templates](https://github.com/JheisonMB/texforge-templates) registry. The `general` template is embedded in the binary and works offline.

| Template | Description |
|---|---|
| `general` | Generic article (default, embedded) |
| `apa-general` | APA 7th edition report |
| `apa-unisalle` | Universidad de La Salle thesis |
| `ieee` | IEEE journal paper |
| `letter` | Formal Spanish correspondence |

```bash
# List installed templates
texforge template list

# Download a template
texforge template add apa-general

# Create project with specific template
texforge new mi-tesis -t apa-general
```

Templates are cached locally in `~/.texforge/templates/` after first download.

---

## Diagrams

`texforge build` intercepts embedded diagram environments before compilation. Originals are never modified — diagrams are rendered in `build/` copies.

### Mermaid

```latex
% Default: width=\linewidth, pos=H, no caption
\begin{mermaid}
flowchart LR
  A[Input] --> B[Process] --> C[Output]
\end{mermaid}

% With options
\begin{mermaid}[width=0.6\linewidth, caption=System flow, pos=t]
flowchart TD
  X --> Y --> Z
\end{mermaid}
```

### Graphviz / DOT

```latex
\begin{graphviz}[caption=Pipeline]
digraph G {
  rankdir=LR
  A -> B -> C
  B -> D
}
\end{graphviz}
```

Both rendered to PNG via pure Rust — no browser, no Node.js, no `dot` binary required.

| Option | Default | Description |
|---|---|---|
| `width` | `\linewidth` | Image width |
| `pos` | `H` | Figure placement (`H`, `t`, `b`, `h`, `p`) |
| `caption` | _(none)_ | Figure caption |

---

## Linter

`texforge check` runs static analysis without compiling:

- `\input{file}` — verifies file exists
- `\includegraphics{img}` — verifies image exists
- `\cite{key}` — verifies key exists in `.bib`
- `\ref{label}` / `\label{label}` — verifies cross-reference consistency
- `\begin{env}` / `\end{env}` — detects unclosed environments

```
ERROR [main.tex:47]
  \includegraphics{missing.png} — file not found

ERROR [main.tex:12]
  \cite{smith2020} — key not found in .bib

ERROR [main.tex:23]
  \begin{figure} never closed
  suggestion: Add \end{figure}
```

---

## Formatter

`texforge fmt` applies opinionated formatting inspired by `rustfmt`:

- Consistent indentation (2 spaces) inside environments
- Collapsed multiple blank lines
- Aligned `\begin{}`/`\end{}` blocks

One canonical output regardless of input style. Git diffs stay clean.

```bash
texforge fmt           # format in place
texforge fmt --check   # check without modifying (CI-friendly)
```

---

## Runtime Directory

```
~/.texforge/
  bin/
    tectonic            # LaTeX engine (auto-installed on first build)
  templates/
    general/            # Cached templates
    apa-general/
    ...
```

---

## Platform Support

| Platform | Architecture | Status |
|---|---|---|
| Linux | x86_64 | ✅ |
| macOS | x86_64 | ✅ |
| macOS | ARM64 (Apple Silicon) | ✅ |
| Windows | x86_64 | ✅ |

---

## Tech Stack

| Concern | Crate |
|---|---|
| CLI parsing | `clap` (derive) |
| Error handling | `anyhow` |
| Serialization | `serde` + `toml` |
| HTTP client | `reqwest` (blocking) |
| Archive extraction | `flate2` + `tar` |
| File traversal | `walkdir` |
| LaTeX engine | `tectonic` (external binary) |
| Mermaid renderer | `mermaid-rs-renderer` |
| Graphviz renderer | `layout-rs` |
| SVG → PNG | `resvg` |

---

---

## License

MIT
