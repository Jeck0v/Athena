# Athena CLI PowerShell Build Script
# PowerShell equivalent of the Makefile for Windows users

param(
    [Parameter(Position=0)]
    [string]$Command = "help"
)

function Show-Help {
    Write-Host "Athena CLI - Available commands:" -ForegroundColor Cyan
    Write-Host "==================================" -ForegroundColor Cyan
    Write-Host "  help           " -ForegroundColor Green -NoNewline; Write-Host "Display help"
    Write-Host "  build          " -ForegroundColor Green -NoNewline; Write-Host "Build the project in release mode"
    Write-Host "  test           " -ForegroundColor Green -NoNewline; Write-Host "Run all tests"
    Write-Host "  install        " -ForegroundColor Green -NoNewline; Write-Host "Install Athena locally"
    Write-Host "  install-system " -ForegroundColor Green -NoNewline; Write-Host "System-wide installation (admin required)"
    Write-Host "  uninstall      " -ForegroundColor Green -NoNewline; Write-Host "Uninstall Athena"
    Write-Host "  clean          " -ForegroundColor Green -NoNewline; Write-Host "Clean build files"
    Write-Host "  dev            " -ForegroundColor Green -NoNewline; Write-Host "Development mode with tests"
    Write-Host "  demo           " -ForegroundColor Green -NoNewline; Write-Host "Full installation and demo"
    Write-Host "  check-install  " -ForegroundColor Green -NoNewline; Write-Host "Check installation"
    Write-Host "  install-part1  " -ForegroundColor Green -NoNewline; Write-Host "Install Part 1 (DSL parser + Docker Compose)"
    Write-Host "  install-part2  " -ForegroundColor Green -NoNewline; Write-Host "Install Part 2 (when available)"
    Write-Host ""
    Write-Host "Usage: .\build.ps1 [command]" -ForegroundColor Yellow
    Write-Host "Example: .\build.ps1 build" -ForegroundColor Yellow
}

function Build-Project {
    Write-Host "Building project..." -ForegroundColor Blue
    cargo build --release
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Build completed: target\release\athena.exe" -ForegroundColor Green
    } else {
        Write-Host "Error during build" -ForegroundColor Red
        exit 1
    }
}

function Test-Project {
    Write-Host "Running tests..." -ForegroundColor Blue
    cargo test
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Tests completed" -ForegroundColor Green
    } else {
        Write-Host "Error during tests" -ForegroundColor Red
        exit 1
    }
}

function Install-Project {
    Build-Project
    if ($LASTEXITCODE -ne 0) { return }

    Write-Host "Installing Athena..." -ForegroundColor Blue
    cargo install --path . --force
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Installation completed!" -ForegroundColor Green
        Write-Host "Test with: athena --help" -ForegroundColor Yellow
    } else {
        Write-Host "Error during installation" -ForegroundColor Red
        exit 1
    }
}

function Install-SystemWide {
    $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

    if (-not $isAdmin) {
        Write-Host "Administrator privileges required for system-wide installation" -ForegroundColor Red
        Write-Host "Run PowerShell as administrator or use 'install'" -ForegroundColor Yellow
        exit 1
    }

    Build-Project
    if ($LASTEXITCODE -ne 0) { return }

    Write-Host "System-wide installation of Athena..." -ForegroundColor Blue

    $installPath = "$env:ProgramFiles\Athena"
    if (-not (Test-Path $installPath)) {
        New-Item -ItemType Directory -Path $installPath -Force | Out-Null
    }

    Copy-Item "target\release\athena.exe" "$installPath\athena.exe" -Force

    $systemPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    if ($systemPath -notlike "*$installPath*") {
        [Environment]::SetEnvironmentVariable("Path", "$systemPath;$installPath", "Machine")
        Write-Host "Athena added to system PATH" -ForegroundColor Green
    }

    Write-Host "Athena installed in $installPath" -ForegroundColor Green
    Write-Host "Restart your terminal and test with: athena --help" -ForegroundColor Yellow
}

