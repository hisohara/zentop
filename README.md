# zentop

A htop-like CPU monitor for AMD Zen architecture processors, written in Rust.

## Features

- Real-time CPU usage monitoring with htop-style bar graphs
- Multiple view modes for AMD Zen topology:
  - **Core view** (`c`): Individual CPU cores
  - **CCD view** (`d`): Grouped by Core Complex Die
  - **NPS view** (`n`): Grouped by NUMA Per Socket nodes
  - **NUMA view** (`u`): Grouped by NUMA nodes
- SMT (Simultaneous Multi-Threading) toggle
- Automatic Zen generation detection
- Scrollable interface for systems with many cores

## Requirements

### Build Dependencies

- Rust 1.70 or later

The hwloc library is bundled via the `vendored` feature, so no system hwloc installation is required.

## Installation

### From source

```bash
git clone https://github.com/hisohara/zentop.git
cd zentop
cargo build --release
```

The binary will be available at `target/release/zentop`.

### Run directly

```bash
cargo run --release
```

## Usage

```bash
zentop [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `-r, --refresh-rate <MS>` | Refresh rate in milliseconds (default: 1000) |
| `-h, --help` | Print help information |
| `-V, --version` | Print version information |

### Key Bindings

| Key | Action |
|-----|--------|
| `c` | Switch to Core view |
| `d` | Switch to CCD view |
| `n` | Switch to NPS view |
| `u` | Switch to NUMA view |
| `s` | Toggle SMT display (all threads / physical cores only) |
| `h` / `?` | Show help overlay |
| `j` / `Down` | Scroll down |
| `k` / `Up` | Scroll up |
| `q` / `Esc` | Quit |

## Architecture

zentop uses the following libraries:

- **hwlocality**: Hardware topology detection via hwloc
- **sysinfo**: CPU usage statistics collection
- **ratatui**: Terminal UI framework
- **crossterm**: Cross-platform terminal handling

### Project Structure

```
src/
├── main.rs          # Application entry point
├── app.rs           # Application state management
├── config.rs        # CLI argument parsing
├── topology/        # CPU topology detection
│   ├── detector.rs  # hwloc-based topology detection
│   ├── types.rs     # Topology data structures
│   └── zen.rs       # AMD Zen-specific detection
├── stats/           # CPU statistics
│   ├── collector.rs # sysinfo-based stats collection
│   └── types.rs     # Stats data structures
├── ui/              # User interface
│   ├── renderer.rs  # Main rendering logic
│   ├── theme.rs     # Color scheme
│   ├── views/       # View mode implementations
│   └── widgets/     # Reusable UI components
└── event/           # Event handling
    └── handler.rs   # Keyboard input processing
```

## AMD Zen Topology

zentop is designed for AMD Zen architecture processors and understands:

- **CCD (Core Complex Die)**: Physical chiplet containing CPU cores
- **CCX (Core Complex)**: Group of cores sharing L3 cache
- **NPS (NUMA Per Socket)**: BIOS-configurable NUMA node grouping
- **NUMA nodes**: Memory locality domains

The tool automatically detects:
- Zen generation (Zen, Zen 2, Zen 3, Zen 4)
- NPS mode (NPS1, NPS2, NPS4)
- SMT configuration

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
