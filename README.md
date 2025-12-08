# Caboose

A modern terminal UI for Rails development with Material Design 3 styling and Claude CLI-inspired commands.

## Features

- **5 Beautiful Themes** - Material Design 3, Solarized Dark, Dracula, Nord, Tokyo Night
- **Command palette** with autocomplete and fuzzy matching (press `:`)
- **Runtime theme switching** - Change themes on the fly with `/theme`
- **ASCII icons** for universal terminal compatibility
- **Multiple views**: Logs, Query Analysis, Database Health, Tests, Exceptions
- **Search & filtering** with real-time updates

## Quick Start

```bash
cargo build --release
./target/release/caboose
```

## Documentation

- **Commands**: See [COMMANDS.md](COMMANDS.md) for command palette guide
- **Themes**: See [THEMES.md](THEMES.md) for color theme options
- **Icons**: See [ICONS.md](ICONS.md) for icon configuration
- **API Docs**: Run `cargo doc --no-deps --open`

## Customization

### Color Themes
Switch between 5 beautiful themes:
```bash
: /theme dracula
```

Available: `material`, `solarized`, `dracula`, `nord`, `tokyo-night`

See [THEMES.md](THEMES.md) for details.

### Icons
By default, Caboose uses **ASCII icons** that work in any terminal:
```
[✓] Success  [✗] Error  [!] Warning  [git] Git  [db] Database
```

To enable Nerd Font icons, see [ICONS.md](ICONS.md).
