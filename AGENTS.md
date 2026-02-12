---
trigger: always_on
---

# SynapSH 项目规范

## 技术栈

| 类别 | 技术 |
|------|------|
| 前端框架 | Vue 3.6 + TypeScript |
| 构建工具 | Vite 8 |
| CSS 框架 | TailwindCSS v4 |
| UI 组件 | shadcn-vue + Radix Vue |
| 图标 | @iconify/tailwind4 (mdi, lucide, carbon) |
| 终端 | xterm.js + xterm-addon-webgl + xterm-addon-fit |
| 可视化 | ECharts |
| 状态管理 | VueUse (主题/持久化) |
| 后端 | Tauri v2 + tokio |
| SSH | russh + russh-keys (SOCKS5 Zero-Copy) |
| 文件传输 | russh-sftp (并发流) |
| 本地数据库 | sqlx (SQLite) |

## 代码规范

### 文件头注释
- 每个源文件顶部必须添加注释，说明该文件的作用
- Vue 组件：使用 `<!-- 文件说明 -->` 格式
- TypeScript/JavaScript：使用 `// 文件说明` 或 `/** 文件说明 */` 格式
- Rust：使用 `//! 文件说明` 或 `/// 文件说明` 格式
- CSS：使用 `/* 文件说明 */` 格式

### Vue 组件
- 使用 `<script setup lang="ts">` 组合式 API
- Props 使用 TypeScript 接口定义
- 使用 `defineEmits` 定义事件
- 使用 shadcn-vue 组件


### CSS 样式
- 优先使用 TailwindCSS 工具类
- 组件级样式使用 `<style scoped>`
- 图标使用 Iconify：`<span class="iconify mdi--icon-name"></span>`

### Rust 后端
- Tauri commands 使用 `#[tauri::command]` 宏
- 异步操作使用 `async/await`
- 错误处理：返回 `Result<T, String>` 给前端
- **并发控制**：避免全局锁，使用 Cloneable Handle + Channel
- **网络 IO**：使用 `tokio::io::copy_bidirectional` 实现零拷贝转发

## 美学 UI 规范

### 设计基调（重新定义）
- **Neo-macOS 桌面感**：保留 macOS 的精致秩序感，但强调更明确的信息层级与功能导向
- **暗色优先 + 亮色兼容**：默认暗色主题，亮色作为可扩展主题，不允许仅在亮色下可读
- **内容优先，装饰克制**：视觉效果服务于可读性与操作反馈，不做无意义炫技
- **动效传达状态**：动画只表达“出现/消失/切换/反馈”，禁止无语义运动

### 视觉层级（从下到上）
- `Layer 0` 桌面背景：允许渐变与弱纹理，但对比度必须低，避免干扰窗口内容
- `Layer 1` 常规容器：主工作区、侧边栏、工具栏，强调稳定与低噪声
- `Layer 2` 活跃窗口：当前焦点窗口需要通过边框高光与阴影区分
- `Layer 3` 浮层组件：菜单、下拉、Tooltip、Popover，必须高于窗口并具备明显分离感
- `Layer 4` 阻断层：Modal、确认框，使用遮罩与聚焦动效锁定注意力

### 色彩 Token（语义化）

| Token | 颜色值 | 用途 |
|------|--------|------|
| `--bg-canvas` | `#0b0d10` | 桌面/全局背景 |
| `--bg-surface` | `#151922` | 主内容面板 |
| `--bg-elevated` | `#1d2430` | 卡片/浮层背景 |
| `--bg-active` | `#263042` | Hover/选中容器底色 |
| `--border-subtle` | `rgba(255,255,255,0.10)` | 常规边框 |
| `--border-strong` | `rgba(255,255,255,0.18)` | 焦点/分割线强化 |
| `--text-primary` | `#f5f7fa` | 主文字 |
| `--text-secondary` | `#c3c9d4` | 次级文字 |
| `--text-tertiary` | `#8e97a8` | 辅助信息 |
| `--accent` | `#0a84ff` | 主强调/可交互高亮 |
| `--success` | `#30d158` | 成功状态 |
| `--warning` | `#ff9f0a` | 警告状态 |
| `--danger` | `#ff453a` | 危险/删除 |

### 空间与圆角规范
- 使用 `8pt` 栅格系统：`4 / 8 / 12 / 16 / 24 / 32`
- 紧凑信息区最小垂直间距 `8px`，卡片内部推荐 `12px` 或 `16px`
- 可点击目标最小尺寸 `32x32`，高频按钮推荐 `36x36`

