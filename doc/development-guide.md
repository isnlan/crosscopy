# CrossCopy 开发指南

## 1. 开发环境搭建

### 1.1 系统要求

- **操作系统**：Windows 10+、macOS 10.15+、Linux (Ubuntu 18.04+)
- **Rust 版本**：1.70.0 或更高版本
- **内存**：至少 4GB RAM
- **存储**：至少 1GB 可用空间

### 1.2 安装 Rust 工具链

```bash
# 安装 rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 更新到最新版本
rustup update

# 安装必要的组件
rustup component add clippy rustfmt
```

### 1.3 克隆项目

```bash
git clone https://github.com/isnlan/crosscopy-rust.git
cd crosscopy-rust
```

### 1.4 构建项目

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

## 2. 项目结构

```
crosscopy/
├── src/
│   ├── main.rs                 # 主入口
│   ├── lib.rs                  # 库入口
│   ├── clipboard/              # 剪贴板模块
│   │   ├── mod.rs
│   │   ├── monitor.rs          # 剪贴板监控
│   │   └── content.rs          # 内容类型定义
│   ├── network/                # 网络通信模块
│   │   ├── mod.rs
│   │   ├── manager.rs          # 网络管理器
│   │   ├── protocol.rs         # 消息协议
│   │   └── connection.rs       # 连接管理
│   ├── crypto/                 # 加密模块
│   │   ├── mod.rs
│   │   ├── encryption.rs       # 加密服务
│   │   └── key_manager.rs      # 密钥管理
│   ├── config/                 # 配置模块
│   │   ├── mod.rs
│   │   └── manager.rs          # 配置管理
│   ├── events/                 # 事件系统
│   │   ├── mod.rs
│   │   ├── bus.rs              # 事件总线
│   │   └── handlers.rs         # 事件处理器
│   └── utils/                  # 工具模块
│       ├── mod.rs
│       ├── logger.rs           # 日志工具
│       └── platform.rs         # 平台相关
├── tests/                      # 测试文件
├── examples/                   # 示例代码
├── docs/                       # 文档
├── Cargo.toml                  # 项目配置
└── README.md                   # 项目说明
```

## 3. 核心模块开发

### 3.1 剪贴板模块开发

#### 创建剪贴板监控器

```rust
// src/clipboard/monitor.rs
use arboard::Clipboard;
use std::time::{Duration, Instant};
use tokio::time::interval;

pub struct ClipboardMonitor {
    clipboard: Clipboard,
    last_content: Option<String>,
    last_update: Instant,
    cooldown: Duration,
}

impl ClipboardMonitor {
    pub fn new(cooldown: Duration) -> Result<Self, ClipboardError> {
        Ok(Self {
            clipboard: Clipboard::new()?,
            last_content: None,
            last_update: Instant::now(),
            cooldown,
        })
    }

    pub async fn start_monitoring(&mut self) -> Result<(), ClipboardError> {
        let mut interval = interval(Duration::from_millis(100));
        
        loop {
            interval.tick().await;
            
            if let Ok(content) = self.clipboard.get_text() {
                if self.should_process_content(&content) {
                    self.handle_content_change(content).await?;
                }
            }
        }
    }

    fn should_process_content(&mut self, content: &str) -> bool {
        let now = Instant::now();
        
        if now.duration_since(self.last_update) < self.cooldown {
            return false;
        }
        
        if let Some(ref last) = self.last_content {
            if last == content {
                return false;
            }
        }
        
        self.last_content = Some(content.to_string());
        self.last_update = now;
        true
    }

    async fn handle_content_change(&self, content: String) -> Result<(), ClipboardError> {
        // 发送内容变化事件
        // 这里会调用事件总线发布事件
        Ok(())
    }
}
```

### 3.2 网络模块开发

#### WebSocket 服务器实现

