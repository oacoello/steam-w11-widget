$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
Set-Location $PSScriptRoot

$sdkBin = "C:\Users\oacoello\.nuget\packages\microsoft.windows.sdk.buildtools\10.0.22621.756\bin\10.0.22621.0\x64"

Write-Host "Compilando Steam Library Widget Provider (release)..." -ForegroundColor Cyan
cargo build --release --offline

Write-Host "Copiando ejecutable actualizado e incrustando manifiesto WinRT SxS..." -ForegroundColor Cyan
Copy-Item ".\target\release\steam-w11-widget.exe" ".\steam-w11-widget.exe" -Force
& "$sdkBin\mt.exe" -nologo -manifest ".\app.manifest" -outputresource:".\steam-w11-widget.exe;#1"

Write-Host "Generando índice de recursos (resources.pri)..." -ForegroundColor Cyan
& "$sdkBin\makepri.exe" new /pr "." /cf ".\priconfig.xml" /in "SteamLibraryWidgetProvider" /of ".\resources.pri" /o | Out-Null

Write-Host "Registrando proveedor de Widgets en Windows 11..." -ForegroundColor Cyan
Add-AppxPackage -Register ".\AppxManifest.xml" -ForceApplicationShutdown

Write-Host "Reiniciando servicio de Widgets de Windows 11 para refrescar la lista..." -ForegroundColor Cyan
Stop-Process -Name "WidgetService", "WidgetBoard" -Force -ErrorAction SilentlyContinue

Write-Host "¡Listo! Abre el Panel de Widgets (Win + W), haz clic en '+' y añade 'Steam Library'." -ForegroundColor Green
