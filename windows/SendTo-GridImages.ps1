[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string] $GridImagesPath,

    [Parameter(Position = 0, ValueFromRemainingArguments = $true)]
    [string[]] $InputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$temporaryPath = Join-Path ([System.IO.Path]::GetTempPath()) (
    'grid-images-{0}.png' -f [guid]::NewGuid().ToString('N')
)
$clipboardImage = $null

try {
    $inputFiles = [System.Collections.Generic.List[string]]::new()

    foreach ($itemPath in $InputPath) {
        $item = Get-Item -LiteralPath $itemPath
        if ($item.PSIsContainer) {
            Get-ChildItem -LiteralPath $item.FullName -File |
                Sort-Object -Property Name |
                ForEach-Object { $inputFiles.Add($_.FullName) }
        }
        else {
            $inputFiles.Add($item.FullName)
        }
    }

    if ($inputFiles.Count -eq 0) {
        throw '入力ファイルがありません。空ではないフォルダ、または画像ファイルを選択してください。'
    }

    & $GridImagesPath '--output' $temporaryPath '--' @inputFiles
    if ($LASTEXITCODE -ne 0) {
        throw "grid-images が終了コード $LASTEXITCODE で失敗しました。"
    }

    Add-Type -AssemblyName System.Drawing
    Add-Type -AssemblyName System.Windows.Forms

    # Image.FromFile keeps the source file locked. Copy it into an independent
    # bitmap so the temporary PNG can always be removed in finally.
    $sourceImage = [System.Drawing.Image]::FromFile($temporaryPath)
    try {
        $clipboardImage = [System.Drawing.Bitmap]::new($sourceImage)
    }
    finally {
        $sourceImage.Dispose()
    }

    [System.Windows.Forms.Clipboard]::SetImage($clipboardImage)
}
catch {
    try {
        Add-Type -AssemblyName System.Windows.Forms
        [void] [System.Windows.Forms.MessageBox]::Show(
            $_.Exception.Message,
            'grid-images',
            [System.Windows.Forms.MessageBoxButtons]::OK,
            [System.Windows.Forms.MessageBoxIcon]::Error
        )
    }
    catch {
        Write-Error $_
    }
    exit 1
}
finally {
    if ($null -ne $clipboardImage) {
        $clipboardImage.Dispose()
    }
    Remove-Item -LiteralPath $temporaryPath -Force -ErrorAction SilentlyContinue
}
