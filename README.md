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
4. Works with the DualSense and DualSense Edge (PS5), which are the ones I own and test with. DualShock 4 (PS4) controllers are recognised too, but I have no way to try them, so that support is untested.

<img src="./images/notification_20.png" alt="Info balloon at 20% battery" width="400" />

<img src="./images/notification_10.png" alt="Warning balloon at 10% battery" width="400" />

<img src="./images/notification_0.png" alt="Error balloon at 0% battery" width="400" />

<img src="./images/tray.png" alt="Tray menu showing connected controllers" width="400" />

## Why the percentage is a multiple of 10

PlayStation controllers report their battery as a 0-10 level, so the real charge is somewhere in a 10% range. The common approach, used by the Linux `hid-playstation` driver among others, is to report the middle of that range (level × 10 + 5)%. PS Battery shows the lower end of that range (level × 10)% because I would rather show too little remaining battery than too much. And round numbers are just prettier to look at in my opinion.

## Download

Two options, same app:

- **[GitHub releases](https://github.com/JorSanders/ps-battery/releases/latest)**: download `ps-battery.exe`. The exe is unsigned (a code signing certificate costs a few hundred euros a year, and this is a free hobby project), so Windows shows an "Unknown publisher" warning. Every release also includes `ps-battery.exe.bundle`, a Sigstore signature created by the release workflow, so you can verify the exe was built by this repository's CI using [cosign](https://docs.sigstore.dev/cosign/system_config/installation/):

  ```powershell
  cosign verify-blob ps-battery.exe --bundle ps-battery.exe.bundle --certificate-identity-regexp "^https://github.com/JorSanders/ps-battery/" --certificate-oidc-issuer https://token.actions.githubusercontent.com
  ```

- **Microsoft Store**: submitted and currently under review. Once it passes certification the app is downloadable there without any warning, since the Store signs the package itself.

## Local release build

Requires the [winapp CLI](https://github.com/microsoft/winappCli) (`winget install microsoft.winappcli`); `winapp run` needs Windows Developer Mode.

Build, then run with package identity:

```powershell
cargo build --release --locked
winapp run .\target\release
```

Test that the MSIX packs. The output is not runnable because it uses an untrusted throwaway certificate. For Microsoft Store uploads I use the CI workflow artifact:

```powershell
cargo build --release --locked
mkdir dist -Force | Out-Null
copy target\release\ps-battery.exe dist\
winapp pack ./dist --manifest Package.appxmanifest --generate-cert
```

## Regenerating icons

`icon-sources\` holds the sources; everything in `Assets\` is generated. Run both steps in this order, because the first also overwrites `Assets/app.ico` with a badged icon the tray should not use:

The Store PNGs, after changing `store-icon.svg`:

```powershell
winapp manifest update-assets icon-sources/store-icon.svg
```

The tray icon, from WSL (`sudo apt install librsvg2-bin imagemagick`):

```sh
for size in 16 24 32 48 256; do
  rsvg-convert -w "$size" -h "$size" icon-sources/tray-icon.svg -o "/tmp/$size.png"
done
convert /tmp/16.png /tmp/24.png /tmp/32.png /tmp/48.png /tmp/256.png \
  -background none -strip Assets/app.ico
```

Rebuild afterwards so `build.rs` embeds the new `app.ico`.

## Privacy

PS Battery collects nothing and sends nothing, because it has no network access at all. It does write a local diagnostic log you can read or delete yourself. See [PRIVACY.md](PRIVACY.md).

## Disclaimer

I am a frontend/backend web developer. I have no prior experience building Windows applications or writing Rust code. Neither do I know anything about the PlayStation controller specifications. This has only been tested using my own controllers on my own Windows installation.

## Any issues?

This is an awesome hobby project I have spent quite some hours on. If you have any issues, feel free to open a GitHub issue or contact me. Otherwise, this code is unlicensed, so do whatever you want with it: https://unlicense.org/
