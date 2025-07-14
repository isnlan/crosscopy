# CrossCopy 6位数字验证码认证系统实现计划

## 1. 实现概述

本文档详细描述了如何在现有的CrossCopy项目中实现6位数字验证码身份认证系统。该系统将与现有的libp2p网络架构无缝集成，提供用户友好的设备配对体验。

## 2. 文件结构规划

### 2.1 新增文件
```
src/
├── auth/                          # 新增认证模块
│   ├── mod.rs                     # 模块声明
│   ├── manager.rs                 # 认证管理器
│   ├── challenge.rs               # 挑战生成和验证
│   ├── trust.rs                   # 设备信任管理
│   └── events.rs                  # 认证事件定义
├── network/
│   ├── protocol.rs                # 扩展消息类型（已存在）
│   └── manager.rs                 # 集成认证流程（已存在）
└── ui/                           # UI相关（如果使用Tauri）
    ├── auth_dialog.rs             # 认证对话框
    └── trust_manager.rs           # 信任设备管理界面
```

### 2.2 修改现有文件
- `src/lib.rs` - 添加auth模块声明
- `src/network/protocol.rs` - 添加认证消息类型
- `src/network/manager.rs` - 集成认证流程
- `src/config/mod.rs` - 添加认证配置
- `Cargo.toml` - 添加必要依赖

## 3. 详细实现步骤

### 3.1 第一步：添加依赖和基础结构

#### 3.1.1 更新Cargo.toml
```toml
[dependencies]
# 现有依赖...
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
rand = "0.8"

# 如果需要持久化存储
sled = "0.34"  # 或者使用其他嵌入式数据库
```

#### 3.1.2 创建认证模块声明
在`src/lib.rs`中添加：
```rust
pub mod auth;
```

在`src/auth/mod.rs`中：
```rust
pub mod manager;
pub mod challenge;
pub mod trust;
pub mod events;

pub use manager::AuthenticationManager;
pub use challenge::{AuthChallenge, AuthResponse, AuthResult};
pub use trust::{TrustedDevice, TrustLevel};
pub use events::AuthEvent;
```

### 3.2 第二步：扩展网络协议

#### 3.2.1 在`src/network/protocol.rs`中添加新消息类型
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum MessageType {
    // 现有类型...
    Handshake = 0x0001,
    Heartbeat = 0x0002,
    ClipboardSync = 0x0003,
    DeviceInfo = 0x0004,
    Ack = 0x0005,
    Error = 0x0006,
    
    // 新增认证类型
    AuthChallenge = 0x0007,
    AuthResponse = 0x0008,
    AuthResult = 0x0009,
    TrustRevoke = 0x000A,
}
```

#### 3.2.2 添加认证相关的消息载荷结构
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthChallengePayload {
    pub challenge_id: String,
    pub expires_at: u64,
    pub device_info: DeviceInfo,
    pub challenge_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponsePayload {
    pub challenge_id: String,
    pub verification_code: String,
    pub device_info: DeviceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResultPayload {
    pub challenge_id: String,
    pub success: bool,
    pub error_code: Option<String>,
    pub trust_level: String,
}
```

### 3.3 第三步：实现核心认证组件

#### 3.3.1 创建`src/auth/challenge.rs`
```rust
use serde::{Deserialize, Serialize};
use std::time::Instant;
use uuid::Uuid;
use rand::{thread_rng, Rng};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthChallenge {
    pub challenge_id: String,
    pub verification_code: String,
    pub expires_at: u64,
    pub device_info: crate::config::DeviceInfo,
    pub challenge_type: ChallengeType,
    pub peer_id: String,
    #[serde(skip)]
    pub created_at: Instant,
    pub attempt_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeType {
    FirstTimeConnection,
    ReAuthentication,
    TrustExpired,
}

impl AuthChallenge {
    pub fn new(peer_id: String, device_info: crate::config::DeviceInfo, expiry_seconds: u64) -> Self {
        let mut rng = thread_rng();
        let verification_code = format!("{:06}", rng.gen_range(100000..=999999));
        
        Self {
            challenge_id: Uuid::new_v4().to_string(),
            verification_code,
            expires_at: chrono::Utc::now().timestamp() as u64 + expiry_seconds,
            device_info,
            challenge_type: ChallengeType::FirstTimeConnection,
            peer_id,
            created_at: Instant::now(),
            attempt_count: 0,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp() as u64;
        now > self.expires_at
    }
    
    pub fn increment_attempts(&mut self) {
        self.attempt_count += 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub challenge_id: String,
    pub verification_code: String,
    pub device_info: crate::config::DeviceInfo,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub challenge_id: String,
    pub success: bool,
    pub error_code: Option<AuthErrorCode>,
    pub trust_level: crate::auth::TrustLevel,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthErrorCode {
    InvalidCode,
    Expired,
    TooManyAttempts,
    DeviceBlocked,
    InvalidChallenge,
}
```

