# ✨ CrossCopy (Rust Edition)

> 一个用 Rust 编写的跨平台剪贴板同步工具，支持多设备间实时同步文本/图片内容，安全、轻量、无侵入。


## 📌 项目简介

**CrossCopy** 是一款支持跨平台（Windows / macOS / Linux）的剪贴板同步工具。通过局域网或远程通信，使多个设备间实现**复制即同步**的体验，类似 Apple 的 Handoff 功能，但面向所有平台，开源、可定制、可拓展。


## 🚀 特性功能

- 🖥️ **跨平台支持**：macOS、Windows、Linux（桌面端）
- 🔄 **剪贴板同步**：实时监听并同步文本、图片等内容
- 🌐 **局域网通信**：基于 WebSocket / TCP 实现设备间直连
- 🔐 **端到端加密**：内容传输全程加密，保障数据隐私
- 🔧 **本地配置文件**：设备名称、配对密钥、自定义端口等
- 📜 **日志输出**：可调级别的日志便于调试和监控
- 🧠 **智能去抖动**：避免内容重复、频繁同步


## 🧰 技术栈

- **语言**：Rust
- **异步框架**：[`tokio`](https://tokio.rs/)
- **剪贴板访问**：[`arboard`](https://crates.io/crates/arboard)
- **WebSocket通信**：[`tokio-tungstenite`](https://crates.io/crates/tokio-tungstenite)
- **加密支持**：`aes-gcm`
- **配置管理**：`confy`
- **序列化**：`serde` + `serde_json`


## 🛠️ 快速开始

### 1️⃣ 安装依赖

确保你已安装 Rust 工具链（推荐使用 [rustup](https://rustup.rs)）：

```bash
rustup update
````

### 2️⃣ 构建项目

```bash
git clone https://github.com/isnlan/crosscopy.git
cd crosscopy
cargo build --release
```

### 3️⃣ 运行程序

```bash
cargo run --release
```

首次运行将自动在 `$HOME/.config/crosscopy/` 下生成默认配置文件。


## ⚙️ 配置说明

配置文件路径：

* Linux/macOS: `~/.config/crosscopy/config.toml`
* Windows: `%APPDATA%\crosscopy\config.toml`

示例配置：

```toml
device_name = "MacBook-Pro"
peer_list = ["192.168.1.100:8888", "192.168.1.101:8888"]
secret_key = "your-shared-key-here"
sync_images = true
cooldown_millis = 300
```


## 🔐 安全说明

* 所有剪贴板内容在发送前使用 AES-GCM 加密，避免中间人攻击。
* 你可以手动设置或生成一个共享密钥用于设备间鉴权。



## 🌍 路线图 / TODO

* [ ] 剪贴板历史记录功能
* [ ] 自动发现局域网设备（mDNS）
* [ ] 文件路径传输（待标准化格式支持）
* [ ] 托盘图标 + UI（egui / tauri）
* [ ] Android / iOS 客户端
* [ ] 云端同步（可选中转服务器）


## 🧑‍💻 开发者提示

* 本项目使用 Rust 2021 edition
* 所有模块均支持跨平台构建，推荐使用 `cargo run --release` 测试性能
* 调试时可设置日志环境变量：

```bash
RUST_LOG=debug cargo run
```


## 📄 许可证

MIT License - 你可以自由使用、修改和分发此工具。


## 🤝 贡献与交流

欢迎 PR / Issues，或联系作者讨论实现细节。期待你将 CrossPaste 打造为真正实用的工具！



