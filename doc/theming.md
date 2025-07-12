# 主题与样式系统

CrossCopy 基于 Tauri + React 架构，实现了完整的主题系统，支持浅色、深色和系统跟随三种主题模式。通过 CSS 变量和 React Context 的结合，实现了主题的实时切换和状态同步，确保在不同环境下的最佳视觉体验。

## 主题系统架构

### 1. 主题模式

CrossCopy 支持三种主题模式，满足不同用户的使用习惯：

- **浅色模式 (Light)**: 适合明亮环境使用，提供清晰的视觉对比
- **深色模式 (Dark)**: 适合暗光环境使用，减少眼部疲劳
- **系统模式 (System)**: 自动跟随操作系统的主题设置

### 2. 设计令牌 (Design Tokens)

#### 颜色系统

```css
/* 主色调 - 绿色系，体现同步和连接的概念 */
--primary-50: #E8F5E8;
--primary-100: #C8E6C9;
--primary-200: #A5D6A7;
--primary-300: #81C784;
--primary-400: #66BB6A;
--primary-500: #4CAF50;  /* 主色调 */
--primary-600: #43A047;
--primary-700: #388E3C;
--primary-800: #2E7D32;
--primary-900: #1B5E20;

/* 中性色系 */
--gray-50: #FAFAFA;
--gray-100: #F5F5F5;
--gray-200: #EEEEEE;
--gray-300: #E0E0E0;
--gray-400: #BDBDBD;
--gray-500: #9E9E9E;
--gray-600: #757575;
--gray-700: #616161;
--gray-800: #424242;
--gray-900: #212121;

/* 功能色系 */
--success-color: #4CAF50;
--warning-color: #FF9800;
--error-color: #F44336;
--info-color: #2196F3;
```

#### 语义化颜色变量

```css
/* 浅色主题 */
:root {
  /* 背景色 */
  --background-primary: var(--gray-50);
  --background-secondary: #FFFFFF;
  --background-tertiary: var(--gray-100);

  /* 文本色 */
  --text-primary: var(--gray-900);
  --text-secondary: var(--gray-600);
  --text-tertiary: var(--gray-500);
  --text-disabled: var(--gray-400);

  /* 边框色 */
  --border-primary: var(--gray-300);
  --border-secondary: var(--gray-200);

  /* 阴影 */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.1);

  /* 主题特定 */
  --primary-color: var(--primary-500);
  --primary-hover: var(--primary-600);
  --primary-active: var(--primary-700);
}

/* 深色主题 */
[data-theme="dark"] {
  /* 背景色 */
  --background-primary: #0F0F0F;
  --background-secondary: #1A1A1A;
  --background-tertiary: #262626;

  /* 文本色 */
  --text-primary: #FFFFFF;
  --text-secondary: #B3B3B3;
  --text-tertiary: #808080;
  --text-disabled: #4D4D4D;

  /* 边框色 */
  --border-primary: #404040;
  --border-secondary: #2D2D2D;

  /* 阴影 */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
  --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.4);
  --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.5);

  /* 主题特定 */
  --primary-color: var(--primary-400);
  --primary-hover: var(--primary-300);
  --primary-active: var(--primary-200);
}
```

### 3. 主题管理

#### React Context 实现

```typescript
// 主题类型定义
export type ThemeMode = 'light' | 'dark' | 'system';

export interface ThemeContextType {
  theme: ThemeMode;
  actualTheme: 'light' | 'dark';
  setTheme: (theme: ThemeMode) => void;
  toggleTheme: () => void;
}

// 主题上下文
export const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

// 主题提供者
export const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [theme, setThemeState] = useState<ThemeMode>('system');
  const [actualTheme, setActualTheme] = useState<'light' | 'dark'>('light');

  // 检测系统主题
  const getSystemTheme = useCallback((): 'light' | 'dark' => {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }, []);

  // 计算实际主题
  const calculateActualTheme = useCallback((themeMode: ThemeMode): 'light' | 'dark' => {
    return themeMode === 'system' ? getSystemTheme() : themeMode;
  }, [getSystemTheme]);

  // 设置主题
  const setTheme = useCallback(async (newTheme: ThemeMode) => {
    setThemeState(newTheme);

    const actual = calculateActualTheme(newTheme);
    setActualTheme(actual);

    // 应用主题到 DOM
    document.documentElement.setAttribute('data-theme', actual);

    // 保存到后端
    try {
      await invoke('update_theme_setting', { theme: newTheme });
    } catch (error) {
      console.error('Failed to save theme setting:', error);
    }
  }, [calculateActualTheme]);

  // 切换主题
  const toggleTheme = useCallback(() => {
    const newTheme = actualTheme === 'light' ? 'dark' : 'light';
    setTheme(newTheme);
  }, [actualTheme, setTheme]);

  // 监听系统主题变化
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

    const handleChange = () => {
      if (theme === 'system') {
        const newActualTheme = getSystemTheme();
        setActualTheme(newActualTheme);
        document.documentElement.setAttribute('data-theme', newActualTheme);
      }
    };

    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, [theme, getSystemTheme]);

  // 初始化主题
  useEffect(() => {
    const initTheme = async () => {
      try {
        // 从后端获取保存的主题设置
        const savedTheme = await invoke('get_theme_setting') as ThemeMode;
        setTheme(savedTheme);
      } catch (error) {
        // 如果获取失败，使用系统主题
        setTheme('system');
      }
    };

    initTheme();
  }, [setTheme]);

  const value: ThemeContextType = {
    theme,
    actualTheme,
    setTheme,
    toggleTheme,
  };

  return (
    <ThemeContext.Provider value={value}>
      {children}
    </ThemeContext.Provider>
  );
};

// 主题 Hook
export const useTheme = (): ThemeContextType => {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
};
```

