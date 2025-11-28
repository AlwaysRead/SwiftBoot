# EzBoot

A fast, intuitive TUI (Terminal User Interface) tool for managing UEFI boot configuration. Built with Rust for performance and safety, EzBoot makes boot management simple and visual—no more memorizing `efibootmgr` commands or dealing with bulky GUI tools.

## Features

- Reorder boot entries interactively
- Boot directly to a selected OS (one-time)
- Clean terminal interface

## Screenshots

<p align="center">
  <img src="assets/ss1.png" alt="EzBoot Main Interface" width="700"/>
  <br>
  <em>Main Interface</em>
</p>

<p align="center">
  <img src="assets/ss2.png" alt="EzBoot Options Menu" width="700"/>
  <br>
  <em>Options Menu</em>
</p>

## Installation

### Prerequisites

- **Rust & Cargo** - [Install from rustup.rs](https://rustup.rs/)
- **efibootmgr** - Required for UEFI boot management
  - Debian/Ubuntu: `sudo apt install efibootmgr`
  - Arch Linux: `sudo pacman -S efibootmgr`
  - Fedora: `sudo dnf install efibootmgr`
- **UEFI System** - This tool only works on UEFI systems (not legacy BIOS)
- **sudo privileges** - Required for modifying boot settings

### Quick Install

```bash
git clone https://github.com/AlwaysRead/ezboot.git
cd ezboot
./install.sh
```

The installation script will:
- Check for required dependencies
- Build the optimized release binary
- Install to `/usr/local/bin/ezboot`

### Manual Installation

```bash
git clone https://github.com/AlwaysRead/ezboot.git
cd ezboot
cargo build --release
sudo install -m 755 target/release/ezboot /usr/local/bin/ezboot
```

### Uninstallation

```bash
./uninstall.sh
```

Or manually:
```bash
sudo rm /usr/local/bin/ezboot
```

## Usage

Launch EzBoot from your terminal:

```bash
ezboot
```

### Keyboard Shortcuts

#### Navigation
- `Tab` - Switch between Boot Priority and Boot To panels
- `↑/↓` or `k/j` - Move selection up/down (vim-style navigation supported)

#### Boot Priority Panel
- `u/d` - Move the selected entry up/down in boot order
- `Enter` - Apply new boot order (requires reboot to take effect)

#### Boot To Panel
- `Enter` - Boot directly to selected OS on next reboot

#### Password Dialog
- `Tab` - Toggle password visibility
- `Enter` - Confirm password
- `Esc` - Cancel operation

#### General
- `?` or `h` - Show help screen with all keybindings
- `q` - Quit application (shows confirmation if there are unsaved changes)
- `Esc` - Cancel countdown timer before reboot

### Visual Indicators
- `→` marker - Indicates the current default boot entry
- Cyan highlight - Currently selected item
- Color-coded prompts - Green for confirmation, Red for warnings/errors

## How It Works

1. **View Boot Entries** - EzBoot reads your UEFI boot configuration using `efibootmgr`
2. **Modify Order** - Reorder entries in the Boot Priority panel using `u/d` keys
3. **Apply Changes** - Press `Enter` to save changes (requires sudo password)
4. **Boot To** - Select an entry in Boot To panel and press `Enter` to boot directly to that OS
5. **Countdown** - A 5-second countdown starts before rebooting (cancellable with `Esc`)
6. **Reboot** - System reboots to the selected entry

## Troubleshooting

### "Failed to run efibootmgr"
- Make sure you're running on a UEFI system (not legacy BIOS)

### "Incorrect password"
- The password prompt is for sudo access
- Press `Tab` to toggle password visibility if needed
- Press any key after the error to retry

### Changes not appearing
- Boot order changes require a reboot to take effect
- "Boot To" directly reboots to the selected OS

## Built With

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---
