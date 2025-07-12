# CrossCopy 用户手册

## 1. 简介

CrossCopy 是一款跨平台剪贴板同步工具，支持在 Windows、macOS 和 Linux 系统之间实时同步剪贴板内容。通过局域网连接，您可以在一台设备上复制内容，然后在另一台设备上直接粘贴，实现无缝的跨设备工作体验。

### 1.1 主要特性

- 🖥️ **跨平台支持**：支持 Windows、macOS、Linux
- 🔄 **实时同步**：毫秒级剪贴板内容同步
- 🔐 **安全加密**：端到端加密保护数据隐私
- 🌐 **局域网通信**：无需互联网连接
- 📝 **多格式支持**：文本、图片等多种内容格式
- ⚡ **轻量高效**：低资源占用，高性能表现

## 2. 安装指南

### 2.1 系统要求

**最低系统要求：**
- Windows 10 或更高版本
- macOS 10.15 (Catalina) 或更高版本
- Linux (Ubuntu 18.04+ 或等效发行版)
- 至少 100MB 可用磁盘空间
- 至少 256MB 可用内存

### 2.2 下载安装

#### Windows 安装

1. 从 [GitHub Releases](https://github.com/your-username/crosscopy-rust/releases) 下载最新的 Windows 安装包
2. 双击 `crosscopy-setup.exe` 运行安装程序
3. 按照安装向导完成安装
4. 安装完成后，程序会自动启动并在系统托盘显示图标

#### macOS 安装

1. 下载 `crosscopy-macos.dmg` 文件
2. 双击打开 DMG 文件
3. 将 CrossCopy 应用拖拽到 Applications 文件夹
4. 在 Applications 文件夹中双击启动 CrossCopy
5. 首次启动时，系统可能会要求授权访问剪贴板

#### Linux 安装

**使用预编译二进制文件：**
```bash
# 下载并解压
wget https://github.com/your-username/crosscopy-rust/releases/download/v1.0.0/crosscopy-linux-x64.tar.gz
tar -xzf crosscopy-linux-x64.tar.gz

# 移动到系统路径
sudo mv crosscopy /usr/local/bin/

# 启动程序
crosscopy
```

**从源码编译：**
```bash
# 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone https://github.com/your-username/crosscopy-rust.git
cd crosscopy-rust

# 编译安装
cargo build --release
sudo cp target/release/crosscopy /usr/local/bin/
```

## 3. 快速开始

### 3.1 首次运行

1. **启动程序**：双击桌面图标或从开始菜单启动 CrossCopy
2. **系统权限**：首次运行时，系统会要求授权访问剪贴板，请点击"允许"
3. **配置向导**：程序会自动打开配置向导，帮助您完成初始设置

### 3.2 基本配置

#### 设备名称设置
```
设备名称：MacBook-Pro-2023
描述：为您的设备设置一个易识别的名称
```

#### 网络设置
```
监听端口：8888 (默认)
连接密钥：your-secret-key-here
```

#### 同步选项
```
☑ 同步文本内容
☑ 同步图片内容
☐ 同步文件路径
```

### 3.3 连接其他设备

1. **获取设备信息**：在主界面查看本设备的 IP 地址和端口
2. **添加对等设备**：在其他设备上添加本设备的 IP 地址
3. **输入连接密钥**：确保所有设备使用相同的连接密钥
4. **测试连接**：复制一段文本，检查是否在其他设备上同步

## 4. 详细功能说明

### 4.1 剪贴板同步

#### 文本同步
- 支持纯文本、富文本格式
- 自动检测文本编码
- 支持多语言字符集
- 最大文本长度：1MB

#### 图片同步
- 支持格式：PNG、JPEG、BMP、GIF
- 最大图片大小：10MB
- 自动压缩大尺寸图片
- 保持图片质量和透明度

#### 文件路径同步
- 同步文件和文件夹路径
- 跨平台路径格式转换
- 支持网络路径和本地路径

### 4.2 设备管理

#### 设备发现
```
自动发现：程序会自动扫描局域网内的其他 CrossCopy 设备
手动添加：可以手动输入设备 IP 地址和端口
设备列表：显示所有已连接和可用的设备
```

#### 连接状态
- 🟢 **已连接**：设备在线且可以同步
- 🟡 **连接中**：正在尝试连接设备
- 🔴 **离线**：设备不可达或已断开
- ⚪ **未知**：设备状态未确定

### 4.3 安全设置

#### 加密配置
```
加密算法：AES-256-GCM
密钥长度：256 位
密钥派生：PBKDF2 + SHA-256
```

#### 访问控制
- 设备白名单：只允许指定设备连接
- 连接密钥：所有设备必须使用相同密钥
- 自动断开：检测到异常连接时自动断开

## 5. 高级配置

### 5.1 配置文件位置

**Windows：**
```
%APPDATA%\crosscopy\config.toml
```

**macOS：**
```
~/Library/Application Support/crosscopy/config.toml
```

**Linux：**
```
~/.config/crosscopy/config.toml
```

### 5.2 配置文件详解

```toml
# 设备配置
[device]
name = "MacBook-Pro-2023"
id = "device-12345"

# 网络配置
[network]
listen_port = 8888
peer_list = [
    "192.168.1.100:8888",
    "192.168.1.101:8888"
]
connection_timeout = 5000
heartbeat_interval = 1000
max_connections = 10

# 剪贴板配置
[clipboard]
sync_images = true
sync_files = false
cooldown_millis = 300
max_content_size = 10485760  # 10MB

# 安全配置
[security]
secret_key = "your-shared-key-here"
enable_encryption = true
key_rotation_interval = 86400  # 24小时

# 日志配置
[logging]
level = "info"
file_path = "/var/log/crosscopy.log"
```

### 5.3 命令行参数

```bash
# 指定配置文件
crosscopy --config /path/to/config.toml

# 设置日志级别
crosscopy --log-level debug

# 后台运行
crosscopy --daemon

# 显示版本信息
crosscopy --version

# 显示帮助信息
crosscopy --help
```

## 6. 故障排除

### 6.1 常见问题

#### 问题：设备无法连接
**可能原因：**
- 网络连接问题
- 防火墙阻止连接
- 端口被占用
- 连接密钥不匹配

**解决方案：**
1. 检查网络连接，确保设备在同一局域网
2. 检查防火墙设置，允许 CrossCopy 通过防火墙
3. 更改监听端口，避免端口冲突
4. 确认所有设备使用相同的连接密钥

#### 问题：剪贴板内容不同步
**可能原因：**
- 内容格式不支持
- 内容大小超过限制
- 去抖动时间过短
- 设备连接不稳定

**解决方案：**
1. 检查内容格式是否在支持范围内
2. 减小内容大小或调整大小限制
3. 增加去抖动时间间隔
4. 检查网络连接稳定性

#### 问题：程序启动失败
**可能原因：**
- 系统权限不足
- 配置文件损坏
- 依赖库缺失
- 端口被占用

**解决方案：**
1. 以管理员权限运行程序
2. 删除配置文件，重新生成默认配置
3. 重新安装程序，确保依赖完整
4. 更改默认端口设置

### 6.2 日志分析

#### 启用调试日志
```bash
# 设置环境变量
export RUST_LOG=debug

# 或使用命令行参数
crosscopy --log-level debug
```

#### 常见日志信息
```
INFO  - CrossCopy started successfully
DEBUG - Clipboard content changed: text/plain
DEBUG - Sending message to device: 192.168.1.100
ERROR - Failed to connect to peer: Connection refused
WARN  - Message encryption failed, skipping sync
```

### 6.3 性能优化

#### 减少资源占用
```toml
[clipboard]
cooldown_millis = 500  # 增加去抖动时间
max_content_size = 1048576  # 减小最大内容大小

[network]
heartbeat_interval = 5000  # 增加心跳间隔
max_connections = 5  # 减少最大连接数
```

#### 网络优化
```toml
[network]
connection_timeout = 3000  # 减少连接超时时间
enable_compression = true  # 启用数据压缩
buffer_size = 8192  # 调整缓冲区大小
```

## 7. 隐私和安全

### 7.1 数据保护

- **本地存储**：所有配置和临时数据仅存储在本地
- **传输加密**：所有网络传输使用 AES-256 加密
- **无云服务**：不依赖任何云服务或第三方服务器
- **开源透明**：源代码完全开源，可审计安全性

### 7.2 隐私政策

CrossCopy 承诺：
- 不收集任何用户个人信息
- 不上传剪贴板内容到服务器
- 不与第三方共享任何数据
- 所有数据处理均在本地完成

### 7.3 安全建议

1. **定期更新密钥**：建议每月更换一次连接密钥
2. **网络隔离**：在可能的情况下，使用专用网络进行同步
3. **权限控制**：只授予必要的系统权限
4. **定期更新**：及时更新到最新版本以获得安全修复

## 8. 技术支持

### 8.1 获取帮助

- **官方文档**：https://github.com/your-username/crosscopy-rust/wiki
- **问题报告**：https://github.com/your-username/crosscopy-rust/issues
- **讨论社区**：https://github.com/your-username/crosscopy-rust/discussions

### 8.2 联系方式

- **邮箱**：support@crosscopy.com
- **GitHub**：@your-username

### 8.3 贡献代码

欢迎贡献代码和改进建议：
1. Fork 项目仓库
2. 创建功能分支
3. 提交 Pull Request
4. 参与代码审查

---

**版本信息**：本手册适用于 CrossCopy v1.0.0 及以上版本
**最后更新**：2025年1月
