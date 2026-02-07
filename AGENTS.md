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
| 图标 | @iconify/tailwind4 (mdi, lucide, carbon) |
| 终端 | xterm.js + xterm-addon-webgl + xterm-addon-fit |
| 可视化 | ECharts |
| 状态管理 | Pinia (待集成) |
| 后端 | Tauri v2 + tokio |
| SSH | russh + russh-keys |
| 文件传输 | russh-sftp |
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

### CSS 样式
- 优先使用 TailwindCSS 工具类
- 组件级样式使用 `<style scoped>`
- 图标使用 Iconify：`<span class="iconify mdi--icon-name"></span>`

### Rust 后端
- Tauri commands 使用 `#[tauri::command]` 宏
- 异步操作使用 `async/await`
- 错误处理：返回 `Result<T, String>` 给前端

## 美学 UI 规范

### 设计理念
- **macOS 风格**：整体视觉风格模仿 macOS，营造高端、现代的桌面体验
- **暗色主题优先**：深色背景 + 高对比度文字，减少视觉疲劳
- **毛玻璃美学**：大量使用 `backdrop-blur` 实现半透明毛玻璃效果
- **微动画**：所有交互都有细腻的过渡动画，提升用户体验

### 颜色系统

| 用途 | 颜色值 | 说明 |
|------|--------|------|
| 背景-深 | `#0a0a0a` / `neutral-950` | 桌面、窗口背景 |
| 背景-中 | `#171717` / `neutral-900` | 内容区域背景 |
| 背景-浅 | `#262626` / `neutral-800` | 工具栏、侧边栏 |
| 边框 | `white/10` ~ `white/15` | 半透明白色边框 |
| 文字-主 | `#e5e5e5` / `neutral-200` | 主要文字 |
| 文字-次 | `#a3a3a3` / `neutral-400` | 次要/辅助文字 |
| 文字-弱 | `#737373` / `neutral-500` | 禁用/占位符 |
| 强调色 | `#3b82f6` / `blue-500` | 选中、激活状态 |
| 成功 | `#22c55e` / `green-500` | 成功状态 |
| 警告 | `#f59e0b` / `amber-500` | 警告状态 |
| 危险 | `#ef4444` / `red-500` | 错误/删除操作 |

### 毛玻璃效果
```css
/* 轻度模糊 - 工具栏 */
backdrop-blur-sm    /* blur(4px) */
bg-neutral-800/60

/* 中度模糊 - 弹窗、面板 */
backdrop-blur-xl    /* blur(24px) */
bg-neutral-800/80

/* 重度模糊 - 右键菜单、悬浮框 */
backdrop-blur-2xl   /* blur(40px) */
bg-neutral-800/70
```

### 圆角规范
| 元素 | 圆角值 | TailwindCSS |
|------|--------|-------------|
| 窗口 | 18px | `rounded-[18px]` |
| 面板/卡片 | 16px | `rounded-2xl` |
| 模态框 | 16px | `rounded-2xl` |
| 按钮 | 8px | `rounded-lg` |
| 输入框 | 8px | `rounded-lg` |
| 图标容器 | 12px | `rounded-xl` |
| 标签/徽章 | 6px | `rounded-md` |

### 阴影规范
```css
/* 悬浮元素 - 右键菜单、下拉框 */
shadow-2xl
box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);

/* 模态框 */
shadow-2xl + 边框高光
box-shadow: 0 25px 50px rgba(0, 0, 0, 0.5);

/* 卡片 */
shadow-lg
box-shadow: 0 10px 15px rgba(0, 0, 0, 0.3);
```

### 动画规范
```css
/* 过渡时长 */
transition-150  /* 快速响应：hover、active */
transition-200  /* 标准过渡：展开、收起 */
transition-300  /* 平滑过渡：模态框、面板 */

/* 缓动函数 */
ease-out       /* 进入动画 */
ease-in        /* 退出动画 */
ease-in-out    /* 状态切换 */

/* 常用动画 */
hover:scale-105      /* 悬停放大 */
group-hover:scale-110 /* 组内图标放大 */
animate-spin         /* 加载旋转 */
```

### 组件样式规范

#### 按钮
```html
<!-- 主要按钮 -->
<button class="px-4 py-2 bg-blue-500 hover:bg-blue-600 rounded-lg transition-colors">

<!-- 次要按钮 -->
<button class="px-4 py-2 bg-white/10 hover:bg-white/20 rounded-lg transition-colors">

<!-- 图标按钮 -->
<button class="p-2 rounded-lg hover:bg-white/10 text-neutral-400 hover:text-white transition">
```

#### 输入框
```html
<input class="bg-black/30 border border-white/10 rounded-lg px-3 py-2 text-sm 
  outline-none focus:border-blue-500 focus:bg-black/50 transition placeholder-neutral-600" />
```

#### 右键菜单
- 背景：`bg-neutral-800/80 backdrop-blur-2xl`
- 边框：`border border-white/15`
- 圆角：`rounded-xl`
- 阴影：`shadow-2xl`
- 菜单项悬停：`hover:bg-blue-500` 渐变效果
- 分隔线：`bg-white/10` 渐变淡出
- 快捷键：右对齐，`text-neutral-500`

#### 窗口标题栏
- 红黄绿三色按钮（macOS 风格）
- 拖拽区域：`-webkit-app-region: drag`
- 按钮区：`-webkit-app-region: no-drag`

### 图标使用
- 使用 Iconify 图标库
- 首选：`mdi`（Material Design Icons）
- 类名格式：`icon-[mdi--icon-name]`
- 图标尺寸：`text-base`（16px）、`text-lg`（18px）、`text-xl`（20px）

## 文件结构

```
src/
├── components/     # Vue 组件
│   ├── MachineManager.vue   # 机器管理
│   ├── DesktopShell.vue     # 桌面环境
│   ├── Terminal.vue         # 终端组件
│   └── ConnectionPanel.vue  # 连接面板
├── assets/         # 静态资源
├── style.css       # 全局样式 + TailwindCSS
└── main.ts         # 入口文件

src-tauri/src/
├── lib.rs          # Tauri commands
├── db.rs           # SQLite 数据库操作
└── main.rs         # 入口
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