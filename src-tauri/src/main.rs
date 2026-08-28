// 入口：调用 lib 的 run()
// Tauri 2.x 标准模式：所有逻辑在 lib.rs，main.rs 只是个壳

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    lx_remote_lib::run();
}