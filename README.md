# Installation

You can install the Miru using one of the following methods:

## Direct Install

For a direct installation (no package manager), you can use curl:

```bash
curl -fsSL https://raw.githubusercontent.com/miruml/agent/main/install.sh | sh
```

## Manual Install

For a more careful approach, you can download and verify the script first:

1. Download the installation script:
```bash
curl -fsSL https://raw.githubusercontent.com/miruml/cli/main/install.sh -o install.sh
```

2. Review the contents (recommended):
```bash
less install.sh
```

3. Run the script:
```bash
sh install.sh
```

## Using Homebrew

If you prefer using Homebrew, you can install Miru CLI with:

```bash
brew install miruml/cli/miru
```

## Supported Platforms

- macOS (Intel and Apple Silicon)
- Linux (x86_64 and ARM64)

## Verification

After installation, verify the installation by running:

```bash
miru --help
```

## Notes

- The script requires `curl`, `tar`, `grep`, and `cut` to be installed
- Sudo privileges may be required for installation
- The default installation directory is `/usr/local/bin` (or `/opt/homebrew/bin` for Apple Silicon Macs)

## Uninstallation

### If installed via script
Remove the binary from your system:
```bash
sudo rm "$(which miru)"
```

### If installed via Homebrew
Uninstall using Homebrew:
```bash
brew uninstall miru
```

### Additional Cleanup
To remove any remaining configuration files:
```bash
rm -rf ~/.miru
```