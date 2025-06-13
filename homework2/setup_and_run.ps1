# Set execution policy for current user if not already suitable
$currentPolicy = Get-ExecutionPolicy -Scope CurrentUser
if ($currentPolicy -ne 'RemoteSigned' -and $currentPolicy -ne 'Unrestricted') {
    Write-Host "Current execution policy is '$currentPolicy'. Setting it to 'RemoteSigned' to allow script execution..."
    Set-ExecutionPolicy RemoteSigned -Scope CurrentUser -Force
    Write-Host "Execution policy set to 'RemoteSigned'."
}

# Check if rustup is installed
if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
    Write-Host "`nRust is not installed. Installing Rust..."
    Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://win.rustup.rs'))
    # Append cargo bin to PATH for current process only
    $env:Path += ';' + $env:USERPROFILE + '\.cargo\bin'
} else {
    Write-Host "`nRust is already installed. Skipping installation and update."
}

# Change directory to the script location
Set-Location -Path (Split-Path -Parent $MyInvocation.MyCommand.Definition)

# Build and run the Rust project
Write-Host "`nStarting the project..."
cargo run
if ($LASTEXITCODE -ne 0) {
    Write-Host "`nBuild or run failed. Exiting."
    exit 1
}

Write-Host "`nExecution finished. Press any key to exit..."
[void][System.Console]::ReadKey($true)