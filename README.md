# Installation

You can install the Miru using one of the following methods:

## Manual Install

For a manual installation (no package manager), you can use curl:

```bash
curl -fsSL https://raw.githubusercontent.com/miruml/agent/main/manual-install.sh | sh
```

> [!Note]
> The script requires `curl`, `tar`, `grep`, and `cut` to be installed and sudo privileges may be required

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

## Verify Installation 

Verify the installation by running:

```bash
sudo curl --unix-socket /run/miru/miru.sock http://localhost/v1/test
```

You should see something similar to the following

```bash
{"server":"miru-config-agent","status":"ok"}
```

## APT Repository Install
COMING SOON

## Supported Platforms

The Miru agent has been tested and verified to work on the following operating systems:

- Ubuntu 20.04 LTS
- Ubuntu 22.04 LTS
- Ubuntu 24.04 LTS
- NVIDIA Jetson (JetPack 5.1)
- NVIDIA Jetson (JetPack 6.1)
- Raspberry Pi OS (64-bit)

Other Linux distributions and versions should also work, but have not been explicitly tested. 
If you encounter issues on a different platform, please email ben@miruml.com.

## Uninstallation
```bash
sudo apt remove miru-agent
```

If you'd like to remove configuration files as well you can run
```bash
sudo apt purge miru-agent
```

