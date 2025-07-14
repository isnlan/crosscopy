# CrossCopy 6位数字验证码身份认证系统设计

## 1. 概述

本文档描述了CrossCopy项目中基于6位数字验证码的身份认证系统设计。该系统在现有的libp2p Noise协议安全层之上，增加了用户友好的设备配对验证机制。

### 1.1 设计目标

- **用户友好**：通过简单的6位数字验证码完成设备配对
- **安全性**：防止未授权设备接入网络
- **兼容性**：与现有libp2p网络架构无缝集成
- **可扩展性**：支持多种认证模式和配置选项

### 1.2 认证流程概述

```
设备A (客户端)                    设备B (服务端)
     |                               |
     |-------- 发现设备 (mDNS) ------>|
     |<------- 设备信息响应 ----------|
     |                               |
     |-------- 连接请求 ------------->|
     |                               | 生成6位验证码
     |<------- 验证码挑战 ------------|  显示给用户
     |                               |
用户输入验证码                        |
     |                               |
     |-------- 验证码响应 ----------->| 验证码验证
     |<------- 认证结果 --------------|
     |                               |
     |====== 建立安全连接 ============|
```

## 2. 系统架构

### 2.1 核心组件

#### 2.1.1 AuthenticationManager
- **职责**：管理整个认证流程
- **功能**：
  - 生成和验证6位数字验证码
  - 管理认证状态和超时
  - 处理认证消息的收发
  - 维护已认证设备列表

#### 2.1.2 VerificationCodeGenerator
- **职责**：生成安全的6位数字验证码
- **功能**：
  - 使用密码学安全的随机数生成器
  - 确保验证码的唯一性和不可预测性
  - 支持验证码有效期管理

#### 2.1.3 DeviceTrustManager
- **职责**：管理设备信任关系
- **功能**：
  - 存储已认证设备的信任状态
  - 支持设备信任的撤销和更新
  - 提供设备黑名单功能

### 2.2 消息类型扩展

在现有的`MessageType`枚举中添加认证相关消息：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum MessageType {
    // 现有消息类型...
    Handshake = 0x0001,
    Heartbeat = 0x0002,
    ClipboardSync = 0x0003,
    DeviceInfo = 0x0004,
    Ack = 0x0005,
    Error = 0x0006,
    
    // 新增认证消息类型
    AuthChallenge = 0x0007,      // 认证挑战（包含验证码）
    AuthResponse = 0x0008,       // 认证响应（用户输入的验证码）
    AuthResult = 0x0009,         // 认证结果（成功/失败）
    TrustRevoke = 0x000A,        // 撤销设备信任
}
```

## 3. 详细设计

### 3.1 认证流程详细步骤

#### 步骤1：设备发现
- 设备A通过mDNS发现局域网内的CrossCopy设备
- 设备B响应发现请求，提供基本设备信息（设备名称、系统信息等）

#### 步骤2：连接建立
- 设备A选择要连接的设备B，发起libp2p连接
- 完成Noise协议握手，建立加密通道
- 连接状态变为`Connected`，但尚未`Authenticated`

#### 步骤3：认证挑战
- 设备B检测到新的连接请求
- 生成6位数字验证码（例如：123456）
- 通过UI显示验证码给用户
- 发送`AuthChallenge`消息给设备A，包含：
  - 挑战ID（用于关联后续响应）
  - 验证码有效期（默认5分钟）
  - 设备信息（设备名称、系统信息）

#### 步骤4：用户输入验证码
- 设备A收到挑战后，通过UI提示用户输入验证码
- 用户在设备A上输入在设备B上看到的6位数字
- 设备A发送`AuthResponse`消息，包含：
  - 挑战ID
  - 用户输入的验证码
  - 设备A的身份信息

#### 步骤5：验证码验证
- 设备B接收到响应，验证：
  - 挑战ID是否有效
  - 验证码是否正确
  - 是否在有效期内
- 发送`AuthResult`消息通知验证结果

#### 步骤6：建立信任关系
- 如果验证成功：
  - 连接状态变为`Authenticated`
  - 将设备A添加到信任设备列表
  - 开始正常的剪贴板同步服务
- 如果验证失败：
  - 断开连接
  - 记录失败日志
  - 可选：临时封禁该设备

### 3.2 数据结构设计

#### 3.2.1 认证挑战消息
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthChallenge {
    pub challenge_id: String,           // 挑战唯一标识
    pub verification_code: String,      // 6位验证码
    pub expires_at: u64,               // 过期时间戳
    pub device_info: DeviceInfo,       // 发起挑战的设备信息
    pub challenge_type: ChallengeType, // 挑战类型
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeType {
    FirstTimeConnection,    // 首次连接
    ReAuthentication,      // 重新认证
    TrustExpired,         // 信任过期
}
```

