---
name: tauri-tray-app
description: Build a tray-resident Tauri 2 desktop application with a health-driven tray icon, a transient popover for at-a-glance status, and a frameless dashboard window with a custom title bar. Use when scaffolding a fleet operator console, a substrate supervisor, or any app that lives in the menu bar and watches background services rather than owning a dock icon. Triggers on tray app, system tray, menu bar app, LSUIElement, accessory activation policy, popover, health aggregator, frameless window, Tauri 2 tray, TrayIconBuilder, tray icon state.
---
<!-- TJ-ARCH-MOB-001 compliant -->

# Tray-Resident Tauri App

A tray app lives in the menu bar, supervises services, and opens a dashboard on
demand. It does not appear in the app switcher and closing its window does not
quit it. Scaffold with `scripts/scaffold-tauri-tray.mjs <project-root>`.

## The three surfaces

| Surface | Properties | Role |
|---|---|---|
| Tray icon | Template image, color driven by aggregated health | Always-visible state |
| Popover | Frameless, transparent, always-on-top, skips taskbar, closes on blur or Escape | At-a-glance status |
| Dashboard | Frameless with a custom title bar, normal window controls | Full interaction |

## Tray icon states

| State | Condition |
|---|---|
| Healthy | Every required service is up and ready |
| Degraded | At least one service is degraded |
| Down | At least one required service has been down past the grace window |
| Paused | Supervision deliberately paused by the operator |
| Starting | Substrate still initializing |

Derive the state in one aggregator and let the icon render it. An icon computed
at the call site drifts from the popover the moment either changes.

## Building the tray (Tauri 2)

```rust
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

let show = MenuItem::with_id(app, "show", "Show dashboard", true, None::<&str>)?;
let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
let menu = Menu::with_items(app, &[&show, &PredefinedMenuItem::separator(app)?, &quit])?;

TrayIconBuilder::with_id("substrate-tray")
    .icon(app.default_window_icon().unwrap().clone())
    .icon_as_template(true)
    .menu(&menu)
    .show_menu_on_left_click(false)
    .on_menu_event(|app, event| match event.id.as_ref() {
        "show" => show_dashboard(app),
        "quit" => app.exit(0),
        _ => {}
    })
    .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            toggle_popover(tray.app_handle());
        }
    })
    .build(app)?;
```

`show_menu_on_left_click(false)` is required. Tauri 2 shows the menu on both
buttons by default, so without it the left-click popover never appears — the
menu opens instead and the handler looks broken.

`icon_as_template(true)` lets macOS tint the icon for light and dark menu bars.
A non-template icon renders as a fixed-color blob in one of the two.

## Staying out of the app switcher

On macOS set the accessory activation policy so the app has no dock icon and no
Cmd+Tab entry:

```rust
#[cfg(target_os = "macos")]
app.set_activation_policy(tauri::ActivationPolicy::Accessory);
```

Then intercept window close so the dashboard hides instead of exiting. Without
that intercept, closing the dashboard on a tray app quits the supervisor —
services stop and the operator has no idea why.

## The health aggregator

One owner of health state, fed by probes, read by the tray and the popover:

- Probes report per-service liveness and readiness on a fixed interval.
- The aggregator folds them into a single state plus a per-service list.
- A service must stay down past a grace window before it counts as down, so a
  restart does not flash the icon red.
- The tray subscribes; it does not poll services directly.

Keep the aggregator behind a channel or a lock, never in UI state. The UI is a
projection of health, not its source.

## Verification

- The app shows no dock icon and no app-switcher entry.
- Left click opens the popover; right click opens the menu.
- Closing the dashboard hides the window and leaves services running.
- Stopping a supervised service turns the icon to its down state after the
  grace window, and recovery clears it.
- The tray icon reads correctly in both light and dark menu bars.

Run the `tauri-ui-review` skill against the dashboard, and `a11y-gate` before
calling any surface done.
