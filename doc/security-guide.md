# CrossCopy 安全指南

## 1. 安全概述

CrossCopy 采用多层安全架构，确保剪贴板数据在传输和存储过程中的安全性。本指南详细介绍了安全机制、最佳实践和安全配置建议。

### 1.1 安全设计原则

- **零信任架构**：不信任任何网络连接，所有通信都需要验证
- **端到端加密**：数据在离开源设备前加密，到达目标设备后解密
- **最小权限原则**：程序只请求必要的系统权限
- **数据最小化**：只传输必要的数据，不保留不必要的信息
- **透明性**：开源代码，安全机制可审计

## 2. 加密机制

### 2.1 加密算法

#### 对称加密
- **算法**：AES-256-GCM (Advanced Encryption Standard)
- **密钥长度**：256 位
- **模式**：GCM (Galois/Counter Mode)
- **特性**：提供加密和认证双重保护

#### 密钥派生
- **算法**：PBKDF2 (Password-Based Key Derivation Function 2)
- **哈希函数**：SHA-256
- **迭代次数**：100,000 次
- **盐值长度**：32 字节

#### 消息认证
- **算法**：HMAC-SHA256
- **用途**：验证消息完整性和来源
- **标签长度**：32 字节

### 2.2 加密流程

```
原始数据 → 序列化 → 压缩(可选) → AES-GCM加密 → 添加认证标签 → 网络传输
```

#### 加密实现示例

```rust
pub struct EncryptionService {
    cipher: Aes256Gcm,
    key: [u8; 32],
}

impl EncryptionService {
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // 生成随机 nonce
        let mut nonce = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce);
        
        // 加密数据
        let ciphertext = self.cipher.encrypt(
            Nonce::from_slice(&nonce), 
            data
        )?;
        
        // 组合 nonce + ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
}
```

### 2.3 密钥管理

#### 密钥生成
```rust
// 生成强随机密钥
pub fn generate_master_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    thread_rng().fill_bytes(&mut key);
    key
}

// 从密码派生密钥
pub fn derive_key_from_password(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, 100_000, &mut key);
    key
}
```

#### 密钥轮换
- **自动轮换**：支持定期自动更换加密密钥
- **手动轮换**：用户可以手动触发密钥更新
- **向后兼容**：短期内保持对旧密钥的支持

## 3. 网络安全

### 3.1 传输层安全

#### WebSocket 安全
- **协议**：WSS (WebSocket Secure) over TLS 1.3
- **证书验证**：支持自签名证书和 CA 证书
- **连接验证**：基于共享密钥的连接认证

#### TCP 安全
- **加密传输**：所有 TCP 数据都经过应用层加密
- **连接认证**：握手阶段进行身份验证
- **重放攻击防护**：使用时间戳和随机数防止重放

### 3.2 网络访问控制

#### IP 白名单
```toml
[security]
enable_ip_whitelist = true
allowed_ips = [
    "192.168.1.100",
    "192.168.1.101",
    "10.0.0.0/24"
]
```

#### 端口安全
```toml
[network]
listen_port = 8888
bind_interface = "127.0.0.1"  # 只绑定本地接口
enable_upnp = false           # 禁用 UPnP 端口映射
```

### 3.3 防火墙配置

#### Windows 防火墙
```powershell
# 允许 CrossCopy 通过防火墙
New-NetFirewallRule -DisplayName "CrossCopy" -Direction Inbound -Port 8888 -Protocol TCP -Action Allow
```

#### Linux iptables
```bash
# 允许特定端口
sudo iptables -A INPUT -p tcp --dport 8888 -j ACCEPT

# 限制来源 IP
sudo iptables -A INPUT -p tcp --dport 8888 -s 192.168.1.0/24 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 8888 -j DROP
```

#### macOS 防火墙
```bash
# 启用应用程序防火墙
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate on

# 允许 CrossCopy
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /Applications/CrossCopy.app
```

## 4. 身份认证

### 4.1 设备认证

#### 设备 ID 生成
```rust
pub fn generate_device_id() -> String {
    let mut hasher = Sha256::new();
    hasher.update(hostname().unwrap_or_default());
    hasher.update(mac_address().unwrap_or_default());
    hasher.update(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_be_bytes());
    
    format!("{:x}", hasher.finalize())[..16].to_string()
}
```

#### 认证流程
```
1. 设备 A 发起连接请求
2. 设备 B 发送认证挑战 (随机数)
3. 设备 A 使用共享密钥签名挑战
4. 设备 B 验证签名
5. 建立加密通道
```

### 4.2 消息认证

#### 消息签名
```rust
pub struct AuthenticatedMessage {
    pub message: Message,
    pub signature: [u8; 32],
    pub timestamp: u64,
}

impl AuthenticatedMessage {
    pub fn sign(message: Message, key: &[u8; 32]) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut mac = HmacSha256::new_from_slice(key).unwrap();
        mac.update(&message.serialize());
        mac.update(&timestamp.to_be_bytes());
        
        let signature = mac.finalize().into_bytes().into();
        
        Self { message, signature, timestamp }
    }
}
```

## 5. 数据保护

### 5.1 内存安全

#### 敏感数据清理
```rust
pub struct SecureBuffer {
    data: Vec<u8>,
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        // 清零敏感数据
        self.data.fill(0);
    }
}
```

#### 内存锁定
```rust
use mlock::mlock;

pub fn secure_memory_allocation(size: usize) -> Result<Vec<u8>, SecurityError> {
    let mut buffer = vec![0u8; size];
    mlock(&buffer)?;  // 防止内存被交换到磁盘
    Ok(buffer)
}
```

### 5.2 存储安全