#### 3.3.2 创建`src/auth/trust.rs`
```rust
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub peer_id: String,
    pub device_info: crate::config::DeviceInfo,
    pub trust_level: TrustLevel,
    #[serde(skip)]
    pub created_at: Instant,
    #[serde(skip)]
    pub last_seen: Instant,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel {
    None,
    Temporary,    // 临时信任（单次会话）
    Session,      // 会话信任（直到断开连接）
    Persistent,   // 持久信任（记住设备）
}

pub struct DeviceTrustManager {
    trusted_devices: HashMap<String, TrustedDevice>,
    blocked_devices: HashMap<String, BlockedDevice>,
}

#[derive(Debug, Clone)]
struct BlockedDevice {
    peer_id: String,
    blocked_until: u64,
    reason: String,
}

impl DeviceTrustManager {
    pub fn new() -> Self {
        Self {
            trusted_devices: HashMap::new(),
            blocked_devices: HashMap::new(),
        }
    }
    
    pub fn add_trusted_device(&mut self, device: TrustedDevice) {
        self.trusted_devices.insert(device.peer_id.clone(), device);
    }
    
    pub fn is_trusted(&self, peer_id: &str) -> bool {
        if let Some(device) = self.trusted_devices.get(peer_id) {
            // 检查是否过期
            if let Some(expires_at) = device.expires_at {
                let now = chrono::Utc::now().timestamp() as u64;
                return now <= expires_at;
            }
            true
        } else {
            false
        }
    }
    
    pub fn is_blocked(&self, peer_id: &str) -> bool {
        if let Some(blocked) = self.blocked_devices.get(peer_id) {
            let now = chrono::Utc::now().timestamp() as u64;
            now < blocked.blocked_until
        } else {
            false
        }
    }
    
    pub fn block_device(&mut self, peer_id: String, duration_seconds: u64, reason: String) {
        let blocked_until = chrono::Utc::now().timestamp() as u64 + duration_seconds;
        let blocked_device = BlockedDevice {
            peer_id: peer_id.clone(),
            blocked_until,
            reason,
        };
        self.blocked_devices.insert(peer_id, blocked_device);
    }
    
    pub fn revoke_trust(&mut self, peer_id: &str) {
        self.trusted_devices.remove(peer_id);
    }
}
```

### 3.4 第四步：实现认证管理器

