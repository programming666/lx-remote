// LX Remote - Tauri 后端
// 职责：透明窗口 + 系统托盘 + 关闭最小化到托盘 + 接收托盘菜单事件转发给前端

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WindowEvent,
};

/// 设置窗口整体透明度（Tauri 2.x 移除了 setOpacity，这里直接用 Win32 API）
#[tauri::command]
fn set_window_opacity(window: tauri::Window, opacity: f64) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, SetLayeredWindowAttributes, GWL_EXSTYLE, LWA_ALPHA,
        WS_EX_LAYERED,
    };
    // tauri 的 HWND 是 windows crate 的包装类型，转成 windows-sys 期望的裸指针
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as *mut core::ffi::c_void;
    let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
    unsafe {
        // 确保窗口带 WS_EX_LAYERED 扩展样式
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        if style & WS_EX_LAYERED as isize == 0 {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_LAYERED as isize);
        }
        SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA);
    }
    Ok(())
}

/// 读取系统光标位置，返回窗口客户区内的物理像素坐标（前端除以 dpr 得 CSS 坐标）
#[tauri::command]
fn get_cursor_pos(window: tauri::Window) -> Result<(f64, f64), String> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetPhysicalCursorPos;
    let mut pt = POINT { x: 0, y: 0 };
    unsafe { GetPhysicalCursorPos(&mut pt) };
    // inner_position 与 JS innerPosition 同源，同为物理像素；GetPhysicalCursorPos 恒为物理像素
    let origin = window.inner_position().map_err(|e| e.to_string())?;
    Ok((pt.x as f64 - origin.x as f64, pt.y as f64 - origin.y as f64))
}


/// 切换主窗口的显示/隐藏
fn toggle_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let visible = win.is_visible().unwrap_or(false);
        if visible {
            let _ = win.hide();
        } else {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

/// 处理托盘菜单事件：大部分命令通过事件转发给前端（前端已实现 HTTP 调用）
fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "show" => toggle_window(app),
        "quit" => {
            // 真的退出整个进程
            app.exit(0);
        }
        // 其余命令：转发事件给前端，由前端调用 LX Music HTTP API
        other => {
            let _ = app.emit_to("main", "tray-cmd", other.to_string());
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![set_window_opacity, get_cursor_pos])
        .setup(|app| {
            // ---- 构建托盘菜单 ----
            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "show", "显示 / 隐藏窗口", true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, "play", "播放 / 暂停", true, None::<&str>)?,
                    &MenuItem::with_id(app, "prev", "上一首", true, None::<&str>)?,
                    &MenuItem::with_id(app, "next", "下一首", true, None::<&str>)?,
                    &MenuItem::with_id(app, "vol-up", "音量 +5", true, None::<&str>)?,
                    &MenuItem::with_id(app, "vol-down", "音量 -5", true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?,
                ],
            )?;

            // ---- 创建托盘图标 ----
            let _tray = TrayIconBuilder::with_id("lx-remote-tray")
                .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
                    // 兜底：若没设置 icon，至少创建一个空图标
                    tauri::image::Image::new_owned(vec![0; 4], 1, 1)
                }))
                .tooltip("LX Remote")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    handle_menu_event(app, event.id.as_ref());
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键单击切换窗口
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // 拦截主窗口关闭：最小化到托盘而不是退出
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}