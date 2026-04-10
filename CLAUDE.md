# CLAUDE.md

## Project overview

A Windows system-tray app written in Rust that monitors PlayStation controller battery levels over HID and fires Windows balloon notifications before a controller runs out of charge.

## Build & run

```
cargo build            # debug
cargo build --release  # release
cargo check            # type-check only (fast)
cargo run              # debug run (shows a console window)
```

The release binary uses `#![windows_subsystem = "windows"]` so it runs without a console window.

## Key architecture

| Module | Responsibility |
|--------|---------------|
| `main.rs` | Spawns the polling thread; runs the Win32 message loop on the main thread |
| `ps_battery/logger.rs` | File logger (`%APPDATA%\ps-battery\ps-battery.log`); exposes `log_info!` and `log_err!` macros used everywhere |
| `ps_battery/poll_controllers.rs` | Calls `get_playstation_controllers`, reads each HID report, parses battery state, stores results; exports `request_poll` (wake condvar early) and `wait_for_next_poll` (60 s condvar sleep) |
| `ps_battery/get_playstation_controllers.rs` | Filters `hidapi` device list to Sony vendor + known product IDs |
| `ps_battery/parse_battery_and_charging.rs` | Extracts battery level and charging state from the raw HID input report byte |
| `ps_battery/read_controller_input_report.rs` | Opens HID device, reads input report, handles truncated Bluetooth header by sending a feature report first |
| `ps_battery/send_controller_feature_report.rs` | Sends the feature report needed to get full Bluetooth reports from DualSense |
| `ps_battery/controller_store.rs` | Global `RwLock<Vec<ControllerStatus>>` shared between the polling thread and the tray/alert code |
| `ps_battery/send_controller_alerts.rs` | Fires balloon notifications + system sounds when a Bluetooth controller is ≤30 % and not charging |
| `ps_battery/play_sound.rs` | Plays Windows system-sound aliases via `PlaySoundW` with `SND_ALIAS` |
| `ps_battery/tray/` | Tray icon, hidden Win32 window, right-click menu, autostart registry key, balloon helper |

## HID report format

- **USB DualSense / Edge**: 64-byte report, battery byte at index 53
- **Bluetooth DualSense / Edge**: 78-byte report (after feature-report handshake), battery byte at index 54
- **DualShock 4 (gen 1 & 2)**: battery byte at index 29

Battery byte layout: `[state nibble (high) | level nibble (low)]`
- level nibble × 10 = battery percent (0–100)
- state bit 0 = charging, state bit 1 = fully charged

Bluetooth is detected by checking whether the HID path contains the Bluetooth GUID `00001124-0000-1000-8000-00805F9B34FB`.

## Tray menu items

- Controller status lines (greyed, read-only)
- **Scan for controllers now** — calls `request_poll()` to wake the polling thread immediately
- **Run on startup** — toggles `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
- **Open log** — opens the log file with `ShellExecuteW`; greyed out if no log file exists
- **Exit** — removes the tray icon via `Shell_NotifyIconW(NIM_DELETE)` then posts `WM_QUIT`

## Alerts

Alerts fire every 5 minutes (`ALERT_INTERVAL`) only for **Bluetooth** controllers that are not charging and not fully charged:
- ≤10 % → Critical Stop sound + error balloon
- ≤20 % → Exclamation sound + warning balloon
- ≤30 % → Notification sound + info balloon

USB-connected controllers are intentionally excluded from alerts.
