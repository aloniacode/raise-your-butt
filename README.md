# raise-your-butt · 久坐提醒

Tauri 2 桌面应用 — 系统托盘常驻，按设定间隔触发「系统通知 + 全屏覆盖层屏幕抖动」提醒你起身活动。

## 功能

- 🖥️ **常驻系统托盘** — 启动后无主窗口，右下角托盘可见。
- 🪟 **托盘左键打开设置** — 自定义弹窗包含：提醒间隔（分钟）、抖动强度滑块（1–10）、提醒窗口关闭方式（自动 / 手动）、自动模式下的显示时长（2–30 秒）、开机自启动开关，并提供「测试抖动」按钮。
- 💾 **设置自动持久化** — 通过 `tauri-plugin-store` 写入 `%APPDATA%` 下的 `settings.json`，重启后保留。
- ⏰ **后台倒计时** — Rust tokio 任务到点触发；修改间隔立即重置（不需等当前周期走完）。
- 🔔 **系统通知** — 「该起身活动啦！久坐伤身，起来走走吧」。
- 💥 **屏幕抖动** — 通过 `tauri` 原生 `WebviewWindow::set_position` 在一个全屏透明、置顶、跳过任务栏的「覆盖层窗口」上跑 36 步阻尼正弦位移（≈ 800ms），叠加覆盖层卡片自身的 CSS 抖动；抖动循环跑在 `tauri::async_runtime`（tokio）任务上，不占用独立 OS 线程。
- 🚪 **提醒窗口关闭模式** — **自动模式**：抖动结束后按设定时长（默认 5 秒）自动隐藏覆盖层；**手动模式**：覆盖层不自动消失，显示「我知道了」关闭按钮，点击后才隐藏。
- 🚀 **开机自启动** — 通过 `tauri-plugin-autostart` 注册 / 注销 Windows Run-key。
- 🎨 **铅笔画风格图标** — 白底黑线的小人从椅子上起身 + 向上箭头，传达「起身活动」。

## 技术栈

- **Tauri 2** (Windows 目标)
- **React 19 + TypeScript + Vite 8**
- **Rust 异步**：tokio（`select!` + `Notify` 实现可重置定时器）
- 插件：`tauri-plugin-autostart`、`tauri-plugin-store`、`tauri-plugin-notification`

## 项目结构

```
raise-your-butt/
├── index.html
├── package.json
├── tsconfig.json / tsconfig.node.json
├── vite.config.ts
├── src/                        # React 前端（单 SPA 双窗口）
│   ├── main.tsx
│   ├── App.tsx                 # 按 window label 分支
│   ├── Settings.tsx            # 设置弹窗 UI
│   ├── Overlay.tsx             # 全屏覆盖层 UI（监听 shake-start）
│   └── styles.css
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json         # 双窗口配置（settings + overlay）
│   ├── capabilities/default.json
│   ├── icons/
│   │   └── A_minimalist_pencil_drawing_il_2026-07-24T10-00-32.png
│   └── src/
│       ├── main.rs             # 薄入口
│       ├── lib.rs              # Tauri 装配：插件、状态、托盘、窗口事件钩子
│       ├── config.rs           # AppConfig + 通过 store 加载/保存
│       ├── timer.rs            # 后台倒计时 + Notify 重置
│       ├── shake.rs            # 覆盖层显示 + 异步 set_position 抖动循环 + 按模式自动/手动隐藏
│       ├── tray.rs             # 托盘图标 + 右键菜单 + 左键唤出设置
│       └── commands.rs         # get_config / set_config / trigger_shake / test_shake / close_overlay
└── README.md
```

## 前置要求（一次性安装）

| 工具         | 安装命令                                                                       |
| ------------ | ------------------------------------------------------------------------------ |
| Rust (stable)| `winget install Rustlang.Rustup`                                               |
| MSVC Build Tools（含 Windows SDK / C++ 桌面）| `winget install Microsoft.VisualStudio.2022.BuildTools` 并勾选 *Desktop development with C++* |
| WebView2 Runtime | Windows 11 / Win10 已自带，旧系统需手动安装                          |
| Node.js      | Node ≥ 18（本项目用 pnpm）                                                     |
| pnpm         | `npm i -g pnpm`                                                                |

