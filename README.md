# LX Remote (Tauri 版)

洛雪音乐（LX Music）桌面悬浮遥控窗。用 **Tauri 2**（系统 WebView2，无内置 Chromium）重写，体积从 Electron 版的 **272 MB 降到约 10 MB**，并原生支持系统托盘。

![认可linux.do](https://ld.xh.do/ld-badge.svg)

## 功能

- **上一首 / 下一首 / 播放暂停** 按钮
- **音量滑块**（拖拽实时生效，范围 0–100）
- **进度条**：点按位置跳转 + 拖拽定位，松手才 seek
- **歌词行**：拉取完整 LRC 时间轴，本地按播放进度即时切换（不依赖推送）；推送/轮询兜底
- **歌曲名 + 歌手 + 专辑** 展示
- **简洁模式**：隐藏歌词和歌手，歌曲名缩小，布局紧凑，窗口变矮（220 → 150）；状态记忆（刷新/重开不丢）
- **系统托盘**：左键单击切换窗口显隐；右键菜单（显示/隐藏、播放/暂停、上一首、下一首、音量±5、退出）
- **设置面板（窗口式）**：铺满窗口的独立面板，带标题栏和关闭按钮，弹出动画；打开时窗口自动增高到 330，关闭后恢复模式高度
- **鼠标穿透（智能）**：仅穿透无响应区域；悬停在按钮/滑块/标题栏等可交互组件上自动取消穿透，可正常点击
- **窗口置顶 / 不透明度（30–100%）** 设置
- **自动重连**：状态流断开 2 秒后重连 + 2 秒轮询兜底

> ⚠️ **技术要点**：LX Music 的 `/status` 返回的是 **NDJSON 长连接**（`Content-Type: application/json` + chunked，每行一个 JSON 对象），**不是标准 SSE**（`text/event-stream`）。所以不能直接用浏览器 `EventSource`（会一直报错），前端用 `fetch` + `ReadableStream` 按行解析 JSON 实现实时推送。

## 前置条件

1. **LX Music** 设置 → 网络 → openAPI：
   - ✅ 启用 openAPI
   - 端口 `23330`（默认）
   - （局域网遥控才需要 `bindLan: true`）
2. **重启 LX Music** 使设置生效

## 开发运行

```bash
cd D:/env/10/lx-remote-tauri/src-tauri
cargo run    # 注意：内存小的机器建议 -j 1 避免分页文件不足
```

## 构建发布版

```bash
cargo build --release -j 1
# 产物：target/release/lx-remote.exe
```

> 本机约束：Windows 分页文件较小，**必须用 `-j 1`** 编译，否则 `os error 1455`（paging file too small）。release 配置了 `lto + opt-level=s + strip`，体积更小，但编译时间更长。

## 目录结构

```
lx-remote-tauri/
├── ui/index.html          # 全部前端（单文件，含 CSS + JS）
├── src-tauri/
│   ├── Cargo.toml         # Rust 依赖
│   ├── tauri.conf.json    # 窗口/安全/打包配置
│   ├── capabilities/      # 权限清单
│   ├── icons/             # 应用图标 + 托盘图标（PIL 脚本生成）
│   └── src/
│       ├── main.rs        # 入口
│       └── lib.rs         # 托盘 + 窗口 + 透明度命令
└── _tools/gen_icons.py    # 图标生成脚本
```

## 与 Electron 版的差异

| | Electron 版 | Tauri 版 |
|---|---|---|
| 引擎 | 内置 Chromium | 系统 WebView2 |
| 体积 | 272 MB | ~10 MB（debug 234MB，release 更小） |
| 托盘 | 无 | ✅ 原生 |
| 透明窗口 | ✅ | ✅ |
| 不透明度 | `setOpacity` | Win32 `SetLayeredWindowAttributes`（自实现命令） |

## 已知限制

- `setOpacity` 在 Tauri 2.11 已被移除，透明度通过 Rust 命令调 Win32 API 实现；如果遇到无效需要重启应用
- 透明 + 置顶窗口在部分旧显卡驱动下可能有合成问题
- 未做安装包（bundle 配置了 nsis，需要时运行 `cargo tauri build`）