function Uninstall-Project {
    Write-Host "Uninstalling Athena..." -ForegroundColor Blue

    cargo uninstall athena 2>$null

    $installPath = "$env:ProgramFiles\Athena"
    if (Test-Path $installPath) {
        $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        if ($isAdmin) {
            Remove-Item $installPath -Recurse -Force -ErrorAction SilentlyContinue

            $systemPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
            if ($systemPath -like "*$installPath*") {
                $newPath = $systemPath -replace [regex]::Escape(";$installPath"), ""
                $newPath = $newPath -replace [regex]::Escape("$installPath;"), ""
                [Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
            }
        } else {
            Write-Host "System-wide installation detected, admin privileges required for full removal" -ForegroundColor Yellow
        }
    }

    Write-Host "Athena uninstalled" -ForegroundColor Green
}

function Clean-Project {
    Write-Host "Cleaning..." -ForegroundColor Blue
    cargo clean

    Remove-Item "docker-compose.yml", "my-compose.yml", "production-compose.yml" -ErrorAction SilentlyContinue
    Remove-Item "test-*.ath", "demo-*.ath" -ErrorAction SilentlyContinue

    Write-Host "Cleaning completed" -ForegroundColor Green
}

function Dev-Mode {
    Build-Project
    if ($LASTEXITCODE -ne 0) { return }

    Write-Host "Development mode..." -ForegroundColor Blue
    Test-Project
    if ($LASTEXITCODE -ne 0) { return }

    & ".\target\release\athena.exe" info
    Write-Host "Ready for development" -ForegroundColor Green
}

function Demo-Project {
    Install-Project
    if ($LASTEXITCODE -ne 0) { return }

    Write-Host "Running Athena demo..." -ForegroundColor Blue

    $demoContent = @"
DEPLOYMENT-ID DEMO

SERVICES SECTION

SERVICE web
IMAGE-ID nginx:alpine
PORT-MAPPING 80 TO 80
END SERVICE
"@

    $demoContent | Out-File -FilePath "demo.ath" -Encoding UTF8

    athena --verbose build demo.ath
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Demo completed! File: docker-compose.yml" -ForegroundColor Green
    } else {
        Write-Host "Error during demo" -ForegroundColor Red
    }
}

function Check-Installation {
    Write-Host "Checking installation..." -ForegroundColor Blue

    $athenaCmd = Get-Command athena -ErrorAction SilentlyContinue
    if ($athenaCmd) {
        Write-Host "Athena is installed:" -ForegroundColor Green
        Write-Host "   Path: $($athenaCmd.Source)" -ForegroundColor Cyan
        Write-Host "   Version:" -ForegroundColor Cyan
        athena --help | Select-Object -First 3 | ForEach-Object { Write-Host "   $_" -ForegroundColor Cyan }
    } else {
        Write-Host "Athena is not installed or not in PATH" -ForegroundColor Red
        Write-Host "Run: .\build.ps1 install" -ForegroundColor Yellow
    }
}

function Install-Part1 {
    Install-Project
    Write-Host "Part 1 installed - DSL parser and Docker Compose generation" -ForegroundColor Green
}

function Install-Part2 {
    Install-Project
    Write-Host "Part 2 in development - FastAPI/Go boilerplate" -ForegroundColor Yellow
}

switch ($Command.ToLower()) {
    "help" { Show-Help }
    "build" { Build-Project }
    "test" { Test-Project }
    "install" { Install-Project }
    "install-system" { Install-SystemWide }
    "uninstall" { Uninstall-Project }
    "clean" { Clean-Project }
    "dev" { Dev-Mode }
    "demo" { Demo-Project }
    "check-install" { Check-Installation }
    "install-part1" { Install-Part1 }
    "install-part2" { Install-Part2 }
    default {
        Write-Host "Unknown command: $Command" -ForegroundColor Red
        Write-Host ""
        Show-Help
        exit 1
    }
}
