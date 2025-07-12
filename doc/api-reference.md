# CrossCopy API 参考文档

## 1. 核心 API 概览

CrossCopy 提供了一套完整的 Rust API，用于剪贴板监控、网络通信、加密服务和配置管理。

## 2. 剪贴板 API

### 2.1 ClipboardMonitor

剪贴板监控器，负责监听系统剪贴板变化。

```rust
pub struct ClipboardMonitor {
    clipboard: Clipboard,
    last_content: Option<ClipboardContent>,
    cooldown: Duration,
}

impl ClipboardMonitor {
    /// 创建新的剪贴板监控器
    pub fn new(cooldown: Duration) -> Result<Self, ClipboardError>;
    
    /// 开始监控剪贴板变化
    pub async fn start_monitoring(&mut self) -> Result<(), ClipboardError>;
    
    /// 停止监控
    pub fn stop_monitoring(&mut self);
    
    /// 获取当前剪贴板内容
    pub fn get_content(&mut self) -> Result<ClipboardContent, ClipboardError>;
    
    /// 设置剪贴板内容
    pub fn set_content(&mut self, content: ClipboardContent) -> Result<(), ClipboardError>;
}
```

### 2.2 ClipboardContent

剪贴板内容的抽象表示。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardContent {
    Text(String),
    Image(ImageData),
    Files(Vec<PathBuf>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub format: ImageFormat,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
    Gif,
}
```

### 2.3 错误类型

```rust
#[derive(Debug, thiserror::Error)]
pub enum ClipboardError {
    #[error("剪贴板访问失败: {0}")]
    AccessFailed(String),
    
    #[error("不支持的内容类型")]
    UnsupportedContentType,
    
    #[error("内容序列化失败: {0}")]
    SerializationFailed(String),
    
    #[error("系统错误: {0}")]
    SystemError(String),
}
```

## 3. 网络通信 API

### 3.1 NetworkManager

网络管理器，处理设备间的通信。

```rust
pub struct NetworkManager {
    config: NetworkConfig,
    connections: HashMap<String, Connection>,
    message_handler: Box<dyn MessageHandler>,
}

impl NetworkManager {
    /// 创建网络管理器
    pub fn new(config: NetworkConfig) -> Self;
    
    /// 启动网络服务
    pub async fn start(&mut self) -> Result<(), NetworkError>;
    
    /// 停止网络服务
    pub async fn stop(&mut self) -> Result<(), NetworkError>;
    
    /// 连接到对等设备
    pub async fn connect_to_peer(&mut self, address: &str) -> Result<(), NetworkError>;
    
    /// 断开与对等设备的连接
    pub async fn disconnect_from_peer(&mut self, device_id: &str) -> Result<(), NetworkError>;
    
    /// 发送消息到指定设备
    pub async fn send_message(&mut self, device_id: &str, message: Message) -> Result<(), NetworkError>;
    
    /// 广播消息到所有连接的设备
    pub async fn broadcast_message(&mut self, message: Message) -> Result<(), NetworkError>;
    
    /// 获取连接状态
    pub fn get_connection_status(&self) -> HashMap<String, ConnectionStatus>;
}
```

### 3.2 Message Protocol

消息协议定义。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub timestamp: u64,
    pub sender_id: String,
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    ClipboardSync,
    Heartbeat,
    DeviceInfo,
    Acknowledgment,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardSyncPayload {
    pub content: ClipboardContent,
    pub sync_timestamp: u64,
}
```

### 3.3 Connection Management

连接管理相关类型。

```rust
#[derive(Debug, Clone)]
pub struct Connection {
    pub device_id: String,
    pub address: SocketAddr,
    pub status: ConnectionStatus,
    pub last_seen: Instant,
    pub websocket: Option<WebSocketStream<TcpStream>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

pub trait MessageHandler: Send + Sync {
    async fn handle_message(&mut self, message: Message, sender: &str) -> Result<(), NetworkError>;
}
```

## 4. 加密服务 API

### 4.1 EncryptionService

加密服务提供端到端加密功能。

```rust
pub struct EncryptionService {
    cipher: Aes256Gcm,
    key: [u8; 32],
}

impl EncryptionService {
    /// 从密钥创建加密服务
    pub fn new(key: &[u8]) -> Result<Self, EncryptionError>;
    
    /// 从密码派生密钥并创建加密服务
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, EncryptionError>;
    
    /// 加密数据
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError>;
    
    /// 解密数据
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, EncryptionError>;
    
    /// 生成随机密钥
    pub fn generate_key() -> [u8; 32];
    
    /// 计算数据校验和
    pub fn calculate_checksum(&self, data: &[u8]) -> String;
    
    /// 验证数据校验和
    pub fn verify_checksum(&self, data: &[u8], checksum: &str) -> bool;
}
```

### 4.2 密钥管理

```rust
pub struct KeyManager {
    master_key: [u8; 32],
    derived_keys: HashMap<String, [u8; 32]>,
}

impl KeyManager {
    /// 创建密钥管理器
    pub fn new(master_key: [u8; 32]) -> Self;
    
    /// 派生设备专用密钥
    pub fn derive_device_key(&mut self, device_id: &str) -> [u8; 32];
    
    /// 轮换主密钥
    pub fn rotate_master_key(&mut self, new_key: [u8; 32]);
    
    /// 获取设备密钥
    pub fn get_device_key(&self, device_id: &str) -> Option<&[u8; 32]>;
}
```

## 5. 配置管理 API

### 5.1 Configuration

应用配置管理。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub device_name: String,
    pub device_id: String,
    pub network: NetworkConfig,
    pub clipboard: ClipboardConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_port: u16,
    pub peer_list: Vec<String>,
    pub connection_timeout: u64,
    pub heartbeat_interval: u64,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardConfig {
    pub sync_images: bool,
    pub sync_files: bool,
    pub cooldown_millis: u64,
    pub max_content_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub secret_key: String,
    pub enable_encryption: bool,
    pub key_rotation_interval: u64,
}
```

### 5.2 ConfigManager

配置管理器。

```rust
pub struct ConfigManager {
    config: AppConfig,
    config_path: PathBuf,
}

impl ConfigManager {
    /// 加载配置
    pub fn load() -> Result<Self, ConfigError>;
    
    /// 从指定路径加载配置
    pub fn load_from_path(path: PathBuf) -> Result<Self, ConfigError>;
    
    /// 保存配置
    pub fn save(&self) -> Result<(), ConfigError>;
    
    /// 获取配置
    pub fn get_config(&self) -> &AppConfig;
    
    /// 更新配置
    pub fn update_config(&mut self, config: AppConfig) -> Result<(), ConfigError>;
    
    /// 重置为默认配置
    pub fn reset_to_default(&mut self) -> Result<(), ConfigError>;
    
    /// 验证配置有效性
    pub fn validate_config(&self) -> Result<(), ConfigError>;
}
```

## 6. 事件系统 API

### 6.1 EventBus

事件总线，用于组件间通信。

```rust
pub struct EventBus {
    subscribers: HashMap<String, Vec<Box<dyn EventHandler>>>,
}

impl EventBus {
    /// 创建事件总线
    pub fn new() -> Self;
    
    /// 订阅事件
    pub fn subscribe<T: EventHandler + 'static>(&mut self, event_type: &str, handler: T);
    
    /// 取消订阅
    pub fn unsubscribe(&mut self, event_type: &str, handler_id: &str);
    
    /// 发布事件
    pub async fn publish(&mut self, event: Event) -> Result<(), EventError>;
    
    /// 发布同步事件
    pub fn publish_sync(&mut self, event: Event) -> Result<(), EventError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub event_type: String,
    pub timestamp: u64,
    pub data: serde_json::Value,
}

pub trait EventHandler: Send + Sync {
    async fn handle_event(&mut self, event: &Event) -> Result<(), EventError>;
}
```

### 6.2 预定义事件类型

```rust
pub mod events {
    pub const CLIPBOARD_CHANGED: &str = "clipboard.changed";
    pub const DEVICE_CONNECTED: &str = "device.connected";
    pub const DEVICE_DISCONNECTED: &str = "device.disconnected";
    pub const SYNC_COMPLETED: &str = "sync.completed";
    pub const SYNC_FAILED: &str = "sync.failed";
    pub const CONFIG_UPDATED: &str = "config.updated";
    pub const ERROR_OCCURRED: &str = "error.occurred";
}
```

## 7. 错误处理

### 7.1 统一错误类型

```rust
#[derive(Debug, thiserror::Error)]
pub enum CrossCopyError {
    #[error("剪贴板错误: {0}")]
    Clipboard(#[from] ClipboardError),
    
    #[error("网络错误: {0}")]
    Network(#[from] NetworkError),
    
    #[error("加密错误: {0}")]
    Encryption(#[from] EncryptionError),
    
    #[error("配置错误: {0}")]
    Config(#[from] ConfigError),
    
    #[error("事件错误: {0}")]
    Event(#[from] EventError),
    
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CrossCopyError>;
```

## 8. 使用示例

### 8.1 基本使用

```rust
use crosscopy::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载配置
    let config_manager = ConfigManager::load()?;
    let config = config_manager.get_config();
    
    // 创建组件
    let mut clipboard_monitor = ClipboardMonitor::new(
        Duration::from_millis(config.clipboard.cooldown_millis)
    )?;
    
    let mut network_manager = NetworkManager::new(config.network.clone());
    let encryption_service = EncryptionService::from_password(
        &config.security.secret_key, 
        b"crosscopy_salt"
    )?;
    
    // 启动服务
    network_manager.start().await?;
    clipboard_monitor.start_monitoring().await?;
    
    // 主循环
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
```

### 8.2 自定义消息处理

```rust
struct CustomMessageHandler {
    encryption_service: EncryptionService,
    clipboard_monitor: ClipboardMonitor,
}

#[async_trait]
impl MessageHandler for CustomMessageHandler {
    async fn handle_message(&mut self, message: Message, sender: &str) -> Result<(), NetworkError> {
        match message.message_type {
            MessageType::ClipboardSync => {
                let decrypted_payload = self.encryption_service.decrypt(&message.payload)?;
                let sync_payload: ClipboardSyncPayload = serde_json::from_slice(&decrypted_payload)?;
                self.clipboard_monitor.set_content(sync_payload.content)?;
            }
            _ => {}
        }
        Ok(())
    }
}
```