```rust
// src/network/manager.rs
use tokio_tungstenite::{accept_async, WebSocketStream};
use tokio::net::{TcpListener, TcpStream};
use futures_util::{SinkExt, StreamExt};

pub struct NetworkManager {
    listener: Option<TcpListener>,
    connections: HashMap<String, WebSocketStream<TcpStream>>,
    config: NetworkConfig,
}

impl NetworkManager {
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            listener: None,
            connections: HashMap::new(),
            config,
        }
    }

    pub async fn start(&mut self) -> Result<(), NetworkError> {
        let addr = format!("0.0.0.0:{}", self.config.listen_port);
        let listener = TcpListener::bind(&addr).await?;
        
        log::info!("WebSocket server listening on {}", addr);
        
        while let Ok((stream, addr)) = listener.accept().await {
            log::info!("New connection from {}", addr);
            
            let ws_stream = accept_async(stream).await?;
            self.handle_connection(ws_stream, addr).await?;
        }
        
        Ok(())
    }

    async fn handle_connection(
        &mut self,
        ws_stream: WebSocketStream<TcpStream>,
        addr: SocketAddr,
    ) -> Result<(), NetworkError> {
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        while let Some(msg) = ws_receiver.next().await {
            match msg? {
                tokio_tungstenite::tungstenite::Message::Text(text) => {
                    self.handle_text_message(text).await?;
                }
                tokio_tungstenite::tungstenite::Message::Binary(data) => {
                    self.handle_binary_message(data).await?;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}
```

### 3.3 加密模块开发

#### AES-GCM 加密实现

```rust
// src/crypto/encryption.rs
use aes_gcm::{Aes256Gcm, Key, Nonce, NewAead, Aead};
use rand::{RngCore, thread_rng};
use sha2::{Sha256, Digest};

pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        
        Self { cipher }
    }

    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, EncryptionError> {
        let key = self.derive_key(password, salt)?;
        Ok(Self::new(&key))
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher.encrypt(nonce, data)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        
        // 将 nonce 和密文组合
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if encrypted_data.len() < 12 {
            return Err(EncryptionError::InvalidData("Data too short".to_string()));
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;
        
        Ok(plaintext)
    }

    fn derive_key(&self, password: &str, salt: &[u8]) -> Result<[u8; 32], EncryptionError> {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        
        Ok(key)
    }
}
```

## 4. 测试开发

### 4.1 单元测试

```rust
// tests/clipboard_tests.rs
use crosscopy::clipboard::*;
use std::time::Duration;

#[tokio::test]
async fn test_clipboard_monitor_creation() {
    let monitor = ClipboardMonitor::new(Duration::from_millis(300));
    assert!(monitor.is_ok());
}

#[tokio::test]
async fn test_content_deduplication() {
    let mut monitor = ClipboardMonitor::new(Duration::from_millis(100)).unwrap();
    
    // 模拟相同内容的重复检测
    let content = "test content";
    assert!(monitor.should_process_content(content));
    assert!(!monitor.should_process_content(content)); // 第二次应该被过滤
}
```

### 4.2 集成测试

```rust
// tests/integration_tests.rs
use crosscopy::*;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_end_to_end_sync() {
    // 创建两个实例模拟不同设备
    let config1 = create_test_config(8881);
    let config2 = create_test_config(8882);
    
    let mut app1 = CrossCopyApp::new(config1).await.unwrap();
    let mut app2 = CrossCopyApp::new(config2).await.unwrap();
    
    // 启动应用
    app1.start().await.unwrap();
    app2.start().await.unwrap();
    
    // 等待连接建立
    sleep(Duration::from_secs(1)).await;
    
    // 模拟剪贴板变化
    app1.simulate_clipboard_change("test content").await.unwrap();
    
    // 验证同步
    sleep(Duration::from_millis(500)).await;
    let content = app2.get_clipboard_content().await.unwrap();
    assert_eq!(content, "test content");
}

fn create_test_config(port: u16) -> AppConfig {
    AppConfig {
        device_name: format!("test-device-{}", port),
        device_id: format!("device-{}", port),
        network: NetworkConfig {
            listen_port: port,
            peer_list: vec![],
            connection_timeout: 5000,
            heartbeat_interval: 1000,
            max_connections: 10,
        },
        clipboard: ClipboardConfig {
            sync_images: true,
            sync_files: false,
            cooldown_millis: 300,
            max_content_size: 1024 * 1024,
        },
        security: SecurityConfig {
            secret_key: "test-key".to_string(),
            enable_encryption: true,
            key_rotation_interval: 86400,
        },
        logging: LoggingConfig {
            level: "debug".to_string(),
            file_path: None,
        },
    }
}
```

