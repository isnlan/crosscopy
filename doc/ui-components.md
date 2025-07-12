# UI 组件设计

CrossCopy UI 基于 Tauri + React 架构，采用组件化设计理念，通过模块化的 React 组件构建直观、高效的用户界面。所有组件都与 Tauri 后端紧密集成，实现了前后端的无缝通信。

## 设计原则

- **组件化**: 每个功能模块都封装为独立的 React 组件
- **响应式**: 支持不同屏幕尺寸和分辨率的自适应布局
- **主题化**: 所有组件支持浅色/深色主题切换
- **可访问性**: 遵循 WCAG 2.1 可访问性标准
- **性能优化**: 使用 React.memo、useMemo 等优化渲染性能

## 主要视图组件

### 1. 剪贴板视图 (ClipboardView)

**功能特性**:
- 显示剪贴板历史记录，支持虚拟滚动优化性能
- 支持文本、图片、文件等多种内容类型预览
- 提供复制、删除、收藏、分享操作
- 支持实时搜索和多条件筛选
- 支持拖拽排序和批量操作

**Tauri 集成**:
```typescript
// 获取剪贴板历史
const { data: clipboardHistory } = useQuery('clipboard-history', async () => {
  return await invoke('get_clipboard_history', { limit: 100 });
});

// 监听剪贴板更新
useEffect(() => {
  const unlisten = listen('clipboard-updated', (event) => {
    queryClient.invalidateQueries('clipboard-history');
  });
  return () => unlisten.then(fn => fn());
}, []);
```

### 2. 设备视图 (DevicesView)

**功能特性**:
- 显示当前设备信息和网络状态
- 显示已连接和可用设备列表
- 提供连接/断开连接操作
- 支持手动添加设备和二维码配对
- 显示设备连接历史和统计信息

**Tauri 集成**:
```typescript
// 获取设备列表
const { data: devices } = useQuery('devices', async () => {
  return await invoke('get_devices');
});

// 连接设备
const connectDevice = useMutation(async (deviceId: string) => {
  return await invoke('connect_device', { deviceId });
});
```

### 3. 扫描视图 (ScanView)

**功能特性**:
- 提供二维码扫描功能（调用系统摄像头）
- 支持生成当前设备的连接二维码
- 快速配对新设备
- 支持手动输入连接码

**Tauri 集成**:
```typescript
// 生成连接二维码
const generateQRCode = async () => {
  const connectionInfo = await invoke('generate_connection_qr');
  setQRCodeData(connectionInfo);
};

// 处理扫描结果
const handleScanResult = async (result: string) => {
  await invoke('process_qr_scan', { data: result });
};
```

### 4. 设置视图 (SettingsView)

**功能特性**:
- 语言设置（支持多语言切换）
- 主题设置（浅色/深色/系统跟随）
- 剪贴板监听设置
- 网络和安全设置
- 通知和音效设置
- 开机启动和系统集成设置
- 高级选项和调试模式

**Tauri 集成**:
```typescript
// 获取设置
const { data: settings } = useQuery('settings', async () => {
  return await invoke('get_settings');
});

// 更新设置
const updateSettings = useMutation(async (newSettings: Settings) => {
  return await invoke('update_settings', { settings: newSettings });
});
```

## 共享组件

### 1. 设备卡片 (DeviceCard)

**功能特性**:
- 显示设备名称、操作系统图标、IP地址
- 实时显示连接状态（在线/离线/连接中）
- 提供连接/断开/重连操作按钮
- 显示连接质量和延迟信息
- 支持设备重命名和删除操作

**实现示例**:
```typescript
interface DeviceCardProps {
  device: Device;
  onConnect: (deviceId: string) => void;
  onDisconnect: (deviceId: string) => void;
}

const DeviceCard: React.FC<DeviceCardProps> = ({ device, onConnect, onDisconnect }) => {
  const [isConnecting, setIsConnecting] = useState(false);

  const handleConnect = async () => {
    setIsConnecting(true);
    try {
      await onConnect(device.id);
    } finally {
      setIsConnecting(false);
    }
  };

  return (
    <Card className="device-card">
      <div className="device-info">
        <OSIcon type={device.os} />
        <div>
          <h4>{device.name}</h4>
          <p>{device.ip_address}</p>
        </div>
      </div>
      <StatusBadge status={device.status} />
      <Button
        loading={isConnecting}
        onClick={device.connected ? () => onDisconnect(device.id) : handleConnect}
      >
        {device.connected ? '断开' : '连接'}
      </Button>
    </Card>
  );
};
```

### 2. 剪贴板项 (ClipboardItem)

