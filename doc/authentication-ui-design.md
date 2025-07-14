# CrossCopy 认证系统UI设计方案

## 1. UI设计概述

本文档描述了CrossCopy 6位数字验证码认证系统的用户界面设计方案。设计目标是提供直观、安全、用户友好的设备配对体验。

## 2. UI组件设计

### 2.1 验证码显示界面（服务端）

当有设备请求连接时，在服务端显示的界面：

```
┌─────────────────────────────────────────────────────────┐
│                   🔐 设备连接请求                        │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  📱 设备 "John的iPhone" 想要连接到此设备                │
│                                                         │
│  请在请求设备上输入以下验证码：                          │
│                                                         │
│              ┌─────────────────────┐                    │
│              │      1 2 3 4 5 6    │                    │
│              └─────────────────────┘                    │
│                                                         │
│  ⏰ 验证码将在 4:32 后过期                              │
│                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │    允许     │  │    拒绝     │  │    复制     │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### 2.2 验证码输入界面（客户端）

在客户端显示的验证码输入界面：

```
┌─────────────────────────────────────────────────────────┐
│                   🔗 连接到设备                          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  正在连接到 "办公室电脑"...                             │
│                                                         │
│  请输入目标设备上显示的6位验证码：                       │
│                                                         │
│  ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐                  │
│  │ 1 │ │ 2 │ │ 3 │ │ 4 │ │ 5 │ │ 6 │                  │
│  └───┘ └───┘ └───┘ └───┘ └───┘ └───┘                  │
│                                                         │
│  ⚠️  验证码错误，请重新输入 (剩余尝试: 2/3)              │
│                                                         │
│  ┌─────────────┐  ┌─────────────┐                      │
│  │    连接     │  │    取消     │                      │
│  └─────────────┘  └─────────────┘                      │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### 2.3 设备信任管理界面

管理已信任设备的界面：

```
┌─────────────────────────────────────────────────────────┐
│                   🛡️ 信任设备管理                        │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  已信任的设备 (3)                                       │
│                                                         │
│  📱 John的iPhone                                        │
│     └─ 上次连接: 2分钟前  [撤销信任] [详情]             │
│                                                         │
│  💻 办公室电脑                                          │
│     └─ 上次连接: 1小时前  [撤销信任] [详情]             │
│                                                         │
│  🖥️ 家里的台式机                                        │
│     └─ 上次连接: 昨天     [撤销信任] [详情]             │
│                                                         │
│  ────────────────────────────────────────────────────   │
│                                                         │
│  被阻止的设备 (1)                                       │
│                                                         │
│  📱 未知设备-ABC123                                     │
│     └─ 阻止原因: 多次输入错误  [解除阻止]               │
│                                                         │
│  ┌─────────────┐  ┌─────────────┐                      │
│  │  清除全部   │  │    关闭     │                      │
│  └─────────────┘  └─────────────┘                      │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### 2.4 系统托盘通知

当有认证请求时的系统托盘通知：

```
┌─────────────────────────────────────┐
│  🔐 CrossCopy                       │
│  ─────────────────────────────────  │
│  设备连接请求                       │
│  "John的iPhone" 想要连接            │
│  验证码: 123456                     │
│  ─────────────────────────────────  │
│  [允许] [拒绝] [查看详情]           │
└─────────────────────────────────────┘
```

## 3. 交互流程设计

### 3.1 服务端流程

1. **检测连接请求**
   - 显示设备信息和验证码
   - 开始倒计时显示
   - 提供操作按钮

2. **用户操作**
   - 允许：继续等待客户端输入
   - 拒绝：立即断开连接
   - 复制：复制验证码到剪贴板

3. **验证结果**
   - 成功：显示连接成功提示
   - 失败：显示失败原因和重试选项

### 3.2 客户端流程

1. **发起连接**
   - 显示目标设备信息
   - 显示连接状态

2. **输入验证码**
   - 6个独立输入框
   - 自动焦点切换
   - 实时验证

3. **处理结果**
   - 成功：显示连接成功，进入主界面
   - 失败：显示错误信息，允许重试

## 4. 技术实现方案

### 4.1 使用Tauri + React实现

#### 4.1.1 React组件结构
```typescript
// 认证相关组件
src/components/auth/
├── VerificationCodeDisplay.tsx    // 验证码显示组件
├── VerificationCodeInput.tsx      // 验证码输入组件
├── TrustedDeviceManager.tsx       // 信任设备管理
├── AuthNotification.tsx           // 认证通知组件
└── AuthDialog.tsx                 // 认证对话框

