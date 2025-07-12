# CrossCopy UI 架构

CrossCopy 采用 Tauri + React 架构实现跨平台 UI 界面，结合系统托盘功能，为用户提供直观、高效的剪贴板同步体验。

## 技术栈

- **前端框架**: React 18 + TypeScript
- **UI 组件库**: Ant Design
- **状态管理**: React Context API + Hooks
- **后端框架**: Tauri (Rust)
- **构建工具**: Vite
- **样式方案**: CSS Modules + SCSS
- **系统集成**: 系统托盘 + 原生通知

## 架构设计

### 1. Tauri + React 混合架构

CrossCopy 采用 Tauri 作为应用框架，结合 React 前端，实现了以下优势：

- **原生性能**: Tauri 使用系统原生 WebView，避免了 Electron 的资源开销
- **安全性**: Rust 后端提供内存安全和类型安全保障
- **跨平台**: 单一代码库支持 Windows、macOS、Linux 三大平台
- **现代化**: React 18 + TypeScript 提供现代化的前端开发体验

### 2. 系统托盘集成

系统托盘是 CrossCopy 的核心交互入口，提供：

- **常驻后台**: 应用可在后台持续运行，无需占用任务栏空间
- **快速访问**: 通过托盘菜单快速访问核心功能
- **状态指示**: 实时显示连接状态和同步状态
- **原生体验**: 符合各平台的系统托盘使用习惯

## 架构优势

1. **性能优化**
   - Tauri 使用系统原生 WebView，资源占用低
   - 冷启动时间 < 500ms，内存占用 < 50MB
   - 系统托盘常驻，主窗口按需显示

2. **跨平台兼容**
   - 支持 Windows 10+, macOS 10.15+, Linux (Ubuntu 18.04+)
   - 自适应 UI 设计，支持高分辨率显示器
   - 系统托盘在各平台表现一致

3. **Rust 集成**
   - 直接调用现有 CrossCopy 核心功能
   - 共享配置和状态管理
   - 事件驱动的通信模型
   - 高性能的网络通信和数据处理

4. **用户体验**
   - 系统托盘提供便捷的后台访问
   - 主窗口提供完整的功能界面
   - 支持热键快速操作
   - 原生通知系统集成

## 应用生命周期

### 启动流程

1. **应用初始化**
   - Tauri 运行时启动
   - 初始化 Rust 后端服务
   - 创建系统托盘

2. **前端加载**
   - 加载 React 应用
   - 初始化状态管理
   - 建立前后端通信

3. **服务启动**
   - 启动剪贴板监听
   - 初始化网络发现
   - 加载用户配置

### 窗口管理

- **主窗口**: 提供完整的功能界面，支持显示/隐藏切换
- **托盘窗口**: 轻量级快速访问界面（可选）
- **设置窗口**: 独立的设置配置界面
- **通知窗口**: 系统原生通知集成

### 退出策略

- **最小化到托盘**: 关闭主窗口时最小化到系统托盘
- **完全退出**: 通过托盘菜单或快捷键完全退出应用
- **优雅关闭**: 保存状态、断开连接、清理资源

## 技术实现细节

### 前后端通信

```typescript
// 前端调用后端命令
const result = await invoke('get_clipboard_history', { limit: 50 });

// 监听后端事件
await listen('clipboard-updated', (event) => {
  updateClipboardState(event.payload);
});
```

### 系统托盘实现

```rust
// 创建系统托盘
let tray = SystemTray::new()
    .with_menu(create_tray_menu())
    .with_tooltip("CrossCopy - 剪贴板同步工具");

// 处理托盘事件
fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            toggle_main_window(app);
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            handle_menu_click(app, &id);
        }
        _ => {}
    }
}
```

### 状态同步

- **单向数据流**: 后端状态变化通过事件通知前端
- **命令模式**: 前端通过命令调用后端功能
- **状态持久化**: 关键状态自动保存到本地存储