### 4. Tauri 后端集成

```rust
// 主题设置管理
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[tauri::command]
pub async fn get_theme_setting(
    app_state: State<'_, AppState>,
) -> Result<ThemeMode, String> {
    let settings = app_state.settings_manager.lock().await;
    Ok(settings.get_theme())
}

#[tauri::command]
pub async fn update_theme_setting(
    theme: ThemeMode,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let mut settings = app_state.settings_manager.lock().await;

    match settings.set_theme(theme.clone()).await {
        Ok(_) => {
            // 发送主题更新事件
            app_state.emit_event("theme-updated", &theme).await;
            Ok(())
        }
        Err(e) => Err(format!("Failed to update theme: {}", e)),
    }
}

// 系统托盘图标主题适配
pub fn update_tray_icon_for_theme(app: &AppHandle, theme: &str) {
    let icon_path = match theme {
        "dark" => "icons/tray-icon-dark.png",
        _ => "icons/tray-icon-light.png",
    };

    if let Ok(icon) = tauri::Icon::File(std::path::PathBuf::from(icon_path)) {
        let _ = app.tray_handle().set_icon(icon);
    }
}
```

## 组件样式实现

### 1. 主题感知组件

```typescript
// 主题感知按钮组件
interface ThemedButtonProps {
  variant?: 'primary' | 'secondary' | 'ghost';
  size?: 'small' | 'medium' | 'large';
  children: React.ReactNode;
  onClick?: () => void;
}

const ThemedButton: React.FC<ThemedButtonProps> = ({
  variant = 'primary',
  size = 'medium',
  children,
  onClick
}) => {
  const { actualTheme } = useTheme();

  return (
    <button
      className={`themed-button themed-button--${variant} themed-button--${size}`}
      data-theme={actualTheme}
      onClick={onClick}
    >
      {children}
    </button>
  );
};
```

### 2. CSS 模块化样式

```scss
// ThemedButton.module.scss
.themed-button {
  // 基础样式
  border: none;
  border-radius: 6px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;

  // 使用 CSS 变量
  background-color: var(--primary-color);
  color: var(--text-primary);
  border: 1px solid var(--border-primary);

  // 悬停效果
  &:hover {
    background-color: var(--primary-hover);
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }

  &:active {
    background-color: var(--primary-active);
    transform: translateY(0);
  }

  // 变体样式
  &--primary {
    background-color: var(--primary-color);
    color: white;
    border-color: var(--primary-color);
  }

  &--secondary {
    background-color: var(--background-secondary);
    color: var(--text-primary);
    border-color: var(--border-primary);
  }

  &--ghost {
    background-color: transparent;
    color: var(--primary-color);
    border-color: var(--primary-color);
  }

  // 尺寸样式
  &--small {
    padding: 4px 8px;
    font-size: 12px;
  }

  &--medium {
    padding: 8px 16px;
    font-size: 14px;
  }

  &--large {
    padding: 12px 24px;
    font-size: 16px;
  }

  // 禁用状态
  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;

    &:hover {
      transform: none;
      box-shadow: none;
    }
  }
}
```

### 3. 动画和过渡