// 主要组件实现
interface VerificationCodeDisplayProps {
  code: string;
  deviceName: string;
  expiresIn: number;
  onAllow: () => void;
  onDeny: () => void;
  onCopy: () => void;
}

const VerificationCodeDisplay: React.FC<VerificationCodeDisplayProps> = ({
  code,
  deviceName,
  expiresIn,
  onAllow,
  onDeny,
  onCopy
}) => {
  const [timeLeft, setTimeLeft] = useState(expiresIn);
  
  useEffect(() => {
    const timer = setInterval(() => {
      setTimeLeft(prev => prev > 0 ? prev - 1 : 0);
    }, 1000);
    
    return () => clearInterval(timer);
  }, []);
  
  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };
  
  return (
    <div className="auth-dialog">
      <div className="header">
        <h2>🔐 设备连接请求</h2>
      </div>
      
      <div className="content">
        <p>📱 设备 "{deviceName}" 想要连接到此设备</p>
        <p>请在请求设备上输入以下验证码：</p>
        
        <div className="verification-code">
          {code.split('').map((digit, index) => (
            <span key={index} className="digit">{digit}</span>
          ))}
        </div>
        
        <p className="expiry">⏰ 验证码将在 {formatTime(timeLeft)} 后过期</p>
      </div>
      
      <div className="actions">
        <button onClick={onAllow} className="btn-primary">允许</button>
        <button onClick={onDeny} className="btn-secondary">拒绝</button>
        <button onClick={onCopy} className="btn-secondary">复制</button>
      </div>
    </div>
  );
};
```

#### 4.1.2 Tauri命令接口
```rust
// src-tauri/src/commands/auth.rs
use tauri::State;
use crate::auth::AuthenticationManager;

#[tauri::command]
pub async fn show_verification_code(
    code: String,
    device_name: String,
    expires_in: u64,
    app_handle: tauri::AppHandle
) -> Result<(), String> {
    // 显示认证对话框
    let window = app_handle.get_window("auth").ok_or("Auth window not found")?;
    window.emit("show-verification-code", serde_json::json!({
        "code": code,
        "deviceName": device_name,
        "expiresIn": expires_in
    })).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn verify_authentication_code(
    challenge_id: String,
    code: String,
    auth_manager: State<'_, AuthenticationManager>
) -> Result<bool, String> {
    // 验证认证码
    let response = crate::auth::AuthResponse {
        challenge_id,
        verification_code: code,
        device_info: get_device_info(),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };
    
    let result = auth_manager.verify_response(&response).await;
    Ok(result.success)
}

#[tauri::command]
pub async fn get_trusted_devices(
    auth_manager: State<'_, AuthenticationManager>
) -> Result<Vec<TrustedDeviceInfo>, String> {
    // 获取信任设备列表
    auth_manager.get_trusted_devices().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn revoke_device_trust(
    peer_id: String,
    auth_manager: State<'_, AuthenticationManager>
) -> Result<(), String> {
    auth_manager.revoke_trust(&peer_id).await;
    Ok(())
}
```

### 4.2 系统托盘集成

```rust
// src-tauri/src/tray.rs
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent};

pub fn create_system_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "退出");
    let show = CustomMenuItem::new("show".to_string(), "显示主界面");
    let trusted_devices = CustomMenuItem::new("trusted_devices".to_string(), "信任设备管理");
    
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(trusted_devices)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(quit);
    
    SystemTray::new().with_menu(tray_menu)
}