#### 3.2.2 认证响应消息
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub challenge_id: String,        // 对应的挑战ID
    pub verification_code: String,   // 用户输入的验证码
    pub device_info: DeviceInfo,    // 响应设备的信息
    pub timestamp: u64,             // 响应时间戳
}
```

#### 3.2.3 认证结果消息
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub challenge_id: String,     // 对应的挑战ID
    pub success: bool,           // 认证是否成功
    pub error_code: Option<AuthErrorCode>, // 失败时的错误码
    pub trust_level: TrustLevel, // 信任级别
    pub expires_at: Option<u64>, // 信任过期时间
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthErrorCode {
    InvalidCode,        // 验证码错误
    Expired,           // 验证码过期
    TooManyAttempts,   // 尝试次数过多
    DeviceBlocked,     // 设备被封禁
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel {
    Temporary,    // 临时信任（单次会话）
    Session,      // 会话信任（直到断开连接）
    Persistent,   // 持久信任（记住设备）
}
```

### 3.3 安全考虑

#### 3.3.1 验证码生成
- 使用密码学安全的随机数生成器（`rand::thread_rng()`）
- 确保验证码在统计上均匀分布
- 避免容易混淆的数字组合（如：000000、123456等）

#### 3.3.2 防暴力破解
- 限制验证码尝试次数（默认3次）
- 失败后增加延迟时间（指数退避）
- 记录失败尝试，支持临时封禁

#### 3.3.3 时间窗口控制
- 验证码默认有效期5分钟
- 支持配置不同的有效期
- 过期后自动清理相关状态

#### 3.3.4 重放攻击防护
- 每个挑战使用唯一的challenge_id
- 验证码只能使用一次
- 包含时间戳验证

## 4. 配置选项

### 4.1 认证配置
```toml
[authentication]
# 是否启用6位数字验证码认证
enable_verification_code = true

# 验证码有效期（秒）
code_expiry_seconds = 300

# 最大尝试次数
max_attempts = 3

# 失败后的封禁时间（秒）
ban_duration_seconds = 600

# 默认信任级别
default_trust_level = "Session"

# 是否允许记住设备
allow_persistent_trust = true

# 自动信任同一用户的设备
auto_trust_same_user = false
```

### 4.2 UI配置
```toml
[ui.authentication]
# 验证码显示样式
code_display_style = "Large"  # Large, Normal, Compact

# 是否显示设备信息
show_device_info = true

# 是否启用声音提示
enable_sound_notification = true

# 验证码字体大小
code_font_size = 24
```

## 5. 实现计划

### 5.1 第一阶段：核心认证功能
- [ ] 实现`AuthenticationManager`核心逻辑
- [ ] 添加认证相关消息类型和数据结构
- [ ] 实现验证码生成和验证逻辑
- [ ] 集成到现有网络管理器中

### 5.2 第二阶段：UI集成
- [ ] 设计验证码显示界面
- [ ] 实现验证码输入界面
- [ ] 添加设备信任管理界面
- [ ] 集成到系统托盘菜单

