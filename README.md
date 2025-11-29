# Blanqr

A lightweight Windows system tray application that fills your screen with a single color. Useful for testing display accuracy, checking for dead pixels, or creating a clean backdrop.

## Features

- **Full-screen color overlay** with multi-monitor support
- **System tray integration** for background operation
- **Global hotkey** (`Ctrl+Shift+B`) for quick toggle
- **16 preset colors** (black, white, grays, and warm tones)
- **Custom color picker** via Windows color dialog

## Usage

1. Launch `blanqr.exe` - an icon appears in the system tray
2. **Toggle overlay:**
   - Double-click the tray icon, or
   - Press `Ctrl+Shift+B`
3. **Change color:** Right-click tray icon and select color
4. **Hide overlay:** Click anywhere, press `Escape`, or toggle again
5. **Exit:** Right-click tray icon and select exit

Note: Menu items appear in Japanese (色を選択 = Select color, 終了 = Exit).

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
