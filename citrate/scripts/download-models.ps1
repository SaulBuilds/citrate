# Citrate Model Download Script for Windows
# Downloads required AI models for enhanced capabilities

param(
    [switch]$All,
    [switch]$SevenB,
    [switch]$HalfB,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# Models directory
$ModelsDir = "$env:APPDATA\citrate\models"

# Colors
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "   Citrate AI Model Download Script    " -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

# Create models directory
if (-not (Test-Path $ModelsDir)) {
    New-Item -ItemType Directory -Path $ModelsDir -Force | Out-Null
}

Write-Host "Models directory: " -NoNewline
Write-Host $ModelsDir -ForegroundColor Yellow
Write-Host ""

# Model definitions
$Models = @{
    "qwen2-0.5b" = @{
        Repo = "Qwen/Qwen2-0.5B-Instruct-GGUF"
        File = "qwen2-0_5b-instruct-q4_k_m.gguf"
        Size = "379MB"
        Desc = "Fast lightweight model (bundled)"
    }
    "qwen2.5-coder-7b" = @{
        Repo = "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF"
        File = "qwen2.5-coder-7b-instruct-q4_k_m.gguf"
        Size = "4.4GB"
        Desc = "Enhanced coding model (recommended)"
    }
}

function Show-Help {
    Write-Host "Usage: .\download-models.ps1 [-All] [-SevenB] [-HalfB] [-Help]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -All     Download all models (~4.8GB total)"
    Write-Host "  -SevenB  Download Qwen2.5-Coder-7B (~4.4GB) - recommended"
    Write-Host "  -HalfB   Download Qwen2-0.5B (~379MB) - lightweight"
    Write-Host "  -Help    Show this help message"
    Write-Host ""
    Write-Host "Available models:"
    foreach ($key in $Models.Keys) {
        $model = $Models[$key]
        Write-Host "  " -NoNewline
        Write-Host $key -ForegroundColor Cyan -NoNewline
        Write-Host ": $($model.Desc) ($($model.Size))"
    }
    Write-Host ""
    Write-Host "The 0.5B model is bundled with the app and works immediately."
    Write-Host "For better AI capabilities, download the 7B model with: .\download-models.ps1 -SevenB"
}

function Download-Model {
    param(
        [string]$Key
    )

    $model = $Models[$Key]
    $url = "https://huggingface.co/$($model.Repo)/resolve/main/$($model.File)"
    $output = Join-Path $ModelsDir $model.File

    Write-Host "[$Key] " -ForegroundColor Cyan -NoNewline
    Write-Host $model.Desc
    Write-Host "  Size: " -NoNewline
    Write-Host $model.Size -ForegroundColor Yellow

    if (Test-Path $output) {
        $size = (Get-Item $output).Length / 1MB
        Write-Host "  " -NoNewline
        Write-Host "Already downloaded" -ForegroundColor Green -NoNewline
        Write-Host " ($([math]::Round($size, 1)) MB)"
        return
    }

    Write-Host "  Downloading from HuggingFace..."
    Write-Host "  URL: $url"

    try {
        # Use BITS for better download experience
        $ProgressPreference = 'Continue'
        Invoke-WebRequest -Uri $url -OutFile $output -UseBasicParsing
        Write-Host "  " -NoNewline
        Write-Host "Downloaded successfully" -ForegroundColor Green
    }
    catch {
        Write-Host "  " -NoNewline
        Write-Host "Failed to download: $_" -ForegroundColor Red
        if (Test-Path $output) {
            Remove-Item $output -Force
        }
    }
}

# Show help if no arguments or -Help
if ($Help -or (-not $All -and -not $SevenB -and -not $HalfB)) {
    Show-Help
    exit 0
}

Write-Host "Starting downloads..."
Write-Host ""

# Download requested models
if ($All -or $HalfB) {
    Download-Model "qwen2-0.5b"
    Write-Host ""
}

if ($All -or $SevenB) {
    Download-Model "qwen2.5-coder-7b"
    Write-Host ""
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "         Download Summary              " -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

foreach ($key in $Models.Keys) {
    $model = $Models[$key]
    $path = Join-Path $ModelsDir $model.File

    if (Test-Path $path) {
        $size = (Get-Item $path).Length / 1MB
        Write-Host "  " -NoNewline
        Write-Host "[OK]" -ForegroundColor Green -NoNewline
        Write-Host " ${key}: $([math]::Round($size, 1)) MB"
    }
    else {
        Write-Host "  " -NoNewline
        Write-Host "[--]" -ForegroundColor Yellow -NoNewline
        Write-Host " ${key}: Not downloaded"
    }
}

Write-Host ""
Write-Host "Models stored in: " -NoNewline
Write-Host $ModelsDir -ForegroundColor Yellow
Write-Host ""
Write-Host "Done!" -ForegroundColor Green
Write-Host ""
Write-Host "To use the downloaded model in Citrate:"
Write-Host "1. Open Citrate"
Write-Host "2. Go to Settings > AI Provider"
Write-Host "3. Select 'Local Model' and choose your preferred model"