```scss
// 主题切换动画
.theme-transition {
  transition:
    background-color 0.3s ease,
    color 0.3s ease,
    border-color 0.3s ease,
    box-shadow 0.3s ease;
}

// 应用到所有元素
* {
  @extend .theme-transition;
}

// 特殊动画效果
.fade-in {
  animation: fadeIn 0.3s ease-in-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

// 主题切换时的闪烁效果
.theme-flash {
  animation: themeFlash 0.2s ease-in-out;
}

@keyframes themeFlash {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.8;
  }
}
```

### 4. 响应式设计

```scss
// 响应式断点
$breakpoints: (
  'mobile': 480px,
  'tablet': 768px,
  'desktop': 1024px,
  'wide': 1440px
);

@mixin respond-to($breakpoint) {
  @media (min-width: map-get($breakpoints, $breakpoint)) {
    @content;
  }
}

// 响应式组件
.responsive-container {
  padding: 16px;

  @include respond-to('tablet') {
    padding: 24px;
  }

  @include respond-to('desktop') {
    padding: 32px;
  }
}

// 响应式字体大小
.responsive-text {
  font-size: 14px;

  @include respond-to('tablet') {
    font-size: 16px;
  }

  @include respond-to('desktop') {
    font-size: 18px;
  }
}
```

## 主题定制

### 1. 自定义主题创建

```typescript
// 自定义主题接口
interface CustomTheme {
  name: string;
  colors: {
    primary: string;
    background: string;
    surface: string;
    text: string;
    border: string;
  };
  typography: {
    fontFamily: string;
    fontSize: {
      small: string;
      medium: string;
      large: string;
    };
  };
  spacing: {
    small: string;
    medium: string;
    large: string;
  };
}

// 主题注册器
class ThemeRegistry {
  private themes = new Map<string, CustomTheme>();

  register(theme: CustomTheme) {
    this.themes.set(theme.name, theme);
    this.generateCSSVariables(theme);
  }

  private generateCSSVariables(theme: CustomTheme) {
    const css = `
      [data-theme="${theme.name}"] {
        --primary-color: ${theme.colors.primary};
        --background-primary: ${theme.colors.background};
        --background-secondary: ${theme.colors.surface};
        --text-primary: ${theme.colors.text};
        --border-primary: ${theme.colors.border};
        --font-family: ${theme.typography.fontFamily};
        --font-size-small: ${theme.typography.fontSize.small};
        --font-size-medium: ${theme.typography.fontSize.medium};
        --font-size-large: ${theme.typography.fontSize.large};
        --spacing-small: ${theme.spacing.small};
        --spacing-medium: ${theme.spacing.medium};
        --spacing-large: ${theme.spacing.large};
      }
    `;

    // 动态插入样式
    const style = document.createElement('style');
    style.textContent = css;
    document.head.appendChild(style);
  }
}

export const themeRegistry = new ThemeRegistry();
```

### 2. 主题预设

```typescript
// 预设主题
const presetThemes: CustomTheme[] = [
  {
    name: 'ocean',
    colors: {
      primary: '#0077BE',
      background: '#F0F8FF',
      surface: '#FFFFFF',
      text: '#1A1A1A',
      border: '#E1E8ED',
    },
    typography: {
      fontFamily: 'Inter, sans-serif',
      fontSize: {
        small: '12px',
        medium: '14px',
        large: '16px',
      },
    },
    spacing: {
      small: '8px',
      medium: '16px',
      large: '24px',
    },
  },
  {
    name: 'sunset',
    colors: {
      primary: '#FF6B35',
      background: '#FFF8F0',
      surface: '#FFFFFF',
      text: '#2D1810',
      border: '#F0E6D6',
    },
    typography: {
      fontFamily: 'Roboto, sans-serif',
      fontSize: {
        small: '12px',
        medium: '14px',
        large: '16px',
      },
    },
    spacing: {
      small: '8px',
      medium: '16px',
      large: '24px',
    },
  },
];

// 注册预设主题
presetThemes.forEach(theme => themeRegistry.register(theme));
```

## 可访问性支持

### 1. 高对比度模式

```scss
// 高对比度模式检测
@media (prefers-contrast: high) {
  :root {
    --border-primary: #000000;
    --text-primary: #000000;
    --background-primary: #FFFFFF;
  }

  [data-theme="dark"] {
    --border-primary: #FFFFFF;
    --text-primary: #FFFFFF;
    --background-primary: #000000;
  }
}
```

### 2. 减少动画模式

```scss
// 减少动画偏好
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

### 3. 焦点管理

```scss
// 焦点样式
.focus-visible {
  outline: 2px solid var(--primary-color);
  outline-offset: 2px;
}

// 键盘导航支持
.keyboard-navigation {
  &:focus-visible {
    @extend .focus-visible;
  }
}
```