## 5. 调试和性能优化

### 5.1 日志配置

```rust
// src/utils/logger.rs
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;

pub fn init_logger(level: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_level = match level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    Builder::from_default_env()
        .target(Target::Stdout)
        .filter_level(log_level)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] [{}:{}] - {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();

    Ok(())
}
```

### 5.2 性能监控

```rust
// src/utils/metrics.rs
use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct PerformanceMetrics {
    timers: HashMap<String, Instant>,
    counters: HashMap<String, u64>,
    durations: HashMap<String, Vec<Duration>>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            timers: HashMap::new(),
            counters: HashMap::new(),
            durations: HashMap::new(),
        }
    }

    pub fn start_timer(&mut self, name: &str) {
        self.timers.insert(name.to_string(), Instant::now());
    }

    pub fn end_timer(&mut self, name: &str) {
        if let Some(start_time) = self.timers.remove(name) {
            let duration = start_time.elapsed();
            self.durations
                .entry(name.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
        }
    }

    pub fn increment_counter(&mut self, name: &str) {
        *self.counters.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn get_average_duration(&self, name: &str) -> Option<Duration> {
        self.durations.get(name).and_then(|durations| {
            if durations.is_empty() {
                None
            } else {
                let total: Duration = durations.iter().sum();
                Some(total / durations.len() as u32)
            }
        })
    }
}
```

## 6. 代码规范

### 6.1 命名规范

- **模块名**：使用 snake_case，如 `clipboard_monitor`
- **结构体名**：使用 PascalCase，如 `ClipboardMonitor`
- **函数名**：使用 snake_case，如 `start_monitoring`
- **常量名**：使用 SCREAMING_SNAKE_CASE，如 `MAX_CONTENT_SIZE`

### 6.2 错误处理

```rust
// 使用 thiserror 定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum ClipboardError {
    #[error("剪贴板访问失败: {0}")]
    AccessFailed(String),
    
    #[error("不支持的内容类型")]
    UnsupportedContentType,
    
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
}

// 使用 Result 类型返回错误
pub type Result<T> = std::result::Result<T, ClipboardError>;
```

### 6.3 文档注释

```rust
/// 剪贴板监控器
/// 
/// 负责监听系统剪贴板变化并触发相应事件
/// 
/// # Examples
/// 
/// ```rust
/// use crosscopy::clipboard::ClipboardMonitor;
/// use std::time::Duration;
/// 
/// let monitor = ClipboardMonitor::new(Duration::from_millis(300))?;
/// monitor.start_monitoring().await?;
/// ```
pub struct ClipboardMonitor {
    // ...
}
```

## 7. 发布流程

### 7.1 版本管理

```bash
# 更新版本号
cargo install cargo-edit
cargo set-version 0.2.0

# 创建发布标签
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

### 7.2 构建发布版本

```bash
# 清理构建缓存
cargo clean

# 构建发布版本
cargo build --release

# 运行完整测试套件
cargo test --release

# 生成文档
cargo doc --no-deps
```

### 7.3 打包分发

```bash
# 创建分发包
mkdir -p dist
cp target/release/crosscopy dist/
cp README.md LICENSE dist/

# 创建压缩包
tar -czf crosscopy-v0.2.0-linux-x64.tar.gz -C dist .
```

## 8. 贡献指南

### 8.1 提交代码

1. Fork 项目仓库
2. 创建功能分支：`git checkout -b feature/new-feature`
3. 提交更改：`git commit -am 'Add new feature'`
4. 推送分支：`git push origin feature/new-feature`
5. 创建 Pull Request

### 8.2 代码审查

- 确保所有测试通过
- 代码覆盖率不低于 80%
- 遵循项目代码规范
- 添加必要的文档和注释

### 8.3 问题报告

使用 GitHub Issues 报告问题，包含：
- 问题描述
- 复现步骤
- 期望行为
- 实际行为
- 环境信息