#### 3.4.1 创建`src/auth/manager.rs`
```rust
use crate::auth::{AuthChallenge, AuthResponse, AuthResult, AuthErrorCode, TrustLevel};
use crate::auth::trust::{DeviceTrustManager, TrustedDevice};
use crate::auth::events::AuthEvent;
use crate::config::{AuthConfig, DeviceInfo};
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::{RwLock, mpsc};
use log::{debug, info, warn, error};

pub struct AuthenticationManager {
    active_challenges: RwLock<HashMap<String, AuthChallenge>>,
    trust_manager: RwLock<DeviceTrustManager>,
    config: AuthConfig,
    event_sender: mpsc::UnboundedSender<AuthEvent>,
}

impl AuthenticationManager {
    pub fn new(config: AuthConfig, event_sender: mpsc::UnboundedSender<AuthEvent>) -> Self {
        Self {
            active_challenges: RwLock::new(HashMap::new()),
            trust_manager: RwLock::new(DeviceTrustManager::new()),
            config,
            event_sender,
        }
    }
    
    /// 检查设备是否已被信任
    pub async fn is_trusted_device(&self, peer_id: &str) -> bool {
        let trust_manager = self.trust_manager.read().await;
        trust_manager.is_trusted(peer_id) && !trust_manager.is_blocked(peer_id)
    }
    
    /// 创建认证挑战
    pub async fn create_challenge(&self, peer_id: &str, device_info: DeviceInfo) -> Result<AuthChallenge, String> {
        // 检查设备是否被封禁
        {
            let trust_manager = self.trust_manager.read().await;
            if trust_manager.is_blocked(peer_id) {
                return Err("Device is blocked".to_string());
            }
        }
        
        let challenge = AuthChallenge::new(
            peer_id.to_string(),
            device_info.clone(),
            self.config.code_expiry_seconds
        );
        
        // 存储挑战
        self.active_challenges.write().await.insert(
            challenge.challenge_id.clone(),
            challenge.clone()
        );
        
        // 发送UI事件
        let _ = self.event_sender.send(AuthEvent::ShowVerificationCode {
            code: challenge.verification_code.clone(),
            device_name: device_info.device_name,
            expires_in: self.config.code_expiry_seconds,
        });
        
        info!("Created authentication challenge for device: {}", peer_id);
        Ok(challenge)
    }
    
    /// 验证认证响应
    pub async fn verify_response(&self, response: &AuthResponse) -> AuthResult {
        let mut challenges = self.active_challenges.write().await;
        
        if let Some(mut challenge) = challenges.remove(&response.challenge_id) {
            // 检查过期
            if challenge.is_expired() {
                warn!("Authentication challenge expired: {}", response.challenge_id);
                return AuthResult {
                    challenge_id: response.challenge_id.clone(),
                    success: false,
                    error_code: Some(AuthErrorCode::Expired),
                    trust_level: TrustLevel::None,
                    expires_at: None,
                };
            }
            
            // 增加尝试次数
            challenge.increment_attempts();
            
            // 检查尝试次数
            if challenge.attempt_count > self.config.max_attempts {
                warn!("Too many authentication attempts for: {}", challenge.peer_id);
                
                // 封禁设备
                let mut trust_manager = self.trust_manager.write().await;
                trust_manager.block_device(
                    challenge.peer_id.clone(),
                    self.config.ban_duration_seconds,
                    "Too many failed authentication attempts".to_string()
                );
                
                return AuthResult {
                    challenge_id: response.challenge_id.clone(),
                    success: false,
                    error_code: Some(AuthErrorCode::TooManyAttempts),
                    trust_level: TrustLevel::None,
                    expires_at: None,
                };
            }
            
            // 验证验证码
            if challenge.verification_code == response.verification_code {
                // 认证成功
                info!("Authentication successful for device: {}", challenge.peer_id);
                
                // 添加到信任设备
                let trusted_device = TrustedDevice {
                    peer_id: challenge.peer_id.clone(),
                    device_info: response.device_info.clone(),
                    trust_level: self.config.default_trust_level.clone(),
                    created_at: Instant::now(),
                    last_seen: Instant::now(),
                    expires_at: self.calculate_trust_expiry(),
                };
                
                self.trust_manager.write().await.add_trusted_device(trusted_device);
                
                // 发送成功事件
                let _ = self.event_sender.send(AuthEvent::AuthenticationSuccess {
                    challenge_id: response.challenge_id.clone(),
                    device_name: response.device_info.device_name.clone(),
                });
                
                AuthResult {
                    challenge_id: response.challenge_id.clone(),
                    success: true,
                    error_code: None,
                    trust_level: self.config.default_trust_level.clone(),
                    expires_at: self.calculate_trust_expiry(),
                }
            } else {
                // 验证失败，重新插入挑战（如果还有尝试机会）
                if challenge.attempt_count <= self.config.max_attempts {
                    challenges.insert(response.challenge_id.clone(), challenge);
                }
                
                warn!("Authentication failed - incorrect code for: {}", response.challenge_id);
                
                AuthResult {
                    challenge_id: response.challenge_id.clone(),
                    success: false,
                    error_code: Some(AuthErrorCode::InvalidCode),
                    trust_level: TrustLevel::None,
                    expires_at: None,
                }
            }
        } else {
            error!("Invalid challenge ID: {}", response.challenge_id);
            AuthResult {
                challenge_id: response.challenge_id.clone(),
                success: false,
                error_code: Some(AuthErrorCode::InvalidChallenge),
                trust_level: TrustLevel::None,
                expires_at: None,
            }
        }
    }
    
    /// 计算信任过期时间
    fn calculate_trust_expiry(&self) -> Option<u64> {
        match self.config.default_trust_level {
            TrustLevel::Temporary | TrustLevel::Session => None,
            TrustLevel::Persistent => {
                Some(chrono::Utc::now().timestamp() as u64 + 86400 * 30) // 30天
            },
            TrustLevel::None => None,
        }
    }
    
    /// 撤销设备信任
    pub async fn revoke_trust(&self, peer_id: &str) {
        self.trust_manager.write().await.revoke_trust(peer_id);
        info!("Revoked trust for device: {}", peer_id);
    }
    
    /// 清理过期的挑战
    pub async fn cleanup_expired_challenges(&self) {
        let mut challenges = self.active_challenges.write().await;
        let initial_count = challenges.len();
        
        challenges.retain(|_, challenge| !challenge.is_expired());
        
        let removed_count = initial_count - challenges.len();
        if removed_count > 0 {
            debug!("Cleaned up {} expired authentication challenges", removed_count);
        }
    }
}
```

