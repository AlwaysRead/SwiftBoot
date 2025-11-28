#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
    echo -e "${RED}Error: Do not run this script as root${NC}"
    echo "Run as a normal user with sudo privileges"
    exit 1
fi

# Check if ezboot is installed
if [ ! -f "/usr/local/bin/ezboot" ]; then
    echo -e "${YELLOW}ezboot is not installed in /usr/local/bin${NC}"
    exit 1
fi

echo -e "${YELLOW}Removing ezboot from /usr/local/bin...${NC}"
sudo rm -f /usr/local/bin/ezboot

echo -e "${GREEN}✓ Uninstallation complete!${NC}"