#### 配置文件加密
```toml
[security]
encrypt_config = true
config_key_file = "~/.crosscopy/config.key"
```

#### 临时文件处理
```rust
pub struct SecureTempFile {
    path: PathBuf,
}

impl Drop for SecureTempFile {
    fn drop(&mut self) {
        // 安全删除临时文件
        if self.path.exists() {
            let _ = secure_delete(&self.path);
        }
    }
}

fn secure_delete(path: &Path) -> Result<(), std::io::Error> {
    // 多次覆写文件内容
    let mut file = OpenOptions::new().write(true).open(path)?;
    let file_size = file.metadata()?.len();
    
    for _ in 0..3 {
        file.seek(SeekFrom::Start(0))?;
        let random_data = vec![thread_rng().gen::<u8>(); file_size as usize];
        file.write_all(&random_data)?;
        file.sync_all()?;
    }
    
    drop(file);
    std::fs::remove_file(path)
}
```

## 6. 安全配置

### 6.1 推荐安全配置

```toml
[security]
# 启用所有安全特性
enable_encryption = true
enable_authentication = true
enable_ip_whitelist = true
enable_rate_limiting = true

# 强密钥设置
secret_key = "your-very-strong-secret-key-here"
key_rotation_interval = 86400  # 24小时轮换一次

# 网络安全
[network]
connection_timeout = 5000
max_connections = 5
enable_tls = true
tls_cert_path = "/path/to/cert.pem"
tls_key_path = "/path/to/key.pem"

# 访问控制
allowed_ips = ["192.168.1.0/24"]
denied_ips = ["0.0.0.0/0"]  # 默认拒绝所有

# 日志安全
[logging]
log_sensitive_data = false
log_encryption = true
log_file_permissions = 0o600
```

### 6.2 安全检查清单

#### 部署前检查
- [ ] 更改默认端口
- [ ] 设置强密钥
- [ ] 配置防火墙规则
- [ ] 启用 IP 白名单
- [ ] 禁用不必要的功能
- [ ] 设置日志权限

#### 运行时监控
- [ ] 监控异常连接
- [ ] 检查加密状态
- [ ] 验证证书有效性
- [ ] 监控资源使用
- [ ] 检查日志异常

#### 定期维护
- [ ] 更新密钥
- [ ] 更新软件版本
- [ ] 审查访问日志
- [ ] 检查安全配置
- [ ] 备份配置文件

## 7. 威胁模型和防护

### 7.1 威胁分析

#### 网络威胁
- **中间人攻击**：通过加密和证书验证防护
- **窃听攻击**：端到端加密保护数据
- **重放攻击**：时间戳和随机数防护
- **拒绝服务**：连接限制和速率限制

#### 本地威胁
- **恶意软件**：权限控制和代码签名
- **物理访问**：配置文件加密
- **内存转储**：敏感数据清理
- **日志泄露**：日志脱敏和权限控制

### 7.2 安全响应

#### 入侵检测
```rust
pub struct SecurityMonitor {
    failed_attempts: HashMap<String, u32>,
    blocked_ips: HashSet<String>,
}

impl SecurityMonitor {
    pub fn check_connection(&mut self, ip: &str) -> bool {
        if self.blocked_ips.contains(ip) {
            return false;
        }
        
        let attempts = self.failed_attempts.get(ip).unwrap_or(&0);
        if *attempts >= 5 {
            self.blocked_ips.insert(ip.to_string());
            log::warn!("IP {} blocked due to too many failed attempts", ip);
            return false;
        }
        
        true
    }
}
```

#### 自动响应
- **连接阻断**：自动阻断可疑连接
- **密钥轮换**：检测到攻击时自动更换密钥
- **服务降级**：在攻击期间限制功能
- **告警通知**：向管理员发送安全告警

## 8. 合规性和审计

### 8.1 安全审计

#### 代码审计
- 使用静态分析工具检查安全漏洞
- 定期进行第三方安全审计
- 开源代码接受社区审查

#### 运行时审计
```rust
pub struct AuditLogger {
    log_file: File,
}

impl AuditLogger {
    pub fn log_security_event(&mut self, event: SecurityEvent) {
        let log_entry = serde_json::json!({
            "timestamp": SystemTime::now(),
            "event_type": event.event_type,
            "source_ip": event.source_ip,
            "details": event.details,
            "severity": event.severity
        });
        
        writeln!(self.log_file, "{}", log_entry).unwrap();
    }
}
```

### 8.2 隐私保护

#### 数据最小化
- 只收集必要的数据
- 不记录敏感内容
- 定期清理临时数据

#### 用户控制
- 用户可以控制同步内容类型
- 支持选择性同步
- 提供数据删除功能

## 9. 安全最佳实践

### 9.1 用户建议

1. **使用强密钥**：至少 32 字符的随机密钥
2. **定期更新**：每月更换一次连接密钥
3. **网络隔离**：使用专用网络进行同步
4. **最小权限**：只授予必要的系统权限
5. **监控日志**：定期检查安全日志

### 9.2 管理员建议

1. **网络分段**：将同步网络与生产网络隔离
2. **访问控制**：实施严格的 IP 白名单
3. **监控告警**：设置安全事件告警
4. **备份恢复**：定期备份配置和密钥
5. **应急响应**：制定安全事件响应计划

### 9.3 开发者建议

1. **安全编码**：遵循安全编码规范
2. **依赖管理**：定期更新安全依赖
3. **测试覆盖**：包含安全测试用例
4. **漏洞披露**：建立负责任的漏洞披露流程
5. **安全培训**：定期进行安全培训

---

**免责声明**：本安全指南提供了推荐的安全配置和最佳实践，但不能保证绝对安全。用户应根据自己的安全需求和威胁模型调整配置。
