# 🚂 Caboose

> **A powerful terminal-based development companion for Rails + Frontend applications**

Caboose is a modern, feature-rich TUI (Terminal User Interface) that brings IDE-like capabilities to your Rails and frontend development workflow. Monitor multiple processes, analyze database queries, track exceptions, run tests, and profile performance—all from a single, elegant terminal interface.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build](https://img.shields.io/github/actions/workflow/status/Balaji2682/caboose/ci.yml?branch=main)](https://github.com/Balaji2682/caboose/actions)

---

## ✨ Features

### 🎯 **Multi-Process Management**
- **Zero-Configuration Setup** - Auto-detects Rails and frontend frameworks (Angular, React, Vue, etc.)
- **PTY-Based Process Spawning** - Full terminal emulation with proper signal handling
- **Unified Process View** - Manage Rails server, Sidekiq workers, and frontend dev servers in one place
- **Smart Procfile Generation** - Automatically creates Procfiles when none exists
- **TOML Configuration** - Team-shareable settings via `.caboose.toml`
- **Environment Variable Management** - Automatic `.env` file loading with per-process overrides

### 📊 **Advanced Query Analysis**
- **N+1 Query Detection** - Automatically identifies and highlights N+1 query patterns
- **SQL Fingerprinting** - Groups similar queries for easy analysis
- **Query Performance Tracking** - Duration tracking with slow query identification
- **Request Context Aggregation** - See all queries executed within a specific HTTP request
- **Per-Endpoint Statistics** - Analyze database performance by controller action

### 🗄️ **Database Health Monitoring**
- **Health Score (0-100)** - Comprehensive database health assessment
- **Slow Query Tracking** - Identify performance bottlenecks
- **Missing Index Detection** - Suggests indexes for improved performance
- **SELECT * Warnings** - Flags inefficient queries
- **Table Statistics** - Monitor table sizes and row counts
- **Issue Prioritization** - Critical issues highlighted for immediate action

### 🧪 **Test Integration**
- **Framework Auto-Detection** - Supports RSpec, Minitest, and Test::Unit
- **Live Test Results** - Real-time test execution tracking
- **Success Metrics** - Pass/fail rates and test duration statistics
- **Slow Test Identification** - Find tests that need optimization
- **Debugger Detection** - Detects Pry, Byebug, and Debug breakpoints
- **Test Coverage Insights** - Track test run history

### 🐛 **Exception Tracking**
- **Automatic Detection** - Captures Ruby and Rails exceptions from logs
- **Smart Grouping** - Groups similar exceptions by fingerprint
- **Severity Classification** - Categorizes exceptions by severity level
- **File:Line Tracking** - Links exceptions to source code locations
- **Recency Stats** - Shows when exceptions last occurred
- **Exception Detail View** - Full stack traces and context

### 📈 **Real-Time Metrics & Monitoring**
- **Time-Series Data Storage** - Track metrics over time with configurable retention
- **CPU/Memory Monitoring** - System resource tracking per process
- **Request Rate Tracking** - Monitor throughput (req/sec)
- **Response Time Analysis** - P50, P95, P99 percentile calculations
- **Error Rate Monitoring** - Track application error percentages
- **Historical Trends** - Sparkline visualizations of metrics over time

### 🎨 **Beautiful Terminal UI**
- **5 Professional Themes** - Material Design 3, Solarized Dark, Dracula, Nord, Tokyo Night
- **Runtime Theme Switching** - Change themes without restarting (`:theme` command)
- **Smart Icon System** - ASCII fallback for universal compatibility, Nerd Font support
- **Responsive Layout** - Adapts to terminal size automatically
- **ANSI Code Stripping** - Clean log output without escape sequence artifacts
- **Smooth Animations** - Fade transitions between views

### 🔍 **Search & Filtering**
- **Live Search** - Real-time log filtering across all processes
- **Process Filtering** - Focus on specific process output
- **Query Search** - Find specific SQL queries instantly
- **Exception Search** - Filter exceptions by type or message
- **Fuzzy Command Matching** - Smart command palette with autocomplete

### 🛠️ **Developer Experience**
- **Command Palette** - Press `:` for powerful command interface
- **Keyboard Navigation** - Vim-inspired shortcuts for efficiency
- **View Cycling** - Quick switching between Logs, Queries, Database, Tests, Exceptions
- **Auto-Scroll** - Smart scrolling that follows new content
- **Log Export** - Export logs for external analysis
- **Git Integration** - Branch, status, and commit info in header

### 🌐 **Frontend Framework Support**
- **Angular** - Full support with ng serve integration
- **React** - Vite, Create React App, Next.js
- **Vue** - Vue CLI, Nuxt.js
- **Svelte** - SvelteKit
- **Other** - Remix, Astro, and custom setups

### 🚀 **Rails Integration**
- **Version Detection** - Automatic Rails version identification
- **Health Checks** - Validates migrations, database connectivity, bundle status
- **Database Detection** - PostgreSQL, MySQL, SQLite support
- **Background Jobs** - Sidekiq, Good Job, Solid Queue detection
- **Asset Pipeline** - Vite, Propshaft, Sprockets support

---

## 🚀 Quick Start

### Installation

#### From GitHub Release

**Linux (Ubuntu 20.04+)**
```bash
curl -LO https://github.com/Balaji2682/caboose/releases/latest/download/caboose-linux-x86_64.tar.gz
tar -xzf caboose-linux-x86_64.tar.gz
sudo install caboose /usr/local/bin/
```

**macOS (Intel)**
```bash
curl -LO https://github.com/Balaji2682/caboose/releases/latest/download/caboose-macos-intel.tar.gz
tar -xzf caboose-macos-intel.tar.gz
sudo install caboose /usr/local/bin/
```

**macOS (Apple Silicon M1/M2/M3)**
```bash
curl -LO https://github.com/Balaji2682/caboose/releases/latest/download/caboose-macos-apple-silicon.tar.gz
tar -xzf caboose-macos-apple-silicon.tar.gz
sudo install caboose /usr/local/bin/
```

**Windows (PowerShell)**
```powershell
Invoke-WebRequest -Uri "https://github.com/Balaji2682/caboose/releases/latest/download/caboose-windows-x86_64.zip" -OutFile "caboose.zip"
Expand-Archive -Path caboose.zip -DestinationPath "$env:USERPROFILE\caboose"
# Add to PATH or run: ~\caboose\caboose.exe
```

> **Platform Support**: Linux, macOS (Intel & Apple Silicon), Windows 10/11
>
> See [COMPATIBILITY.md](COMPATIBILITY.md) for detailed platform-specific instructions

#### From Source
```bash
# Clone the repository
git clone https://github.com/Balaji2682/caboose.git
cd caboose

# Build release binary
cargo build --release

# Install (Linux/macOS)
sudo cp target/release/caboose /usr/local/bin/

# Or on Windows
copy target\release\caboose.exe C:\Windows\System32\
```

### Basic Usage

```bash
# Navigate to your Rails project root
cd my-rails-app

# Start Caboose (auto-detects Rails + frontend)
caboose

# Or with explicit configuration
caboose --config .caboose.toml
```

That's it! Caboose will:
1. ✅ Detect your Rails application
2. ✅ Discover frontend frameworks (Angular, React, Vue, etc.)
3. ✅ Auto-generate a Procfile if needed
4. ✅ Start all processes with proper monitoring
5. ✅ Launch the beautiful TUI

---

## ⚙️ Configuration

### Zero Configuration (Recommended)

Just run `caboose` from your Rails root. It auto-detects everything:
- Rails (via `Gemfile` + `config/application.rb`)
- Frontend frameworks (via `package.json`, `angular.json`, etc.)
- Database type (PostgreSQL, MySQL, SQLite)
- Background job frameworks (Sidekiq, Good Job, Solid Queue)
- Package managers (npm, yarn, pnpm, bun)

### Configuration Hierarchy

```
1. Procfile (explicit process definitions)
2. .caboose.toml (team-shareable configuration)
3. Auto-detection (zero-config defaults)
```

### `.caboose.toml` Configuration

Create `.caboose.toml` in your project root for custom settings:

```toml
# Frontend Configuration
[frontend]
path = "angularV2"                    # Path to frontend directory
process_name = "angular"              # Name shown in logs
port = 4200                           # Override default port
dev_command = "npm start"             # Custom dev command

# Rails Configuration
[rails]
port = 3000                           # Rails server port

# Process-Specific Overrides
[processes.web]
command = "bundle exec puma -p 3000"
env = { RAILS_ENV = "development", RAILS_LOG_LEVEL = "debug" }

[processes.angular]
command = "cd angularV2 && npm start"
env = { NODE_ENV = "development" }
```

### Common Configuration Scenarios

#### Non-Standard Frontend Directory
```toml
[frontend]
path = "client"  # or "apps/web", "../web", etc.
```

#### Multiple Frontends
```toml
[frontend]
disable_auto_detect = true  # Disable auto-detection

# Define in Procfile instead:
# admin: cd admin && npm start
# customer: cd customer && npm start
```

#### Custom Ports
```toml
[rails]
port = 4000

[frontend]
port = 3001
```

#### Custom Package Manager
```toml
[frontend]
dev_command = "pnpm dev"  # or "bun dev", "yarn dev"
```

---

## ⌨️ Keyboard Shortcuts

### Global
| Key | Action |
|-----|--------|
| `q` | Quit application |
| `t` | Toggle between views (Logs → Query Analysis → Database Health → Tests → Exceptions) |
| `:` | Open command palette |
| `Esc` | Go back / Cancel |
| `?` | Show help |

### Navigation
| Key | Action |
|-----|--------|
| `↑` / `k` | Scroll up |
| `↓` / `j` | Scroll down |
| `PageUp` | Page up |
| `PageDown` | Page down |
| `Home` | Jump to top |
| `End` | Jump to bottom |

### Logs View
| Key | Action |
|-----|--------|
| `/` | Start search |
| `c` | Clear filters |
| `Enter` | Enable auto-scroll |
| `1-9` | Filter by process number |

### Query Analysis
| Key | Action |
|-----|--------|
| `Enter` | View request details |
| `↑` / `↓` | Select request |

### Exception View
| Key | Action |
|-----|--------|
| `Enter` | View exception details |
| `↑` / `↓` | Select exception |

---

## 🎨 Themes

Caboose includes 5 professionally designed themes:

### Change Theme at Runtime
```bash
: /theme dracula
```

### Available Themes

| Theme | Description |
|-------|-------------|
| `material` | Material Design 3 (Default) - Modern, vibrant |
| `solarized` | Solarized Dark - Easy on the eyes |
| `dracula` | Dracula - Popular dark theme |
| `nord` | Nord - Arctic, bluish palette |
| `tokyo-night` | Tokyo Night - Neon cyberpunk |

---

## 📊 Views & Features

### 1. Logs View
- **Multi-process log streaming** with color-coded output
- **Process filtering** - Focus on specific processes
- **Real-time search** - Filter logs as you type
- **Smart scrolling** - Auto-scroll follows new content
- **Process status** - Running, Stopped, Crashed indicators

### 2. Query Analysis View
- **Request-based grouping** - See all queries per HTTP request
- **N+1 detection warnings** - Highlights potential N+1 problems
- **Query duration** - Identify slow queries
- **Fingerprinting** - Groups similar queries
- **Request detail view** - Dive deep into specific requests

### 3. Database Health View
- **Health score** - 0-100 rating of database health
- **Slow query list** - Top slowest queries with durations
- **Performance issues** - Missing indexes, SELECT * usage
- **Recommendations** - Actionable suggestions for improvement
- **Table statistics** - Row counts and sizes

### 4. Test Results View
- **Live test tracking** - Real-time test execution monitoring
- **Framework detection** - RSpec, Minitest, Test::Unit
- **Success metrics** - Pass/fail rates and percentages
- **Slow test tracking** - Identify tests needing optimization
- **Debugger status** - Shows when Pry/Byebug breakpoints are hit

### 5. Exception Tracking View
- **Grouped exceptions** - Similar exceptions grouped together
- **Severity indicators** - Critical, High, Medium, Low
- **Occurrence counts** - How many times each exception occurred
- **Stack traces** - Full backtraces available
- **Source location** - File:line information

---

## 🔌 Command Palette

Press `:` to open the command palette. Available commands:

| Command | Description |
|---------|-------------|
| `/theme <name>` | Switch color theme |
| `/clear` | Clear logs |
| `/export <file>` | Export logs to file |
| `/filter <process>` | Filter by process name |
| `/help` | Show help information |

---

## 🏗️ Architecture

### Module Overview

| Module | Purpose |
|--------|---------|
| `cli` | Command-line argument parsing |
| `config` | TOML config and Procfile parsing |
| `process` | PTY-based process management |
| `parser` | Rails log parsing (HTTP, SQL, errors) |
| `query` | SQL fingerprinting and N+1 detection |
| `context` | Request-scoped query aggregation |
| `database` | Health scoring and issue detection |
| `stats` | Performance metrics collection |
| `metrics` | Advanced time-series metrics with CPU/memory monitoring |
| `test` | Test framework detection and result tracking |
| `exception` | Exception capture and grouping |
| `frontend` | Frontend framework detection |
| `rails` | Rails project detection |
| `git` | Git status integration |
| `ui` | Ratatui TUI components and views |

### Tech Stack

- **Language**: Rust 2024 Edition
- **TUI Framework**: [Ratatui](https://ratatui.rs/)
- **Terminal Backend**: [Crossterm](https://github.com/crossterm-rs/crossterm)
- **Process Management**: [portable-pty](https://github.com/wez/wezterm/tree/main/pty)
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **System Monitoring**: [sysinfo](https://github.com/GuillaumeGomez/sysinfo)

---

## 🛣️ Roadmap

### ✅ Implemented
- [x] Multi-process management
- [x] Rails and frontend auto-detection
- [x] N+1 query detection
- [x] Database health monitoring
- [x] Exception tracking
- [x] Test framework integration
- [x] Real-time metrics infrastructure
- [x] ANSI escape code stripping
- [x] Advanced time-series metrics
- [x] CPU/Memory monitoring

### 🚧 In Progress
- [ ] CPU/Memory profiling integration (stackprof, memory_profiler)
- [ ] Real-time metrics dashboard view
- [ ] Request/response time distribution (P50, P95, P99)
- [ ] Database query plan visualization (EXPLAIN)

### 📋 Planned
- [ ] Live code coverage integration
- [ ] Background job monitoring (Sidekiq, Good Job)
- [ ] Cache hit/miss analytics
- [ ] HTTP client request tracking
- [ ] Asset pipeline profiler
- [ ] API endpoint documentation generator
- [ ] Dependency vulnerability scanner
- [ ] Request flow tracer (waterfall)
- [ ] Git blame integration

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/Balaji2682/caboose.git
cd caboose

# Run in development mode
cargo run

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run clippy lints
cargo clippy -- -D warnings

# Generate documentation
cargo doc --no-deps --open
```

### Adding New Features

1. Create a new module in `src/`
2. Add module to `src/lib.rs`
3. Implement the feature following existing patterns
4. Add tests in `tests/`
5. Update README.md with new feature documentation

---

## 📝 License

MIT License - see [LICENSE](LICENSE) file for details

---

## 🙏 Acknowledgments

- Built with [Ratatui](https://ratatui.rs/) - Excellent TUI framework
- Inspired by [Overmind](https://github.com/DarthSim/overmind) and [Foreman](https://github.com/ddollar/foreman)
- Terminal icons powered by [Nerd Fonts](https://www.nerdfonts.com/)

---

## 📚 Documentation

### Full Documentation
- **API Docs**: Run `cargo doc --no-deps --open`
- **Module Docs**: See inline documentation in source files
- **Examples**: Check `example/` directory for sample projects

### Troubleshooting

#### Frontend Not Detected
- Ensure `package.json` exists in frontend directory
- Set explicit path in `.caboose.toml`: `[frontend] path = "your-path"`

#### Wrong Package Manager Used
- Check for correct lockfile: `yarn.lock`, `package-lock.json`, `pnpm-lock.yaml`, `bun.lockb`
- Or override in `.caboose.toml`: `[frontend] dev_command = "pnpm dev"`

#### Port Conflicts
- Override ports in `.caboose.toml`: `[rails] port = 4000` and `[frontend] port = 3001`

#### Bundle Install Needed
- Run `bundle install` before starting Caboose
- Caboose will detect and warn you if bundles are outdated

---

## 📧 Support

- **Issues**: [GitHub Issues](https://github.com/Balaji2682/caboose/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Balaji2682/caboose/discussions)

---

<div align="center">

**Made with ❤️ by developers, for developers**

[⭐ Star on GitHub](https://github.com/Balaji2682/caboose) • [🐛 Report Bug](https://github.com/Balaji2682/caboose/issues) • [💡 Request Feature](https://github.com/Balaji2682/caboose/issues)

</div>
