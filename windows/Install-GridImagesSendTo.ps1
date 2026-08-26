[CmdletBinding()]
param(
    [string] $GridImagesPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($GridImagesPath)) {
    $commands = @(Get-Command 'grid-images.exe' -CommandType Application -ErrorAction SilentlyContinue)
    if ($commands.Count -eq 0) {
        throw 'grid-images.exe was not found. Install it first or specify -GridImagesPath.'
    }
    # mise can expose both its installed binary and its shim. Get-Command may
    # return both, so use the first one according to PowerShell's lookup order.
    $GridImagesPath = $commands[0].Source
}

$GridImagesPath = (Resolve-Path -LiteralPath $GridImagesPath).Path
if (-not (Test-Path -LiteralPath $GridImagesPath -PathType Leaf)) {
    throw "grid-images executable is not a file: $GridImagesPath"
}
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
