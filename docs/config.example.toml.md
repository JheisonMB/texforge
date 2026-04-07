# Texforge Global Configuration

This is an example of a `~/.texforge/config.toml` file. Copy this to `~/.texforge/config.toml` and customize for your workflows.

## Location

The configuration file is located at:
- **Linux/macOS**: `~/.texforge/config.toml` or `$XDG_CONFIG_HOME/texforge/config.toml`
- **Windows**: `%USERPROFILE%\.texforge\config.toml`

## Example Configuration

```toml
# User information — used as defaults in new projects
[user]
name = "Jane Doe"
email = "jane@example.com"

# Institution details — embedded in generated documents
[institution]
name = "UniverLab"
address = "Av. Example 123, Madrid, Spain"

# Default values for LaTeX documents
[defaults]
documentclass = "article"      # article | book | report | letter
fontsize = "11pt"              # 10pt | 11pt | 12pt
papersize = "a4paper"          # a4paper | letterpaper | etc
language = "es"                # es | en | fr | de | etc

# Templates configuration
[templates]
source = "github:UniverLab/texforge-templates"
auto_update = true             # Check for template updates automatically
watch = false                  # Watch for template changes in --watch mode
```

## Configuration Management

### View configuration
```bash
texforge config list
```

### Get a specific value
```bash
texforge config get user.name
```

### Set a value
```bash
texforge config set user.name "Your Name"
texforge config set defaults.language en
```

## Placeholder Resolution Chain

When generating a project from a template, placeholder values are resolved in this order:

1. **CLI arguments** (if you specify `--title "My Doc"`)
2. **Project config** (`./.texforge/config.toml` in the project directory)
3. **User config** (`~/.texforge/config.toml` — this file)
4. **Template defaults** (`template.toml` in the template)
5. **Interactive prompt** (if value is required and not found above)

This means your global config acts as a fallback — you can override it per-project or per-command.

## Example Usage

After setting up this config:

```bash
# Create a new project
texforge new mi-tesis -t apa-general

# The template will auto-fill:
# - author → "Jane Doe" (from user.name above)
# - institution → "UniverLab"
# - language → "es" (from defaults)
# - documentclass → "article"
```

If the template has placeholders matching the config keys, they'll be filled automatically.