| 元素 | 圆角值 | TailwindCSS |
|------|--------|-------------|
| 桌面窗口 | 18px | `rounded-[18px]` |
| 面板/卡片 | 16px | `rounded-2xl` |
| 模态框/浮层 | 14px | `rounded-xl` |
| 按钮/输入框 | 8px | `rounded-lg` |
| 图标容器 | 10px | `rounded-[10px]` |
| Tag/Badge | 6px | `rounded-md` |

### 材质与阴影规范
- 面板允许轻微毛玻璃：`backdrop-blur-xl` + `bg-black/35`（或等价暗色透明）
- 非活跃窗口降低对比：背景亮度下调、边框透明度下调

| 场景 | Tailwind 建议 | 阴影参考 |
|------|---------------|---------|
| 卡片/普通悬浮 | `shadow-lg` | `0 10px 24px rgba(0,0,0,0.30)` |
| 菜单/下拉/右键 | `shadow-2xl` | `0 16px 40px rgba(0,0,0,0.45)` |
| Modal/关键弹层 | `shadow-2xl` + 边框高光 | `0 24px 64px rgba(0,0,0,0.55)` |

### 动效规范（语义驱动）
- 悬停反馈：`120ms`，用于按钮、图标、列表项
- 状态切换：`180ms`，用于选中、展开、Tab 切换
- 结构过渡：`260ms`，用于窗口、Modal、面板进出

```css
/* 推荐缓动 */
ease-out: cubic-bezier(0.22, 1, 0.36, 1);   /* 进入 */
ease-in: cubic-bezier(0.32, 0, 0.67, 0);    /* 退出 */
ease-in-out: cubic-bezier(0.65, 0, 0.35, 1);/* 状态切换 */
```

### 交互状态规范
- `hover`：仅增强可见性，不改变布局
- `active`：可使用 `scale-[0.98]` 或亮度降低，持续时间短于 `120ms`
- `focus-visible`：必须显示 `2px` 强调环（颜色使用 `--accent`）
- `disabled`：降低对比并禁止阴影/位移动效
- `loading`：优先局部骨架屏或按钮内加载，不阻断整个页面

### 字体与图标规范
- 字体优先级：`SF Pro Display/Text` -> `PingFang SC` -> `Inter` -> `sans-serif`
- 数字信息（CPU/内存/流量）推荐使用等宽数字风格，避免跳动感
- 图标使用 Iconify，首选 `mdi`，类名格式：`icon-[mdi--icon-name]`
- 图标尺寸基准：`16 / 18 / 20`，同一行内禁止混用超过两种尺寸

### 可访问性与约束
- 正文与背景对比度不低于 `4.5:1`
- 关键状态不能只靠颜色区分，必须有形状或文案辅助
- 支持 `prefers-reduced-motion`，在低动效模式下降级动画
- 禁止高饱和霓虹色大面积铺底
- 禁止同时使用超过 2 种强调色

## 文件结构

```
src/
├── views/              # 页面和应用
│   ├── MachineManager.vue   # 机器管理页
│   ├── DesktopShell.vue     # 桌面环境页
│   └── apps/                # 桌面内应用
│       ├── FilesApp.vue
│       ├── ActivityMonitor.vue
│       ├── TextEditorApp.vue
│       ├── SettingsApp.vue
│       └── BrowserApp.vue
├── components/         # 可复用组件
│   ├── desktop/        # 桌面 UI 组件
│   │   ├── AppWindow.vue
│   │   ├── DesktopDock.vue
│   │   └── DesktopIcon.vue
│   ├── Terminal.vue
│   ├── TabBar.vue
│   ├── ConnectionPanel.vue
│   └── ui/             # shadcn-vue 组件
├── assets/             # 静态资源
├── style.css           # 全局样式 + TailwindCSS
└── main.ts             # 入口文件

src-tauri/src/
├── lib.rs              # Tauri commands
├── db.rs               # SQLite 数据库操作
├── ssh.rs              # SSH 会话/SOCKS5 代理/SFTP
└── main.rs             # 入口
```

## 运行命令

```bash
# 开发
pnpm tauri dev

# 构建
pnpm tauri build
```

## 工作流规则

### 进度管理
- 完成任务后必须更新 `PROGRESS.md` 文件
- 将已完成的任务从 `[ ]` 改为 `[x]`
- 进行中的任务标记为 `[/]`
- 如有新增功能，添加对应的任务项

### 技能使用
- 如有合适的技能，并遵循其指导
