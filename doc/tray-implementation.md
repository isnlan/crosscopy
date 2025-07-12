# 系统托盘实现

CrossCopy 的系统托盘功能是应用的核心交互入口，为用户提供便捷的后台访问和快速操作能力。通过 Tauri 的跨平台系统托盘 API，实现了在 Windows、macOS 和 Linux 上的一致体验。

## 设计理念

系统托盘设计遵循以下原则：

- **常驻可用**: 应用可在后台持续运行，随时响应用户操作
- **状态透明**: 通过图标和提示清晰显示当前状态
- **操作便捷**: 提供最常用功能的快速访问入口
- **平台一致**: 在不同操作系统上保持一致的用户体验

## 功能规格

### 1. 状态显示

- **连接状态指示**
  - 已连接: 绿色图标，显示连接设备数量
  - 未连接: 灰色图标，显示"未连接"状态
  - 连接中: 动画图标，显示"正在连接"状态
  - 错误状态: 红色图标，显示错误信息

- **同步状态指示**
  - 同步成功: 短暂显示成功提示
  - 同步失败: 显示错误提示和重试选项
  - 同步进行中: 显示进度指示

- **图标动态变化**
  - 支持不同状态的图标切换
  - 支持动画效果（连接中、同步中）
  - 支持系统主题适配（浅色/深色模式）

### 2. 快捷操作菜单

- **窗口管理**
  - 显示/隐藏主窗口
  - 置顶主窗口
  - 最小化到托盘

- **核心功能**
  - 查看剪贴板历史（快速预览）
  - 手动触发同步
  - 暂停/恢复剪贴板监听
  - 快速连接最近设备

- **设置和帮助**
  - 打开设置界面
  - 查看帮助文档
  - 关于信息
  - 退出应用

## 技术实现

### 1. 托盘创建和初始化

```rust
use tauri::{SystemTray, SystemTrayMenu, CustomMenuItem, MenuItem, SystemTrayEvent};

// 创建托盘菜单
fn create_tray_menu() -> SystemTrayMenu {
    SystemTrayMenu::new()
        // 主要操作
        .add_item(CustomMenuItem::new("show", "显示主窗口").accelerator("Cmd+Shift+C"))
        .add_item(CustomMenuItem::new("history", "剪贴板历史").accelerator("Cmd+Shift+H"))
        .add_native_item(MenuItem::Separator)

        // 同步控制
        .add_item(CustomMenuItem::new("sync_now", "立即同步"))
        .add_item(CustomMenuItem::new("pause", "暂停监听"))
        .add_native_item(MenuItem::Separator)

        // 设备管理
        .add_submenu(SystemTraySubmenu::new(
            "连接设备",
            SystemTrayMenu::new()
                .add_item(CustomMenuItem::new("scan_devices", "扫描设备"))
                .add_item(CustomMenuItem::new("device_list", "设备列表"))
        ))
        .add_native_item(MenuItem::Separator)

        // 设置和帮助
        .add_item(CustomMenuItem::new("settings", "设置"))
        .add_item(CustomMenuItem::new("about", "关于"))
        .add_native_item(MenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "退出"))
}

// 创建系统托盘
pub fn create_system_tray() -> SystemTray {
    SystemTray::new()
        .with_menu(create_tray_menu())
        .with_tooltip("CrossCopy - 剪贴板同步工具")
        .with_icon(get_tray_icon("default"))
}
```

### 2. 事件处理

```rust
// 托盘事件处理
pub fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { position, size, .. } => {
            // 左键点击 - 显示/隐藏主窗口
            toggle_main_window(app);
        }

        SystemTrayEvent::RightClick { position, size, .. } => {
            // 右键点击 - 显示上下文菜单（某些平台）
            show_context_menu(app, position);
        }

        SystemTrayEvent::MenuItemClick { id, .. } => {
            // 菜单项点击处理
            handle_menu_click(app, &id);
        }

        SystemTrayEvent::DoubleClick { position, size, .. } => {
            // 双击 - 打开主窗口
            show_main_window(app);
        }

        _ => {}
    }
}

// 菜单点击处理
fn handle_menu_click(app: &AppHandle, menu_id: &str) {
    match menu_id {
        "show" => toggle_main_window(app),
        "history" => show_clipboard_history(app),
        "sync_now" => trigger_manual_sync(app),
        "pause" => toggle_clipboard_monitoring(app),
        "scan_devices" => start_device_scan(app),
        "settings" => show_settings_window(app),
        "about" => show_about_dialog(app),
        "quit" => quit_application(app),
        _ => {}
    }
}
```

