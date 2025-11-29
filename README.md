# Blanqr

A lightweight Windows system tray application that fills your screen with a single color. Useful for testing display accuracy, checking for dead pixels, or creating a clean backdrop.

## Features

- **Full-screen color overlay** with multi-monitor support
- **System tray integration** for background operation
- **Configurable global hotkey** (default: `Ctrl+Shift+B`)
- **16 preset colors** (black, white, grays, and warm tones)
- **Custom color picker** via Windows color dialog

## Usage

1. Launch `blanqr.exe` - an icon appears in the system tray
2. **Toggle overlay:**
   - Double-click the tray icon, or
   - Press `Ctrl+Shift+B` (configurable)
3. **Change color:** Right-click tray icon and select color
4. **Hide overlay:** Click anywhere, press `Escape`, or toggle again
5. **Exit:** Right-click tray icon and select exit

## Configuration

Settings are stored in `%APPDATA%\Blanqr\config.ini`:

```ini
hotkey = Ctrl+Shift+B
```

**Hotkey format:** `modifier+modifier+key`
- Modifiers: `Ctrl`, `Alt`, `Shift`, `Win`
- Keys: `A`-`Z`, `0`-`9`, `F1`-`F12`

Examples: `Ctrl+Alt+F1`, `Win+Shift+C`, `Ctrl+F12`

## Requirements

- Windows 10 or later
- [Rust](https://rustup.rs/) (for building from source)

## Building

```console
cargo build --release
```

The executable will be at `target/release/blanqr.exe`.

## License

MIT
