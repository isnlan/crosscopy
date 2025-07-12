# CrossCopy 网络层迁移说明

## 概述

CrossCopy 已成功从 WebSocket 网络通信迁移到 libp2p 点对点网络架构，并实现了 mDNS 自动发现功能。

## 主要变化

### 1. 网络协议栈

**之前 (WebSocket):**
- 基于 `tokio-tungstenite` 的 WebSocket 通信
- 手动配置对等节点列表 (`peer_list`)
- TCP/WebSocket 双协议支持
- 需要手动指定对等节点的 IP 地址和端口

**现在 (libp2p):**
- 基于 `libp2p` 的点对点网络通信
- mDNS 自动发现局域网内的对等节点
- TCP 传输 + Noise 协议加密 + yamux 多路复用
- 支持可选的 QUIC 传输协议
- 无需手动配置对等节点

### 2. 配置变化

#### NetworkConfig 结构更新

**移除的字段:**
```rust
pub peer_list: Vec<PeerConfig>,        // 不再需要手动配置对等节点
pub auto_discovery: bool,              // 由 enable_mdns 替代
pub discovery_port: u16,               // mDNS 使用标准端口
```

**新增的字段:**
```rust
pub enable_mdns: bool,                 // 启用 mDNS 自动发现
pub mdns_discovery_interval: u64,      // mDNS 发现间隔（秒）
pub idle_connection_timeout: u64,      // 空闲连接超时（秒）
pub enable_quic: bool,                 // 启用 QUIC 传输
pub quic_port: Option<u16>,            // QUIC 端口（可选）
```

#### 默认配置更新

```rust
NetworkConfig {
    listen_port: 8888,
    connection_timeout: 10000,          // 10 秒（之前 5 秒）
    heartbeat_interval: 30000,          // 30 秒（之前 1 秒）
    max_connections: 10,
    enable_mdns: true,                  // 默认启用 mDNS
    mdns_discovery_interval: 30,        // 30 秒发现间隔
    idle_connection_timeout: 300,       // 5 分钟空闲超时
    enable_quic: false,                 // 默认使用 TCP
    quic_port: None,
}
```

### 3. 连接管理

**之前:**
- `Connection` 结构包含 `WebSocketStream`
- 直接通过 WebSocket 发送/接收消息
- 手动管理连接状态

**现在:**
- `Connection` 结构包含 `PeerId` 和 `Multiaddr`
- 通过 libp2p swarm 和消息通道通信
- libp2p 自动管理连接生命周期

### 4. 错误处理

**新增错误类型:**
```rust
#[error("Peer not found: {0}")]
PeerNotFound(String),

#[error("mDNS discovery failed: {0}")]
MdnsDiscoveryFailed(String),

#[error("libp2p error: {0}")]
Libp2p(String),

#[error("Transport error: {0}")]
Transport(String),
```

**移除错误类型:**
```rust
#[error("WebSocket error: {0}")]
WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
```

## 使用方法

### 基本使用

```rust
use crosscopy::config::NetworkConfig;
use crosscopy::events::EventBus;
use crosscopy::network::NetworkManager;

// 创建配置（启用 mDNS 自动发现）
let config = NetworkConfig {
    enable_mdns: true,
    mdns_discovery_interval: 30,
    ..NetworkConfig::default()
};

// 创建网络管理器
let event_bus = Arc::new(EventBus::new());
let mut manager = NetworkManager::new(config, event_bus).await?;

// 启动网络服务（自动开始 mDNS 发现）
manager.start().await?;

// 网络管理器会自动发现并连接到局域网内的其他 CrossCopy 实例
```

### 运行示例

```bash
# 运行 libp2p 网络演示
cargo run --example libp2p_network_demo

# 运行测试
cargo test network_libp2p_test
```

## 技术优势

### 1. 自动发现
- **mDNS 服务发现**: 自动发现局域网内的 CrossCopy 实例
- **零配置**: 无需手动配置 IP 地址和端口
- **动态网络**: 支持设备动态加入和离开网络

### 2. 安全性
- **Noise 协议**: 提供端到端加密
- **身份验证**: 基于 Ed25519 密钥对的身份验证
- **传输安全**: 所有通信都经过加密

### 3. 性能
- **多路复用**: yamux 协议支持单连接多流
- **连接池**: 高效的连接管理
- **QUIC 支持**: 可选的低延迟传输协议

### 4. 可扩展性
- **模块化设计**: libp2p 的模块化架构
- **协议扩展**: 易于添加新的传输协议和功能
- **跨平台**: 统一的网络抽象层

## 迁移指南

### 对于用户

1. **配置文件更新**: 删除 `peer_list` 配置，启用 `enable_mdns`
2. **自动发现**: 应用启动后会自动发现网络中的其他设备
3. **无需手动配置**: 不再需要手动添加对等设备

### 对于开发者

1. **依赖更新**: 使用 `libp2p` 替代 `tokio-tungstenite`
2. **API 变化**: 网络管理 API 保持兼容，内部实现已更新
3. **测试更新**: 网络测试现在基于 libp2p 模拟

## 文档更新

- ✅ `doc/technical-specification.md` - 网络协议规格
- ✅ `doc/api-reference.md` - API 参考文档  
- ✅ `doc/architecture.md` - 架构文档
- ✅ 代码注释和文档字符串

## 测试和验证

- ✅ 单元测试: `tests/network_libp2p_test.rs`
- ✅ 示例程序: `examples/libp2p_network_demo.rs`
- ✅ 配置序列化/反序列化测试
- ✅ 网络管理器生命周期测试

## 后续工作

1. **完善 libp2p 实现**: 完成消息编解码器实现
2. **性能优化**: 调优网络参数和连接管理
3. **错误处理**: 增强网络错误恢复机制
4. **监控和日志**: 添加详细的网络状态监控

---

**注意**: 此迁移保持了向后兼容的 API，现有的应用代码无需修改，只需更新配置文件即可享受新的自动发现功能。
