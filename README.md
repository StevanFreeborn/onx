# Onx

An agent-friendly CLI (built in Rust btw) for the Onspring API.

## Installation

### Linux & macOS

Install via shell script:

```bash
curl -fsSL https://raw.githubusercontent.com/StevanFreeborn/onx/main/scripts/install.sh | bash
```

To install a specific version or customize the install directory:

```bash
curl -fsSL https://raw.githubusercontent.com/StevanFreeborn/onx/main/scripts/install.sh | bash -s -- --version v0.1.0 --to ~/.local/bin
```

### Windows

Install via PowerShell:

```powershell
irm https://raw.githubusercontent.com/StevanFreeborn/onx/main/scripts/install.ps1 | iex
```

To install a specific version or customize the install directory:

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/StevanFreeborn/onx/main/scripts/install.ps1))) -Version v0.1.0 -InstallDir "$HOME\.onx\bin"
```

## Configuration

`onx` can be configured using command-line arguments, a JSON configuration file, or environment variables.

### Precedence

1. **Command-line flags** (`--api-key`, `--base-url`, `--pretty`)
2. **Configuration file** (`~/.config/onx/config.json` or `--config <PATH>`)
3. **Environment variables** (`ONSPRING_API_KEY`, `ONSPRING_BASE_URL`)

### Configuration File Format

Create `~/.config/onx/config.json`:

```json
{
  "apiKey": "your-api-key",
  "baseUrl": "https://api.onspring.com",
  "pretty": true
}
```
