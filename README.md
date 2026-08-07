# PS Battery

<img src="./images/reason.jpg" alt="Controller dies on 5% hp" width="800" />

I was so annoyed by my PlayStation controllers running out of battery without warning when playing on PC. I made this Rust app to give me a heads-up before it's too late.

## Features

1. Alerts you every 5 minutes if you have a low-battery controller connected via Bluetooth that is not charging.
2. If a low-battery controller is detected, alert the user in the following way. Controllers report their charge as a single 0-10 level, so the battery percentage is always a multiple of 10 and these are the only three levels that trigger an alert:
   - Show a Windows notification balloon based on remaining battery:
     - 0% => Error balloon
     - 10% => Warning balloon
     - 20% => Info balloon
   - Play a system sound when in game / fullscreen / presentation mode (to cut through the DND silence):
     - 0% => Critical Stop
     - 10% => Exclamation
     - 20% => Notification
3. Adds an application to the Windows tray. Opening the menu triggers an immediate background scan (instead of waiting for the next automatic check), then shows:
   - A "Scanning for controllers…" / "Scan completed" line while the scan is running and just after it finishes.
   - All connected controllers, their battery %, and whether they are charging.
   - **Run on startup**: run the app on Windows login (off by default).
   - **Open log**: open the log file (`%APPDATA%\ps-battery\ps-battery.log`).

<img src="./images/notification_20.png" alt="Info balloon at 20% battery" width="400" />

<img src="./images/notification_10.png" alt="Warning balloon at 10% battery" width="400" />

<img src="./images/notification_0.png" alt="Error balloon at 0% battery" width="400" />

<img src="./images/tray.png" alt="Tray menu showing connected controllers" width="400" />

## Microsoft Store packaging

The app is also packaged as an MSIX so it can be submitted to the Microsoft Store, where Microsoft signs the package and users don't get the "Unknown publisher" warning that the direct `.exe` download shows. The direct `.exe` download stays available either way.

Every release builds the MSIX and uploads it as a **workflow artifact** rather than a release asset, because it is signed with a throwaway self-signed certificate that end users do not trust. The Store re-signs it on submission.

Requires the [winapp CLI](https://github.com/microsoft/winappCli) (`winget install microsoft.winappcli`):

```
cargo build --release                                  # build the exe
mkdir dist && copy target\release\ps-battery.exe dist\  # stage the layout
winapp pack ./dist --manifest Package.appxmanifest --generate-cert
```

To run locally with real package identity (requires Windows Developer Mode; no signing needed):

```
winapp run .\target\release
```

## App icons

`icon-sources/store-icon.svg` is the source of truth for the app icon. Everything in `Assets/` is generated from it and shouldn't be hand-edited.

Regenerate from Windows after changing the SVG. The [winapp CLI](https://github.com/microsoft/winappCli) reads the SVG directly and rewrites every image referenced in `Package.appxmanifest`, at all required sizes and scales, along with `Assets/app.ico`, which `build.rs` embeds into the exe:

```
winapp manifest update-assets icon-sources/store-icon.svg
```

The committed assets were generated with winapp 0.5.0. Different versions rasterize the same SVG with slightly different antialiasing, so upgrading rewrites every generated image even when the SVG has not changed. That is expected rather than a problem: regenerate the full set, commit it in one go, and note the new version here so the next person knows what produced them.

## Privacy

PS Battery collects nothing and sends nothing, because it has no network access at all. It does write a local diagnostic log you can read or delete yourself. See [PRIVACY.md](PRIVACY.md).

## Disclaimer

I am a frontend/backend web developer. I have no prior experience building Windows applications or writing Rust code. Neither do I know anything about the PlayStation controller specifications. This has only been tested using my own controllers on my own Windows installation.

## Any issues?

This has been an awesome weekend project. If you have any issues, feel free to open a GitHub issue or contact me. Otherwise, this code is unlicensed, so do whatever you want with it: https://unlicense.org/