> Rust 最低推荐版本 ≥ 1.77.2（`tauri-plugin-*` 的最低要求）。

## 启动开发模式

```powershell
cd D:\WorkSpace\raise-your-butt

# 第一次：安装前端依赖
pnpm install

# 启动 Tauri 开发（首次会编译 Rust crate，约 2–5 分钟，之后增量编译极快）
pnpm tauri dev
```

运行后将看到：

1. 编译 Rust + 启动 Vite，**没有任何窗口弹出** —— 这是预期的，正常。
2. 看右下角任务栏托盘，多了一个「久坐提醒」小图标（铅笔画风）。
3. **左键点击**托盘图标 → 弹出设置窗口 → 修改任意设置会自动保存。
4. 点 **「测试抖动」** 立刻看到全屏抖动 + 系统通知。
5. **右键**托盘 → 设置 / 退出。
6. 关掉设置窗口（点 X）不会退出进程，只是隐藏。

## 构建发布版本（生成安装器）

首次构建前建议先生成完整的图标集（`tauri icon` 需要源 PNG）：

```powershell
# 可选：把生成图标复制到项目根当源（更整洁，可省略）
copy "src-tauri\icons\A_minimalist_pencil_drawing_il_2026-07-24T10-00-32.png" "app-icon.png"

# 生成标准图标集（32.png / 128.png / 128@2x.png / icon.ico / icon.icns ...）
pnpm tauri icon ./app-icon.png
```

然后把 `tauri.conf.json` 的 `bundle.icon` 数组改为完整列表（即模板默认那几个）。最后：

```powershell
# 产出在 src-tauri\target\release\bundle\msi\ 和 nsis\
pnpm tauri build
```

> 首次 `tauri build` 还会下载 WiX / NSIS 工具链，期间确保网络畅通。

## 故障排查

- **编译报错 `tauri-plugin-store … undefined method store`**：不同 2.x 小版本 API 微调，本项目用的是 `use tauri_plugin_store::StoreExt; app.store("settings.json")` 这种经典写法；如果 cargo 拉到了更新的破坏性版本，请固定到 `tauri-plugin-store = "=2.2.0"`（或打开 `Cargo.lock` 看实际版本）。
- **`MacosLauncher::LaunchAgent` 报错**：把 `src-tauri\src\lib.rs` 里改成对应版本里有的枚举变体；Windows 下此参数完全忽略。
- **托盘上图标是一片白色方块或很糊**：`pnpm tauri icon ./app-icon.png` 重新生成多尺寸；16×16 / 32×32 自动缩放通常是清晰的。
- **关掉设置窗口后进程真的退了**：确保 `src-tauri\src\lib.rs` 的 `setup()` 里 `on_window_event(CloseRequested)` 拦截块存在；本项目已包含。
- **`Error: permission X not found`**：打开警告中提示的核心 capability 名加进 `src-tauri\capabilities\default.json` 的 `permissions` 数组里。

## 设计备注

- **为什么用覆盖层而不是移动真实窗口？**

  按需求，移动其它用户的窗口风险高（窗口可能移出可见区域、最大化窗口不可被 `SetWindowPos` 移动）；覆盖层干净可逆。
- **抖动为什么用 `set_position` 而不是 Win32 `SetWindowPos`？**

  纯 Tauri 2 API 路线（底层仍是 Win32），可编译性高、跨平台友好；若需要更激进的抖动可加 `windows` + `raw-window-handle` crate 改为直接调 `SetWindowPos`。
- **抖动循环为什么跑在 tokio 而不是独立线程？**

  36 帧、每帧 `sleep` ≈22ms 的动画用 `tauri::async_runtime::spawn` + `tokio::time::sleep` 协作调度，帧间让出执行权，避免为一个 800ms 动画独占一整个 OS 线程。`set_position` 是快速同步窗口调用，在 async 任务里直接调用开销可忽略。
- **为什么关窗口不退出进程？**

  常驻托盘应用 — `CloseRequested` 触发 `prevent_close()` + `hide()`，进程继续运行以维持定时器和托盘图标。

## License

MIT
