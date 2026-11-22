# Steam Library Widget for Windows 11

Native Windows 11 Widgets Board provider (`IWidgetProvider` / Windows App SDK) built in Rust that automatically scans all installed Steam games across your Steam library folders and lets you launch them directly from the `Win + W` Widgets panel.

## Features

- **Automatic Steam Library Discovery**: Reads `HKCU\Software\Valve\Steam`, `steamapps/libraryfolders.vdf`, `appmanifest_*.acf`, and `userdata/<id>/config/localconfig.vdf`.
- **Sorted by Recently Opened**: Orders installed games by their most recent `LastPlayed` timestamp and promotes games to the top as soon as you launch them.
- **Playtime & Disk Size**: Displays hours played and disk usage in a compact format (e.g., `314,1 h | 321 MB`).
- **Responsive Widget Sizes**:
  - **Small**: Horizontal row of game icons/logos—click any icon to launch the game immediately.
  - **Medium & Large**: Spacious list with game icon, title, playtime, size on disk, and compact `▶` launch button.
- **Multi-Language Support (System Locale)**: Automatically adapts to the Windows UI language with built-in translations for **English, Spanish, Japanese, Portuguese, Russian, Mandarin Chinese, and French**.

## Installation & Development

Run the PowerShell installer script to compile the release binary, embed the WinRT Side-by-Side manifest, generate `resources.pri`, and register the widget provider package:

```powershell
.\install-widget.ps1
```

Then open the Windows 11 Widgets Board (`Win + W`), click **`+`**, and pin **Steam Library**.
