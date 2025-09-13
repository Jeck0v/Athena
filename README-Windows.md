# Athena CLI - Windows Installation Guide

This guide will help you install and use **Athena CLI** on Windows.

---

## Prerequisites

Before installing Athena, make sure you have:

1. **Rust** – Install from [rustup.rs](https://rustup.rs/)
2. **PowerShell 5.0+** – Included by default with Windows 10/11
3. **Git** (optional) – To clone the repository

---

## Installation

### Method 1: Local Installation (recommended)
```powershell
# Open PowerShell in the project directory
.\build.ps1 install
```

### Method 2: System-wide Installation (requires administrator privileges)
```powershell
# Open PowerShell as Administrator
.\build.ps1 install-system
```
## Available Commands
The PowerShell script build.ps1 provides the following commands:
```powershell
.\build.ps1 help              # Show help
.\build.ps1 build             # Build in release mode
.\build.ps1 test              # Run all tests
.\build.ps1 install           # Local installation
.\build.ps1 install-system    # System-wide installation (admin required)
.\build.ps1 uninstall         # Uninstall
.\build.ps1 clean             # Clean build artifacts
.\build.ps1 dev               # Development mode
.\build.ps1 demo              # Run a full demo
.\build.ps1 check-install     # Verify installation
```


## Usage After Installation
Once installed, Athena can be used directly from the terminal:
```powershell
athena --help
athena build my-project.ath
athena generate fastapi my-api
```

## Troubleshooting
### PowerShell Execution Policy
If you encounter an execution policy error when running scripts:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```
### PATH Not Updated
After a system-wide installation, restart your terminal or manually refresh PATH:
```powershell
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
```

## Uninstallation
### To uninstall Athena CLI:
```powershell
.\build.ps1 uninstall
```
This command removes:
- The local Cargo installation
- The system-wide installation (if admin privileges are available)
- Associated PATH entries