pub fn handle_system_tray_event(app: &tauri::AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "trusted_devices" => {
                    // 打开信任设备管理窗口
                    show_trusted_devices_window(app);
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

// 显示认证通知
pub fn show_auth_notification(app: &tauri::AppHandle, code: &str, device_name: &str) {
    let notification = format!("设备 \"{}\" 想要连接\n验证码: {}", device_name, code);
    
    app.notification()
        .builder()
        .title("CrossCopy - 设备连接请求")
        .body(&notification)
        .icon("auth-icon")
        .show()
        .unwrap();
}
```

## 5. 样式设计

### 5.1 CSS样式示例

```css
/* 认证对话框样式 */
.auth-dialog {
    width: 400px;
    padding: 24px;
    background: #ffffff;
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

.auth-dialog .header h2 {
    margin: 0 0 20px 0;
    color: #1a1a1a;
    font-size: 20px;
    font-weight: 600;
    text-align: center;
}

.verification-code {
    display: flex;
    justify-content: center;
    gap: 8px;
    margin: 24px 0;
}

.verification-code .digit {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 48px;
    background: #f5f5f5;
    border: 2px solid #e0e0e0;
    border-radius: 8px;
    font-size: 24px;
    font-weight: 600;
    color: #333;
    font-family: 'SF Mono', Monaco, monospace;
}

.expiry {
    text-align: center;
    color: #666;
    font-size: 14px;
    margin: 16px 0;
}

.actions {
    display: flex;
    gap: 12px;
    justify-content: center;
    margin-top: 24px;
}

.btn-primary {
    background: #007AFF;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
}

.btn-primary:hover {
    background: #0056CC;
}

.btn-secondary {
    background: #f5f5f5;
    color: #333;
    border: 1px solid #e0e0e0;
    padding: 10px 20px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
}

.btn-secondary:hover {
    background: #e8e8e8;
}

/* 验证码输入框样式 */
.code-input-container {
    display: flex;
    gap: 8px;
    justify-content: center;
    margin: 24px 0;
}

.code-input {
    width: 40px;
    height: 48px;
    text-align: center;
    font-size: 24px;
    font-weight: 600;
    border: 2px solid #e0e0e0;
    border-radius: 8px;
    background: #ffffff;
    font-family: 'SF Mono', Monaco, monospace;
    transition: border-color 0.2s;
}

.code-input:focus {
    outline: none;
    border-color: #007AFF;
    box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
}

.code-input.error {
    border-color: #FF3B30;
    background: #FFF5F5;
}

.error-message {
    color: #FF3B30;
    font-size: 14px;
    text-align: center;
    margin: 12px 0;
}

/* 信任设备管理样式 */
.trusted-devices-list {
    max-height: 300px;
    overflow-y: auto;
}

.device-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    margin-bottom: 8px;
    background: #ffffff;
}

.device-info {
    flex: 1;
}

.device-name {
    font-weight: 500;
    color: #1a1a1a;
    margin-bottom: 4px;
}

.device-status {
    font-size: 12px;
    color: #666;
}

.device-actions {
    display: flex;
    gap: 8px;
}

.btn-small {
    padding: 4px 8px;
    font-size: 12px;
    border-radius: 4px;
    border: 1px solid #e0e0e0;
    background: #f5f5f5;
    color: #333;
    cursor: pointer;
    transition: background 0.2s;
}

.btn-small:hover {
    background: #e8e8e8;
}

.btn-danger {
    color: #FF3B30;
    border-color: #FF3B30;
}

.btn-danger:hover {
    background: #FFF5F5;
}
```

## 6. 用户体验优化

### 6.1 交互优化
- **自动焦点管理**：验证码输入时自动切换焦点
- **键盘快捷键**：支持回车确认、ESC取消
- **剪贴板集成**：支持粘贴验证码
- **声音提示**：认证成功/失败的音效反馈

### 6.2 可访问性
- **高对比度支持**：适配系统主题
- **屏幕阅读器支持**：添加适当的ARIA标签
- **键盘导航**：完整的键盘操作支持
- **字体大小适配**：支持系统字体大小设置

### 6.3 国际化支持
- **多语言界面**：支持中英文切换
- **本地化格式**：时间、日期格式本地化
- **RTL支持**：为阿拉伯语等RTL语言做准备

这个UI设计方案提供了完整的用户界面解决方案，确保6位数字验证码认证系统具有良好的用户体验和视觉效果。