## 4. 集成到网络管理器

### 4.1 修改`src/network/manager.rs`
在NetworkManager中添加认证管理器的集成：

```rust
use crate::auth::AuthenticationManager;

pub struct NetworkManager {
    // 现有字段...
    auth_manager: Option<AuthenticationManager>,
}

impl NetworkManager {
    // 在处理新连接时集成认证
    async fn handle_new_peer_connection(&mut self, peer_id: PeerId) -> Result<()> {
        if let Some(ref auth_manager) = self.auth_manager {
            if !auth_manager.is_trusted_device(&peer_id.to_string()).await {
                // 需要认证
                self.initiate_authentication(peer_id).await?;
                return Ok(());
            }
        }
        
        // 已信任设备，直接建立连接
        self.establish_authenticated_connection(peer_id).await
    }
    
    async fn initiate_authentication(&mut self, peer_id: PeerId) -> Result<()> {
        if let Some(ref auth_manager) = self.auth_manager {
            let device_info = self.get_local_device_info();
            let challenge = auth_manager.create_challenge(
                &peer_id.to_string(),
                device_info
            ).await.map_err(|e| NetworkError::AuthenticationFailed(e))?;
            
            // 发送认证挑战消息
            let payload = serde_json::to_vec(&challenge)?;
            let message = Message::new(
                MessageType::AuthChallenge,
                payload,
                self.device_system.clone()
            );
            
            self.send_message_to_peer(peer_id, message).await?;
        }
        Ok(())
    }
}
```

## 5. 配置扩展

### 5.1 在`src/config/mod.rs`中添加认证配置
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enable_verification_code: bool,
    pub code_expiry_seconds: u64,
    pub max_attempts: u32,
    pub ban_duration_seconds: u64,
    pub default_trust_level: crate::auth::TrustLevel,
    pub allow_persistent_trust: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enable_verification_code: true,
            code_expiry_seconds: 300,  // 5分钟
            max_attempts: 3,
            ban_duration_seconds: 600, // 10分钟
            default_trust_level: crate::auth::TrustLevel::Session,
            allow_persistent_trust: true,
        }
    }
}
```

## 6. 测试计划

### 6.1 单元测试
- 验证码生成测试
- 认证流程状态机测试
- 信任管理测试
- 配置验证测试

### 6.2 集成测试
- 完整认证流程测试
- 网络消息处理测试
- 多设备认证测试
- 异常情况处理测试

### 6.3 运行测试
```bash
# 运行认证演示
cargo run --example authentication_demo

# 运行单元测试
cargo test auth::

# 运行集成测试
cargo test --test authentication_integration
```

## 7. 部署步骤

1. **添加依赖** - 更新Cargo.toml
2. **创建认证模块** - 实现核心认证组件
3. **扩展网络协议** - 添加认证消息类型
4. **集成网络管理器** - 修改连接处理逻辑
5. **更新配置系统** - 添加认证相关配置
6. **实现UI组件** - 创建认证对话框（如果使用GUI）
7. **测试验证** - 运行测试确保功能正常
8. **文档更新** - 更新用户文档和API文档

这个实现计划提供了完整的6位数字验证码认证系统集成方案，确保与现有CrossCopy架构的无缝集成。
