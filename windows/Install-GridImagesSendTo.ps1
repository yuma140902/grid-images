[CmdletBinding()]
param(
    [string] $GridImagesPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($GridImagesPath)) {
    $command = Get-Command 'grid-images.exe' -CommandType Application -ErrorAction SilentlyContinue
    if ($null -eq $command) {
        throw 'grid-images.exe was not found. Install it first or specify -GridImagesPath.'
    }
    $GridImagesPath = $command.Source
}

$GridImagesPath = (Resolve-Path -LiteralPath $GridImagesPath).Path
$sourceScript = Join-Path $PSScriptRoot 'SendTo-GridImages.ps1'
$installDirectory = Join-Path $env:LOCALAPPDATA 'grid-images'
$installedScript = Join-Path $installDirectory 'SendTo-GridImages.ps1'
$sendToDirectory = Join-Path $env:APPDATA 'Microsoft\Windows\SendTo'
$shortcutPath = Join-Path $sendToDirectory 'grid-images to clipboard.lnk'

New-Item -ItemType Directory -Path $installDirectory -Force | Out-Null
Copy-Item -LiteralPath $sourceScript -Destination $installedScript -Force

$powerShellPath = Join-Path $PSHOME 'powershell.exe'
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $powerShellPath
$shortcut.Arguments = '-NoProfile -STA -ExecutionPolicy Bypass -File "{0}" -GridImagesPath "{1}"' -f $installedScript, $GridImagesPath
$shortcut.WorkingDirectory = $installDirectory
$shortcut.Description = 'Combine selected images and copy the result to the clipboard'
$shortcut.Save()

Write-Host "Installed: $shortcutPath"