**功能特性**:
- 支持文本、图片、文件等多种内容类型预览
- 显示复制时间、来源设备、内容大小
- 提供复制、删除、收藏、分享等快速操作
- 支持内容搜索高亮显示
- 支持拖拽操作和右键菜单

**实现示例**:
```typescript
interface ClipboardItemProps {
  item: ClipboardEntry;
  onCopy: (item: ClipboardEntry) => void;
  onDelete: (itemId: string) => void;
  onFavorite: (itemId: string) => void;
}

const ClipboardItem: React.FC<ClipboardItemProps> = ({
  item, onCopy, onDelete, onFavorite
}) => {
  const handleCopy = async () => {
    await invoke('copy_to_clipboard', { itemId: item.id });
    onCopy(item);
  };

  return (
    <div className="clipboard-item">
      <ContentPreview content={item.content} type={item.type} />
      <div className="item-meta">
        <span className="timestamp">{formatTime(item.timestamp)}</span>
        <span className="source">{item.source_device}</span>
      </div>
      <div className="item-actions">
        <Button icon={<CopyIcon />} onClick={handleCopy} />
        <Button icon={<StarIcon />} onClick={() => onFavorite(item.id)} />
        <Button icon={<DeleteIcon />} onClick={() => onDelete(item.id)} />
      </div>
    </div>
  );
};
```

### 3. 状态指示器 (StatusIndicator)

**功能特性**:
- 显示连接状态（已连接/未连接/连接中/错误）
- 显示同步状态（同步中/已同步/同步失败）
- 支持动画效果和状态转换
- 提供详细的状态信息提示
- 支持点击查看详细状态

**实现示例**:
```typescript
interface StatusIndicatorProps {
  status: 'connected' | 'disconnected' | 'connecting' | 'error';
  details?: string;
  animated?: boolean;
}

const StatusIndicator: React.FC<StatusIndicatorProps> = ({
  status, details, animated = true
}) => {
  const getStatusConfig = (status: string) => {
    switch (status) {
      case 'connected':
        return { color: 'green', icon: <CheckCircleIcon />, text: '已连接' };
      case 'connecting':
        return { color: 'blue', icon: <LoadingIcon spin={animated} />, text: '连接中' };
      case 'error':
        return { color: 'red', icon: <ErrorIcon />, text: '连接错误' };
      default:
        return { color: 'gray', icon: <DisconnectedIcon />, text: '未连接' };
    }
  };

  const config = getStatusConfig(status);

  return (
    <Tooltip title={details}>
      <div className={`status-indicator status-${status}`}>
        {config.icon}
        <span>{config.text}</span>
      </div>
    </Tooltip>
  );
};
```

### 4. 确认对话框 (ConfirmDialog)

**功能特性**:
- 用于危险操作的二次确认
- 支持自定义标题、消息和按钮文本
- 支持不同类型的警告级别
- 集成 Tauri 的原生对话框 API
- 支持键盘快捷键操作

**实现示例**:
```typescript
interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: string;
  type?: 'info' | 'warning' | 'error';
  onConfirm: () => void;
  onCancel: () => void;
}

const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
  open, title, message, type = 'warning', onConfirm, onCancel
}) => {
  // 使用 Tauri 原生对话框（可选）
  const showNativeDialog = async () => {
    const confirmed = await ask(message, { title, type });
    if (confirmed) {
      onConfirm();
    } else {
      onCancel();
    }
  };

  return (
    <Modal
      open={open}
      title={title}
      onCancel={onCancel}
      footer={[
        <Button key="cancel" onClick={onCancel}>
          取消
        </Button>,
        <Button key="confirm" type="primary" danger onClick={onConfirm}>
          确认
        </Button>,
      ]}
    >
      <div className={`confirm-dialog-content type-${type}`}>
        <WarningIcon />
        <p>{message}</p>
      </div>
    </Modal>
  );
};
```

## 组件通信模式

### 1. 父子组件通信
- 通过 props 传递数据和回调函数
- 使用 React.forwardRef 暴露子组件方法

### 2. 跨组件通信
- 使用 React Context 共享全局状态
- 使用自定义 Hooks 封装业务逻辑

### 3. 与 Tauri 后端通信
- 使用 `invoke` 调用后端命令
- 使用 `listen` 监听后端事件
- 使用 React Query 管理异步状态

### 4. 事件总线
```typescript
// 全局事件总线
export const eventBus = {
  emit: (event: string, data?: any) => {
    window.dispatchEvent(new CustomEvent(event, { detail: data }));
  },

  on: (event: string, callback: (data: any) => void) => {
    const handler = (e: CustomEvent) => callback(e.detail);
    window.addEventListener(event, handler);
    return () => window.removeEventListener(event, handler);
  }
};
```