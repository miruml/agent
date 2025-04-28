# Installation

You can install the Miru using one of the following methods:

## Manual Install

For a manual installation (no package manager), you can use curl:

```bash
curl -fsSL https://raw.githubusercontent.com/miruml/agent/main/manual-install.sh | sh
```

### Examine the Installation Script

For a more careful approach, you can download and verify the script first:

1. Download the installation script:
```bash
curl -fsSL https://raw.githubusercontent.com/miruml/cli/main/manual-install.sh -o install.sh
```

2. Review the contents (recommended):
```bash
less install.sh
```

3. Run the script:
```bash
sh install.sh
```

## Using APT
COMING SOON

## Supported Platforms

- Ubuntu (x86_64 and ARM64)

## Verification

After installation, verify the installation by running:

```bash
miru --help
```

## Notes

- The script requires `curl`, `tar`, `grep`, and `cut` to be installed
- Sudo privileges are required for installation

## Uninstallation
```bash
sudo apt remove miru-agent
```

If you'd like to remove configuration files as well you can run
```bash
sudo apt purge miru-agent
```

