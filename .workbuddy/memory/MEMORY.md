# raise-your-butt — 项目长期笔记

## 项目概述
Tauri 2 + React 19 + TS 桌面应用（久坐提醒）。前端单 SPA 按 window label 分支（settings / overlay）。Rust 后端模块：config / timer / shake / tray / commands。

## 技术栈关键版本
- Tauri 2.11.x，tauri-plugin-store 2.x，tauri-plugin-autostart 2.x，tauri-plugin-notification 2.x
- React 19，Vite 8，TS 5.6
- 配置持久化用 tauri-plugin-store（`settings.json`）

## Rust 反复踩的坑（务必警惕）

### 1. `app.state::<AppState>()` 返回临时 `State`，不能链式 `.field.lock()`
- **症状**：E0716 temporary value dropped while borrowed。`app.state::<AppState>().config.lock()` 里 guard 借用临时 `State`，语句结束即悬垂。
- **跨 await 更严重**：`tokio::select!` / async task 里的 future 借用临时 State 会要求 `'static`。
- **统一解法**：先 `let state = app.state::<AppState>();` 绑定，再访问字段。需要跨 await 且借的是 `Arc` 字段时，先 `let notify = state.timer.notify.clone();` 拿 owned `Arc`。
- **已出现位置**：timer.rs、shake.rs（同会话内踩 2 次）。

### 2. `store.get(key)` 返回 owned `JsonValue`，`as_str()` 会借用悬垂
- **症状**：E0515 returns a value referencing data owned by the current function。
- **解法**：用 `serde_json::from_value::<T>(v).ok()` 直接反序列化，复用类型的 `#[serde(...)]`。`as_u64()`/`as_bool()` 返回 owned 没问题，只有 `as_str()` 借用会坑。

### 3. `MonitorHandle::position()` 返回 `&PhysicalPosition`（借用）
- **症状**：E0597 borrowed value does not live long enough（move 进 `'static` 线程/任务闭包）。
- **解法**：`let base = *mon.position();` 解引用为 owned（`PhysicalPosition<i32>` 是 `Copy`）。`size()` 同理用 `*mon.size()`。

### 4. trait 方法必须 `use` trait 才能调用
- `state()` / `get_webview_window()` 等来自 `tauri::Manager`，需 `use tauri::Manager;`。`autolaunch()` 来自 `tauri_plugin_autostart::ManagerExt`，是另一个 trait。

## 约定
- shake 抖动：后端 `set_position` 移动 overlay 窗口做"窗口抖动" + 前端 CSS 卡片抖动，两者叠加。动画跑在 `tauri::async_runtime::spawn`（tokio）上，不用裸 OS 线程。
- overlay 关闭模式：Auto（到时自动隐藏）/ Manual（显示关闭按钮，调 `close_overlay` 命令）。
- 设置变更只在 `interval_min` 真正改变时才重置 timer 倒计时，避免调强度/overlay 设置时反复推迟提醒。
- settings 窗口 X 按钮 → `prevent_close` + `hide`，进程留托盘。
