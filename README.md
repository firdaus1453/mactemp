# mactemp 🌡️

A lightweight macOS menu bar system monitor written in Rust. Displays real-time CPU/GPU temperature, CPU usage, and RAM usage directly in the macOS menu bar.

Built for **MacBook Pro 16-inch 2019 (Intel i9-9980HK)** and compatible with other Intel Macs running macOS 10.15+.

## Features

- **CPU Temperature** — read directly from the SMC (System Management Controller)
- **GPU Temperature** — discrete GPU temperature (if available)
- **CPU Usage** — overall CPU utilization percentage
- **RAM Usage** — used/total memory with percentage
- **Auto-refresh** — updates every 2 seconds
- **Dropdown menu** — click for detailed system stats
- **Auto Launch at Login** — toggle via the dropdown menu
- **Minimal footprint** — ~366 KB binary, <2% CPU, <50 MB RAM

## Menu Bar Display

```
CPU 72°C | GPU 68°C | CPU 31% | RAM 55%
```

Click to see the detailed dropdown:
```
━━ System Stats ━━
  CPU Usage: 31.2%
  CPU Temperature: 72.4°C
  GPU Temperature: 68.1°C
─────────────────
  RAM Used:  9.2 GB
  RAM Total: 16.0 GB
  RAM Usage: 57.3%
─────────────────
↻ Refresh
  Auto Launch at Login: ✓
─────────────────
✕ Quit
```

## Quick Start

### Prerequisites

Install Rust (if not already installed):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### Build & Run

```bash
# Clone or navigate to the project
cd /path/to/mactemp

# Build release binary
cargo build --release

# Run the application
cargo run --release
```

The app will appear in your macOS menu bar — no window opens.

### Package as .app Bundle

```bash
chmod +x create_app_bundle.sh
./create_app_bundle.sh
```

This creates `mactemp.app` which you can:
- Move to `/Applications/`
- Double-click to launch
- Add to Login Items via System Settings

The `.app` bundle has `LSUIElement=true` set, meaning it runs as a menu bar-only app with no Dock icon.

## Project Structure

```
mactemp/
├── Cargo.toml              # Dependencies and build config
├── create_app_bundle.sh    # .app bundle packaging script
├── README.md               # This file
└── src/
    ├── main.rs             # Entry point, event loop, wiring
    ├── smc.rs              # SMC temperature reading (CPU/GPU)
    ├── system_stats.rs     # CPU usage and RAM monitoring
    ├── menu.rs             # Menu bar title + dropdown construction
    └── autostart.rs        # LaunchAgent auto-start toggle
```

## How SMC Temperature Reading Works

The **System Management Controller (SMC)** is a hardware chip in every Mac that manages thermal sensors, fans, and power. Temperature reading works as follows:

1. **Connection** — opens an IOKit connection to the `AppleSMC` service
2. **Sensor Keys** — reads sensors using FourCC keys:
   - `TC0P` — CPU proximity temperature
   - `TG0P` — GPU proximity temperature
3. **Decoding** — raw bytes are converted to Celsius (`f32` values)
4. **No root required** — the SMC service is readable by the current user

The `macsmc` crate handles all of this, providing a safe Rust API.

## Auto Launch at Login

The app uses **LaunchAgent** for auto-start:
- Toggle via the dropdown menu: **"Auto Launch at Login: ✓/✗"**
- Creates/removes a plist at `~/Library/LaunchAgents/com.mactemp.monitor.plist`
- Uses `RunAtLoad: true` to start at login

## Dependencies

| Crate | Purpose |
|---|---|
| `system_status_bar_macos` | macOS NSStatusBar integration |
| `macsmc` | SMC temperature sensor reading |
| `sysinfo` | CPU usage and RAM monitoring |

No async runtime, no heavy frameworks.

## Performance

| Metric | Value |
|---|---|
| Binary size | ~366 KB |
| CPU usage | <2% |
| Memory usage | <50 MB |
| Refresh interval | 2 seconds |

## Notes for Apple Silicon Macs

This app is built and tested on Intel Macs. On Apple Silicon (M1/M2/M3/M4):

- **SMC sensor keys** may differ — the `macsmc` crate has some Apple Silicon support but the FourCC keys are different
- **Temperature access** — Apple Silicon Macs may use different IOKit interfaces; newer macOS versions (14+) use SMC, older ones (12-13) may need IOHID
- **Cross-compilation** — to build for Apple Silicon from Intel: `rustup target add aarch64-apple-darwin && cargo build --release --target aarch64-apple-darwin`
- **Universal binary** — use `lipo` to combine both architectures: `lipo -create target/release/mactemp target/aarch64-apple-darwin/release/mactemp -output mactemp-universal`

## License

MIT
