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

`icon-sources/` holds the two hand-drawn icons. Everything in `Assets/` is generated from them and shouldn't be hand-edited:

| Source | Generates | Used for |
| --- | --- | --- |
| `icon-sources/store-icon.svg` | `Assets/*.png` | Store listing and Windows app list |
| `icon-sources/tray-icon.svg` | `Assets/app.ico` | The system tray icon, embedded into the exe by `build.rs` |

The two differ only in the notification badge, which the Store icon has and the tray icon does not. In the notification area a badge reads as an unread notification from the app rather than as part of its logo, so the tray variant drops it, and shifts its viewBox down by 30 to recentre the artwork the badge used to balance.

**Run the two steps below in order.** `winapp manifest update-assets` writes `Assets/app.ico` as well as the PNGs, so it replaces the tray icon with a badged one built from the Store source. Regenerating the tray icon afterwards puts the right one back. Running only the tray step is fine on its own.

Regenerate the Store assets from Windows after changing `store-icon.svg`. The [winapp CLI](https://github.com/microsoft/winappCli) reads the SVG directly and rewrites every image referenced in `Package.appxmanifest`, at all required sizes and scales:

```
winapp manifest update-assets icon-sources/store-icon.svg
```

Regenerate the tray icon after changing `tray-icon.svg`, or after running the step above. This one needs `rsvg-convert` and ImageMagick, which are easiest to get from WSL (`sudo apt install librsvg2-bin imagemagick`):

```sh
for size in 16 24 32 48 256; do
  rsvg-convert -w "$size" -h "$size" icon-sources/tray-icon.svg -o "/tmp/$size.png"
done
convert /tmp/16.png /tmp/24.png /tmp/32.png /tmp/48.png /tmp/256.png \
  -background none -strip Assets/app.ico
```

Windows picks whichever of those five sizes fits the display scaling, so all of them need to be present. `-strip` keeps the output reproducible by dropping the timestamp ImageMagick would otherwise write into the 256px frame. Rebuild afterwards to embed the new icon, since `build.rs` only reruns when `Assets/app.ico` changes.

The committed assets were generated with winapp 0.5.0 and ImageMagick 6.9. Different versions rasterize the same SVG with slightly different antialiasing, so upgrading either tool rewrites every generated image even when no SVG changed. That is expected rather than a problem: regenerate the full set, commit it in one go, and note the new version here so the next person knows what produced them.

## Privacy

PS Battery collects nothing and sends nothing, because it has no network access at all. It does write a local diagnostic log you can read or delete yourself. See [PRIVACY.md](PRIVACY.md).

## Disclaimer

I am a frontend/backend web developer. I have no prior experience building Windows applications or writing Rust code. Neither do I know anything about the PlayStation controller specifications. This has only been tested using my own controllers on my own Windows installation.

## Any issues?

This has been an awesome weekend project. If you have any issues, feel free to open a GitHub issue or contact me. Otherwise, this code is unlicensed, so do whatever you want with it: https://unlicense.org/
