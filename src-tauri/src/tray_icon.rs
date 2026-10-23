use std::process;

use objc2_app_kit::NSWindowCollectionBehavior;
use objc2_foundation::{NSOperatingSystemVersion, NSProcessInfo};
use tauri::{
    menu::{Menu, MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    ActivationPolicy, Manager, Runtime,
};
use tauri_plugin_nspopover::{AppExt, WindowExt as _};
use tauri_specta::Event;

use crate::{event::PowerUpdatedEvent, ext::WebviewWindowExt};

pub fn setup_tray_icon<R: Runtime>(app: &impl Manager<R>) -> tauri::Result<()> {
    let supports_joining_applications = unsafe {
        NSProcessInfo::processInfo().isOperatingSystemAtLeastVersion(NSOperatingSystemVersion {
            majorVersion: 13,
            minorVersion: 0,
            patchVersion: 0,
        })
    };
    let show = MenuItemBuilder::new("Show Window").build(app)?;
    let quit = MenuItemBuilder::new("Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .separator()
        .item(&quit)
        .build()
        .unwrap();

    let tray_icon = TrayIconBuilder::with_id("main")
        .title("0 w")
        .menu_on_left_click(false)
        .build(app)
        .unwrap();

    tray_icon.on_menu_event(move |tray_handle, event| match event.id() {
        val if val == show.id() => {
            let (window, _) = tray_handle
                .app_handle()
                .get_or_create_window("main")
                .unwrap();

            if !window.is_visible().unwrap() {
                window.show().unwrap();
                window.set_focus().unwrap();

                tray_handle
                    .app_handle()
                    .set_activation_policy(ActivationPolicy::Regular)
                    .unwrap();
            }
        }
        val if val == quit.id() => {
            tray_handle.app_handle().cleanup_before_exit();
            process::exit(0);
        }
        _ => {}
    });

    tray_icon.on_tray_icon_event(move |tray_handle, event| {
        tauri_plugin_positioner::on_tray_event(tray_handle.app_handle(), &event);
        if let TrayIconEvent::Click {
            button,
            button_state,
            ..
        } = event
        {
            let handle = tray_handle.app_handle();
            match (button, button_state) {
                (MouseButton::Left, MouseButtonState::Up) => {
                    if handle.is_popover_shown() {
                        handle.hide_popover();
                    } else {
                        handle.show_popover();
                        // The plugin moves the webview into a separate native
                        // window. Configure that window, rather than its hidden
                        // Tauri host, to overlay other apps' fullscreen Spaces.
                        let window = unsafe { handle.ns_popover().contentViewController() }
                            .and_then(|controller| unsafe { controller.view().window() });
                        if let Some(window) = window {
                            unsafe {
                                let mut behavior = window.collectionBehavior();
                                behavior.remove(
                                    NSWindowCollectionBehavior::FullScreenPrimary
                                        | NSWindowCollectionBehavior::FullScreenNone
                                        | NSWindowCollectionBehavior::MoveToActiveSpace,
                                );
                                behavior.insert(
                                    NSWindowCollectionBehavior::CanJoinAllSpaces
                                        | NSWindowCollectionBehavior::FullScreenAuxiliary,
                                );
                                if supports_joining_applications {
                                    behavior.remove(
                                        NSWindowCollectionBehavior::Primary
                                            | NSWindowCollectionBehavior::Auxiliary,
                                    );
                                    behavior
                                        .insert(NSWindowCollectionBehavior::CanJoinAllApplications);
                                }
                                window.setCollectionBehavior(behavior);
                                window.orderFrontRegardless();
                            }
                        }
                    }
                }
                (MouseButton::Right, MouseButtonState::Down) => {
                    handle.hide_popover();
                    // Let the status item present its own menu in the current
                    // Space, including over fullscreen apps. Attach it only
                    // while tracking so left-click still opens the popover.
                    if let Err(error) = tray_handle.set_menu(Some(menu.clone())) {
                        log::warn!("failed to attach tray context menu: {error}");
                        return;
                    }
                    unsafe {
                        handle.ns_statusbar_button().performClick(None);
                    }
                    if let Err(error) = tray_handle.set_menu(None::<Menu<R>>) {
                        log::warn!("failed to detach tray context menu: {error}");
                    }
                }
                _ => {}
            }
        }
    });

    PowerUpdatedEvent::listen(app.app_handle(), move |event| {
        tray_icon.set_title(Some(event.payload.0)).unwrap();
    });

    app.popover_window().unwrap().to_popover();

    Ok(())
}
