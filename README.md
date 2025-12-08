# Caboose

A modern terminal UI for Rails development

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

## Install from GitHub Release (Linux)

Every merge to `main` publishes a Linux build to GitHub Releases.

```bash
# Download the latest Linux binary
curl -LO https://github.com/Balaji2682/caboose/releases/latest/download/caboose-linux-x86_64.tar.gz

# (Optional) verify checksum
curl -LO https://github.com/Balaji2682/caboose/releases/latest/download/caboose-linux-x86_64.tar.gz.sha256
sha256sum -c caboose-linux-x86_64.tar.gz.sha256

# Install into your PATH
tar -xzf caboose-linux-x86_64.tar.gz
sudo install caboose /usr/local/bin/caboose
caboose
```

## Documentation
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

To enable Nerd Font icons install it.
