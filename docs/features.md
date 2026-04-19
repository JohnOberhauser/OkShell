# Features

OkShell has many built-in features.

### The Frame

The frame is drawn around the outer edge of the screen and contains bars and menus.
The frame can be hidden if you would rather the bars not appear to be connected to each other.

### Bars

Bars can be configured with various widgets.
You can have a top, bottom, right and/or left bar.

::: details Available bar widgets
Audio Input  
Audio Output  
Battery  
Bluetooth  
Clipboard  
Clock  
Hyprland Dock  
Hyprland Workspaces  
HyprPicker  
Lock  
Logout  
Network  
Notifications  
Power Profile  
Quick Settings  
Reboot  
Recording Indicator  
Screenshot  
Shutdown  
Tray  
Vpn Indicator  
Wallpaper  
:::

### Menus

There are various menus that can be opened through `okshellctl` or by click bar widgets.

::: details Available menus
App Launcher  
Clock  
Clipboard  
Quick Settings  
Notifications  
Screen Share  
Screenshot  
Wallpaper  
:::

Each menu's widgets can be customized, and the layouts can be changed.

::: details Available menu widgets
App Launcher  
Audio Input  
Audio Output  
Bluetooth  
Calendar  
Clipboard  
Clock  
Container  
Divider  
Media Player  
Network  
Notifications  
PowerProfiles  
Quick Action: Airplane Mode  
Quick Action: Do Not Disturb  
Quick Action: HyprPicker  
Quick Action: Idle Inhibitor  
Quick Action: Lock  
Quick Action: Logout  
Quick Action: Nightlight  
Quick Action: Reboot  
Quick Action: Settings  
Quick Action: Shutdown  
Screenshots  
Screen Recording  
Spacer  
ThemePicker    
Wallpaper  
Weather  
:::

### Notifications

OkShell manages notifications for you.

### OSD

On screen display for volume and brightness.

### Wallpaper Management

OkShell manages wallpaper for you.
If your theme is set to Matugen, then your theme will change when you change your wallpaper.

### Lock Screen

The lockscreen shares the same wallpaper as your desktop.
I recommend setting session lock xray to true in your hyprland config for a smooth animation when locking and unlocking.

```
misc {
    session_lock_xray = true
}
```

### Polkit Agent

Okshell comes with a polkit agent.
