#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
    echo -e "${RED}Error: Do not run this script as root${NC}"
    echo "Run as a normal user with sudo privileges"
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo not found${NC}"
    echo "Please install Rust: https://rustup.rs/"
    exit 1
fi

# Check if efibootmgr is available
if ! command -v efibootmgr &> /dev/null; then
    echo -e "${YELLOW}Warning: efibootmgr not found${NC}"
    echo "This tool requires efibootmgr to function"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo -e "${CYAN}Building ezboot...${NC}"
cargo build --release

echo -e "${CYAN}Installing ezboot to /usr/local/bin...${NC}"
sudo install -m 755 target/release/ezboot /usr/local/bin/ezboot

echo -e "${GREEN}✓ Installation complete!${NC}"
echo
echo "You can now run ezboot with:"
echo -e "  ${CYAN}ezboot${NC}"
echo
echo "Note: ezboot requires sudo privileges to modify boot settings"