### 5.3 第三阶段：高级功能
- [ ] 实现设备信任持久化存储
- [ ] 添加设备黑名单功能
- [ ] 实现信任级别管理
- [ ] 添加认证日志和审计功能

### 5.4 第四阶段：优化和测试
- [ ] 性能优化和内存管理
- [ ] 全面的单元测试和集成测试
- [ ] 安全性测试和漏洞扫描
- [ ] 用户体验优化

## 6. 测试策略

### 6.1 单元测试
- 验证码生成算法测试
- 认证流程状态机测试
- 消息序列化/反序列化测试
- 安全性边界条件测试

### 6.2 集成测试
- 完整认证流程测试
- 多设备并发认证测试
- 网络异常情况处理测试
- UI交互流程测试

### 6.3 安全测试
- 暴力破解攻击测试
- 重放攻击防护测试
- 时间窗口攻击测试
- 网络中间人攻击测试

## 7. 部署和维护

### 7.1 向后兼容性
- 新的认证系统作为可选功能
- 支持禁用验证码认证，回退到原有机制
- 渐进式部署策略

### 7.2 监控和日志
- 认证成功/失败统计
- 异常行为检测和告警
- 性能指标监控
- 详细的调试日志

### 7.3 配置管理
- 支持运行时配置更新
- 配置验证和错误处理
- 默认配置的安全性保证

## 8. 核心代码示例

### 8.1 AuthenticationManager实现概览

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use rand::{thread_rng, Rng};
use serde::{Serialize, Deserialize};

pub struct AuthenticationManager {
    // 活跃的认证挑战
    active_challenges: RwLock<HashMap<String, AuthChallenge>>,
    // 已认证的设备
    trusted_devices: RwLock<HashMap<String, TrustedDevice>>,
    // 配置
    config: AuthConfig,
    // 事件通知
    event_sender: mpsc::UnboundedSender<AuthEvent>,
}

impl AuthenticationManager {
    /// 生成6位数字验证码
    pub fn generate_verification_code() -> String {
        let mut rng = thread_rng();
        format!("{:06}", rng.gen_range(100000..=999999))
    }

    /// 创建认证挑战
    pub async fn create_challenge(&self, peer_id: &str, device_info: DeviceInfo) -> Result<AuthChallenge> {
        let challenge_id = uuid::Uuid::new_v4().to_string();
        let verification_code = Self::generate_verification_code();
        let expires_at = chrono::Utc::now().timestamp() as u64 + self.config.code_expiry_seconds;

        let challenge = AuthChallenge {
            challenge_id: challenge_id.clone(),
            verification_code: verification_code.clone(),
            expires_at,
            device_info,
            challenge_type: ChallengeType::FirstTimeConnection,
            peer_id: peer_id.to_string(),
            created_at: Instant::now(),
            attempt_count: 0,
        };

        // 存储挑战
        self.active_challenges.write().await.insert(challenge_id.clone(), challenge.clone());

        // 发送UI事件显示验证码
        let _ = self.event_sender.send(AuthEvent::ShowVerificationCode {
            code: verification_code,
            device_name: device_info.device_name,
            expires_in: self.config.code_expiry_seconds,
        });

        Ok(challenge)
    }

