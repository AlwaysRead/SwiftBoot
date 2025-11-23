# EzBoot
EzBoot is a fast, intuitive rust based TUI tool that makes managing your UEFI boot configuration simple and visual. No more memorizing `efibootmgr` commands or dealing with bulky GUI tools—just clean, arrow-key navigation in your terminal.

## Features

- View all UEFI boot entries at a glance
- Reorder boot priority interactively
- Set temporary "Boot Once" entries
- Clean, minimalistic design
- Built with Rust for performance and safety

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

### Arch Linux

```bash
git clone https://github.com/AlwaysRead/ezboot.git
cd ezboot
makepkg -s
sudo pacman -U ezboot-git-*.pkg.tar.zst
```

### Build from Source

```bash
git clone https://github.com/AlwaysRead/ezboot.git
cd ezboot
cargo build --release
sudo install -Dm755 target/release/ezboot /usr/bin/ezboot
```

## Usage

Launch EzBoot from your terminal:

```bash
ezboot
```

## Built With

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---
