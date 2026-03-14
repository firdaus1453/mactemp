# mactemp 🌡️

<img src="assets/icon_rounded.png" width="128" align="right" alt="mactemp icon" />

A lightweight macOS menu bar system monitor written in Rust. Displays real-time CPU/GPU temperature, CPU usage, and RAM usage directly in the macOS menu bar.

Built for **MacBook Pro 16-inch 2019 (Intel i9-9980HK)** and compatible with other Intel Macs running macOS 10.15+.

## ✨ Features

- ⚡️ **Ultra-Lightweight & Fast** — Written in 100% pure Rust. No Electron, no web views, no heavy frameworks.
- 🎯 **Pinpoint Accuracy** — Reads temperature directly from the raw hardware **die sensors** (`TC0D` / `TG0D`) via macOS Kernel (IOKit).
- 🥶 **Zero Bloat** — Consumes less than `< 2% CPU` and `< 50 MB RAM`. The entire application is roughly `~450 KB`!
- 🖥 **Native macOS Experience** — Integrates seamlessly into the Menu Bar with an elegant dropdown UI and a beautiful squircle icon.
- 🔒 **Privacy First** — Fully local processing. Zero telemetry, zero analytics, zero internet connection required.
- 🚀 **Set and Forget** — Includes a built-in "Auto Launch at Login" toggle so monitoring is always one click away.
- 🔍 **Comprehensive Stats** — Real-time tracking of:
  - CPU Die Temperature
  - Dedicated GPU Die Temperature (auto-hides if using Integrated Graphics)
  - Overall CPU Usage (%)
  - Physical RAM Usage (GB and %)

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

## 📥 Installation (For Regular Users)

The easiest way to install mactemp is to download the pre-built `.dmg` file.

1. Go to the [Releases page](https://github.com/firdaus1453/mactemp/releases/latest) and download `mactemp.dmg`.
2. Double-click the downloaded `.dmg` file to open it.
3. **Drag and drop** the `mactemp.app` icon into the **Applications** folder shortcut.
4. Open your Applications folder (or Launchpad).
5. **Important:** Because this is an indie open-source app, macOS Gatekeeper might block it the first time. To open it safely:
   - **Right-click** (or Control-click) on `mactemp.app`.
   - Select **Open** from the menu.
   - Click **Open** again on the pop-up warning.
6. The app will quietly start and appear in your menu bar at the top of the screen!

---

## 🛠 Quick Start (For Developers)

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

### Package as .dmg for Distribution

```bash
chmod +x create_dmg.sh
./create_dmg.sh
```

This creates a highly compressed `mactemp.dmg` file with an Applications folder shortcut for drag-and-drop installation.

Alternatively, just push a tag `v*` to trigger the GitHub Actions workflow which automatically builds a **Universal Binary** and publishes a `.dmg` release!

## Project Structure

```
mactemp/
├── Cargo.toml              # Dependencies and build config
├── create_app_bundle.sh    # .app bundle packaging script
├── README.md               # This file
└── src/
    ├── main.rs             # Entry point, event loop, wiring, locking
    ├── smc.rs              # SMC temperature reading (CPU/GPU die)
    ├── system_stats.rs     # CPU usage and RAM monitoring
    ├── menu.rs             # Menu bar title + dropdown construction
    └── autostart.rs        # LaunchAgent auto-start toggle
├── assets/                 # App icon and assets
├── .github/workflows/      # GitHub Actions CI/CD pipeline
└── create_dmg.sh           # DMG packaging script
```

## How SMC Temperature Reading Works

The **System Management Controller (SMC)** is a hardware chip in every Mac that manages thermal sensors, fans, and power. Temperature reading works as follows:

1. **Connection** — opens an IOKit connection to the `AppleSMC` service
2. **Sensor Keys** — reads sensors using FourCC keys that target the actual chip die:
   - `TC0D` — CPU **die** temperature (matches `powermetrics`)
   - `TG0D` — GPU **die** temperature (matches `powermetrics`)
3. **Fallback** — if the die sensor is unavailable, falls back to proximity sensors (`TC0P` / `TG0P`)
4. **Decoding** — raw bytes are converted to Celsius (`f64` values)
5. **No root required** — the SMC service is readable by the current user

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
| Binary size | ~368 KB |
| Total .app size | ~456 KB |
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
