# Privacy Policy for PS Battery

Last updated: 7 August 2026

## Summary

PS Battery does not collect, transmit, or share any data. It has no network
access, no analytics, no telemetry, and no accounts. Everything it reads stays
on your own computer.

## What the app reads

To report battery levels, PS Battery reads from PlayStation controllers
connected to your computer over USB or Bluetooth. For each controller it reads
the product name, the USB vendor and product ID, the device path Windows
assigns it, and the battery status report the controller returns.

It does not read anything from other USB or Bluetooth devices, and it does not
read your files, input, or any other information about you or your computer.

## What the app stores

PS Battery writes a diagnostic log to your own machine, at:

```
%APPDATA%\ps-battery\ps-battery.log
%APPDATA%\ps-battery\ps-battery.old.log
```

The log records the controller information described above, including the raw
battery report bytes and the Windows device path, which contains a
device-instance identifier for the controller. The log is overwritten each time
the app starts, and the previous run's log is kept as `ps-battery.old.log`.

These files are never sent anywhere. You can open the current one from the tray
menu ("Open log"), and you can delete either file at any time.

To start with Windows, the app records that preference. The version installed
from the Microsoft Store registers a startup task with Windows, which you can
turn off under Settings → Apps → Startup. The standalone `.exe` version instead
stores the path to the program under
`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, which is removed when you
turn the option off.

## What the app sends

Nothing. PS Battery contains no networking code and no network-capable
dependencies, so it cannot transmit data even accidentally. You are welcome to
verify this: the source code is public at
https://github.com/JorSanders/ps-battery

## Contact

For questions about this policy, please open an issue at
https://github.com/JorSanders/ps-battery/issues
