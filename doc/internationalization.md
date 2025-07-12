# 国际化支持

CrossCopy UI 提供完整的国际化支持，初始阶段支持简体中文和英文，后续可扩展支持更多语言。

## 国际化架构

1. **翻译资源**
   - 使用 JSON 格式存储翻译文本
   - 按语言和功能模块组织
   - 支持变量插值和复数形式

2. **语言切换**
   - 实时切换无需重启
   - 保存用户语言偏好
   - 默认跟随系统语言

3. **日期和数字格式化**
   - 根据语言环境格式化日期
   - 支持不同数字格式和单位

## 实现方式

### 翻译资源文件

```json
// zh-CN.json
{
  "app": {
    "name": "CrossCopy",
    "slogan": "跨平台剪贴板同步工具"
  },
  "tabs": {
    "clipboard": "粘贴板",
    "devices": "设备",
    "scan": "扫描"
  },
  "settings": {
    "title": "设置",
    "language": "语言",
    "theme": "主题",
    "themes": {
      "light": "浅色",
      "dark": "深色",
      "system": "跟随系统"
    },
    "clipboard": {
      "monitoring": "粘贴板监听",
      "encryption": "加密同步",
      "sounds": "交互音效",
      "autostart": "开机启动",
      "debug": "调试模式"
    }
  },
  "devices": {
    "myDevice": "我的设备",
    "addDevice": "手动添加设备",
    "nearbyDevices": "附近的设备",
    "noDevices": "未找到启用 CrossCopy 的附近设备",
    "status": {
      "connected": "已连接",
      "disconnected": "未连接",
      "connecting": "连接中",
      "failed": "连接失败"
    }
  }
}
```

### 国际化 Hook

```typescript
import { useEffect, useState } from 'react';
import zhCN from '../locales/zh-CN.json';
import enUS from '../locales/en-US.json';

// 支持的语言
const locales = {
  'zh-CN': zhCN,
  'en-US': enUS,
};

// 国际化 Hook
export const useI18n = () => {
  const [locale, setLocale] = useState(() => {
    // 从存储中获取用户偏好语言，默认使用系统语言
    const savedLocale = localStorage.getItem('locale');
    if (savedLocale && locales[savedLocale]) {
      return savedLocale;
    }
    
    // 检测系统语言
    const browserLocale = navigator.language;
    return locales[browserLocale] ? browserLocale : 'en-US';
  });
  
  // 翻译函数
  const t = (key: string, variables?: Record<string, string>) => {
    const keys = key.split('.');
    let value = locales[locale];
    
    // 遍历嵌套键
    for (const k of keys) {
      if (!value[k]) {
        return key; // 如果找不到翻译，返回键名
      }
      value = value[k];
    }
    
    // 处理变量
    if (variables && typeof value === 'string') {
      return Object.entries(variables).reduce(
        (text, [key, val]) => text.replace(`{${key}}`, val),
        value
      );
    }
    
    return value;
  };
  
  // 切换语言
  const changeLocale = (newLocale: string) => {
    if (locales[newLocale]) {
      setLocale(newLocale);
      localStorage.setItem('locale', newLocale);
      document.documentElement.setAttribute('lang', newLocale);
    }
  };
  
  return { locale, t, changeLocale };
};
```