    /// 验证用户输入的验证码
    pub async fn verify_code(&self, challenge_id: &str, input_code: &str) -> AuthResult {
        let mut challenges = self.active_challenges.write().await;

        if let Some(challenge) = challenges.get_mut(challenge_id) {
            // 检查是否过期
            let now = chrono::Utc::now().timestamp() as u64;
            if now > challenge.expires_at {
                challenges.remove(challenge_id);
                return AuthResult {
                    challenge_id: challenge_id.to_string(),
                    success: false,
                    error_code: Some(AuthErrorCode::Expired),
                    trust_level: TrustLevel::None,
                    expires_at: None,
                };
            }

            // 增加尝试次数
            challenge.attempt_count += 1;

            // 检查尝试次数
            if challenge.attempt_count > self.config.max_attempts {
                challenges.remove(challenge_id);
                return AuthResult {
                    challenge_id: challenge_id.to_string(),
                    success: false,
                    error_code: Some(AuthErrorCode::TooManyAttempts),
                    trust_level: TrustLevel::None,
                    expires_at: None,
                };
            }

            // 验证验证码
            if challenge.verification_code == input_code {
                // 验证成功
                let peer_id = challenge.peer_id.clone();
                let device_info = challenge.device_info.clone();
                challenges.remove(challenge_id);

                // 添加到信任设备列表
                let trusted_device = TrustedDevice {
                    peer_id: peer_id.clone(),
                    device_info,
                    trust_level: self.config.default_trust_level.clone(),
                    created_at: Instant::now(),
                    last_seen: Instant::now(),
                    expires_at: self.calculate_trust_expiry(),
                };

                self.trusted_devices.write().await.insert(peer_id, trusted_device);

                // 发送成功事件
                let _ = self.event_sender.send(AuthEvent::AuthenticationSuccess {
                    challenge_id: challenge_id.to_string(),
                });

                AuthResult {
                    challenge_id: challenge_id.to_string(),
                    success: true,
                    error_code: None,
                    trust_level: self.config.default_trust_level.clone(),
                    expires_at: self.calculate_trust_expiry(),
                }
            } else {
                // 验证失败
                AuthResult {
                    challenge_id: challenge_id.to_string(),
                    success: false,
                    error_code: Some(AuthErrorCode::InvalidCode),
                    trust_level: TrustLevel::None,
                    expires_at: None,
                }
            }
        } else {
            AuthResult {
                challenge_id: challenge_id.to_string(),
                success: false,
                error_code: Some(AuthErrorCode::InvalidChallenge),
                trust_level: TrustLevel::None,
                expires_at: None,
            }
        }
    }
}
```

### 8.2 网络协议集成示例

```rust
// 在现有的网络管理器中集成认证
impl NetworkManager {
    async fn handle_new_connection(&mut self, peer_id: PeerId) -> Result<()> {
        // 检查是否为已信任设备
        if self.auth_manager.is_trusted_device(&peer_id.to_string()).await {
            // 直接建立连接
            self.establish_authenticated_connection(peer_id).await?;
        } else {
            // 需要进行认证
            self.initiate_authentication(peer_id).await?;
        }
        Ok(())
    }

    async fn initiate_authentication(&mut self, peer_id: PeerId) -> Result<()> {
        let device_info = self.get_local_device_info();
        let challenge = self.auth_manager.create_challenge(
            &peer_id.to_string(),
            device_info
        ).await?;

        // 发送认证挑战消息
        let challenge_msg = Message::new(
            MessageType::AuthChallenge,
            serde_json::to_vec(&challenge)?,
            self.device_system.clone(),
        );

        self.send_message_to_peer(peer_id, challenge_msg).await?;
        Ok(())
    }

    async fn handle_auth_response(&mut self, peer_id: PeerId, response: AuthResponse) -> Result<()> {
        let result = self.auth_manager.verify_code(
            &response.challenge_id,
            &response.verification_code
        ).await;

        // 发送认证结果
        let result_msg = Message::new(
            MessageType::AuthResult,
            serde_json::to_vec(&result)?,
            self.device_system.clone(),
        );

        self.send_message_to_peer(peer_id, result_msg).await?;

        if result.success {
            // 认证成功，建立正常连接
            self.establish_authenticated_connection(peer_id).await?;
        } else {
            // 认证失败，断开连接
            self.disconnect_peer(peer_id).await?;
        }

        Ok(())
    }
}
```

---

本设计文档为CrossCopy 6位数字验证码身份认证系统提供了完整的技术规范和实现示例。实现时应严格遵循安全最佳实践，确保用户数据和隐私的安全。