### 3. 状态管理

```rust
// 托盘状态管理
pub struct TrayState {
    pub connected_devices: usize,
    pub is_monitoring: bool,
    pub last_sync_status: SyncStatus,
    pub current_theme: Theme,
}

// 更新托盘图标和提示
pub fn update_tray_status(app: &AppHandle, state: &TrayState) {
    let icon = match (state.connected_devices, state.is_monitoring) {
        (0, _) => get_tray_icon("disconnected"),
        (_, false) => get_tray_icon("paused"),
        (n, true) => get_tray_icon("connected"),
    };

    let tooltip = format!(
        "CrossCopy - {} 设备连接{}",
        state.connected_devices,
        if state.is_monitoring { "" } else { " (已暂停)" }
    );

    app.tray_handle()
        .set_icon(icon)
        .and_then(|_| app.tray_handle().set_tooltip(&tooltip))
        .unwrap_or_else(|e| eprintln!("Failed to update tray: {}", e));
}

// 动态更新菜单项
pub fn update_tray_menu(app: &AppHandle, state: &TrayState) {
    let pause_text = if state.is_monitoring { "暂停监听" } else { "恢复监听" };

    app.tray_handle()
        .get_item("pause")
        .set_title(pause_text)
        .unwrap_or_else(|e| eprintln!("Failed to update menu: {}", e));
}
```

### 4. 图标资源管理

```rust
// 图标资源管理
fn get_tray_icon(status: &str) -> tauri::Icon {
    let icon_path = match status {
        "connected" => "icons/tray-connected.png",
        "disconnected" => "icons/tray-disconnected.png",
        "paused" => "icons/tray-paused.png",
        "syncing" => "icons/tray-syncing.png",
        "error" => "icons/tray-error.png",
        _ => "icons/tray-default.png",
    };

    // 支持系统主题适配
    let theme_suffix = if is_dark_mode() { "-dark" } else { "-light" };
    let themed_path = icon_path.replace(".png", &format!("{}.png", theme_suffix));

    tauri::Icon::File(std::path::PathBuf::from(themed_path))
}
```

## 平台特定实现

### Windows 平台

- **交互方式**: 左键点击显示菜单，右键点击显示主窗口
- **图标要求**: 16x16 像素 ICO 格式，支持透明背景
- **通知集成**: 支持 Windows 10/11 原生通知
- **任务栏集成**: 支持任务栏图标闪烁提醒

### macOS 平台

- **交互方式**: 点击显示菜单，支持拖拽操作
- **图标要求**: 16x16 像素 PNG 格式，自动适配 Retina 显示器
- **主题适配**: 自动适配浅色/深色模式
- **菜单栏集成**: 符合 macOS 菜单栏设计规范

### Linux 平台

- **桌面环境支持**:
  - GNOME: 通过 AppIndicator 实现
  - KDE: 通过 StatusNotifierItem 实现
  - XFCE/其他: 通过 GtkStatusIcon 实现
- **图标要求**: 22x22 像素 PNG 格式，支持 SVG
- **主题集成**: 自动适配系统图标主题

## 用户体验优化

### 1. 响应性能

- **快速响应**: 托盘点击响应时间 < 100ms
- **异步操作**: 耗时操作在后台执行，避免界面卡顿
- **状态缓存**: 缓存常用状态信息，减少查询延迟

### 2. 视觉反馈

- **状态指示**: 通过图标颜色和形状直观显示状态
- **动画效果**: 连接和同步过程显示动画反馈
- **通知提醒**: 重要事件通过系统通知提醒用户

### 3. 可访问性

- **键盘导航**: 支持键盘快捷键操作
- **屏幕阅读器**: 提供适当的可访问性标签
- **高对比度**: 支持系统高对比度模式