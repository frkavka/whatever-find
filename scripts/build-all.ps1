# Cross-platform build script for file-search (PowerShell)

$ErrorActionPreference = "Stop"

Write-Host "Building file-search for multiple platforms..." -ForegroundColor Green

# Define targets
$targets = @(
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu", 
    "x86_64-apple-darwin",
    "aarch64-apple-darwin"
)

# Create output directory
New-Item -ItemType Directory -Force -Path "dist" | Out-Null

foreach ($target in $targets) {
    Write-Host "Building for $target..." -ForegroundColor Yellow
    
    # Add target if not already installed
    try {
        rustup target add $target *>$null
    } catch {
        # Target might already be installed
    }
    
    # Build with CLI features
    cargo build --release --target $target --features=cli
    
    # Copy binary to dist directory
    if ($target -like "*windows*") {
        Copy-Item "target\$target\release\file-search.exe" "dist\file-search-$target.exe"
    } else {
        Copy-Item "target\$target\release\file-search" "dist\file-search-$target"
    }
    
    Write-Host "✓ Built for $target" -ForegroundColor Green
}

Write-Host "All builds completed! Binaries are in the dist/ directory." -ForegroundColor Green
Write-Host ""
Write-Host "Available binaries:" -ForegroundColor Cyan
Get-ChildItem dist\ | Format-Table Name, Length, LastWriteTime