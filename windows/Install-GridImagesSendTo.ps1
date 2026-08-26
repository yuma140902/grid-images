[CmdletBinding()]
param(
    [string] $GridImagesPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($GridImagesPath)) {
    $command = Get-Command 'grid-images.exe' -CommandType Application -ErrorAction SilentlyContinue
    if ($null -eq $command) {
        throw 'grid-images.exe が見つかりません。先にインストールするか、-GridImagesPath で指定してください。'
    }
    $GridImagesPath = $command.Source
}

$GridImagesPath = (Resolve-Path -LiteralPath $GridImagesPath).Path
$sourceScript = Join-Path $PSScriptRoot 'SendTo-GridImages.ps1'
$installDirectory = Join-Path $env:LOCALAPPDATA 'grid-images'
$installedScript = Join-Path $installDirectory 'SendTo-GridImages.ps1'
$sendToDirectory = Join-Path $env:APPDATA 'Microsoft\Windows\SendTo'
$shortcutPath = Join-Path $sendToDirectory 'grid-images (クリップボード).lnk'

New-Item -ItemType Directory -Path $installDirectory -Force | Out-Null
Copy-Item -LiteralPath $sourceScript -Destination $installedScript -Force

$powerShellPath = Join-Path $PSHOME 'powershell.exe'
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $powerShellPath
$shortcut.Arguments = '-NoProfile -STA -ExecutionPolicy Bypass -File "{0}" -GridImagesPath "{1}"' -f $installedScript, $GridImagesPath
$shortcut.WorkingDirectory = $installDirectory
$shortcut.Description = '選択した画像を grid-images で結合してクリップボードへコピー'
$shortcut.Save()

Write-Host "インストールしました: $shortcutPath"
