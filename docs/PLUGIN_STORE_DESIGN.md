# SynapSH 应用商店系统设计文档

## 目录

1. [系统概述](#系统概述)
2. [插件开发规范](#插件开发规范)
3. [安全沙箱机制](#安全沙箱机制)
4. [插件审核与发布流程](#插件审核与发布流程)
5. [版本管理与更新机制](#版本管理与更新机制)
6. [用户评价与反馈系统](#用户评价与反馈系统)
7. [收益分成模式](#收益分成模式)
8. [技术实现架构](#技术实现架构)

---

## 系统概述

### 设计目标

SynapSH 应用商店是一个开放的插件生态系统，允许第三方开发者为 SynapSH 创建扩展应用，同时确保：

- **安全性**：插件运行在隔离的沙箱环境中，无法直接访问系统资源
- **稳定性**：插件崩溃不影响主程序运行
- **易用性**：一键安装、自动更新、无缝集成
- **公平性**：透明的审核流程和收益分成

### 架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                      SynapSH 主程序                              │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Plugin Host (插件宿主)                      │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │   │
│  │  │  App Store  │  │  Sandbox    │  │  API Bridge │     │   │
│  │  │   UI        │  │   Engine    │  │             │     │   │
│  │  └─────────────┘  └──────┬──────┘  └─────────────┘     │   │
│  └──────────────────────────┼──────────────────────────────┘   │
└──────────────────────────────┼─────────────────────────────────┘
                               │
                    ┌──────────┴──────────┐
                    │   WebView Isolate   │
                    │   (插件运行环境)     │
                    └─────────────────────┘
```

---

## 插件开发规范

### 1. 插件结构

```
my-plugin/
├── plugin.json           # 插件元数据
├── package.json          # NPM 依赖（可选）
├── dist/
│   ├── index.html        # 插件入口
│   └── assets/
├── src/
│   ├── main.ts           # 插件主逻辑
│   ├── components/       # Vue 组件
│   └── styles/           # 样式文件
└── docs/
    ├── README.md         # 文档
    └── CHANGELOG.md      # 更新日志
```

### 2. 插件元数据 (plugin.json)

```typescript
interface PluginManifest {
  // 基础信息
  id: string; // 唯一标识符 (反向域名格式: com.example.plugin)
  name: string; // 显示名称
  version: string; // 语义化版本 (遵循 semver)
  description: string; // 简短描述
  author: string; // 作者信息

  // 分类与标签
  category: PluginCategory; // 插件分类
  tags: string[]; // 标签用于搜索

  // 入口配置
  entry: string; // 入口文件路径 (相对路径)
  icon: string; // 图标路径

  // 权限声明
  permissions: Permission[]; // 所需权限列表

  // 兼容性
  minAppVersion: string; // 最低 SynapSH 版本要求
  supportedPlatforms: Platform[]; // 支持的平台

  // 商店信息
  screenshots: string[]; // 截图路径
  pricing: PricingInfo; // 定价信息

  // 本地化
  i18n: Record<string, I18nInfo>; // 多语言支持
}

type PluginCategory =
  | "terminal" // 终端扩展
  | "file-manager" // 文件管理
  | "monitoring" // 系统监控
  | "productivity" // 生产力工具
  | "development" // 开发工具
  | "database" // 数据库工具
  | "network" // 网络工具
  | "security" // 安全工具
  | "theme" // 主题美化
  | "utility"; // 其他工具

type Permission =
  | "ssh:read" // 读取 SSH 会话信息
  | "ssh:write" // 执行 SSH 命令
  | "sftp:read" // 读取远程文件
  | "sftp:write" // 写入远程文件
  | "fs:temp" // 访问临时文件系统
  | "fs:appdata" // 访问应用数据目录
  | "network:http" // HTTP 请求
  | "network:ws" // WebSocket
  | "ui:dock" // 添加到 Dock
  | "ui:menubar" // 添加到菜单栏
  | "ui:statusbar" // 添加到状态栏
  | "system:notification"; // 发送系统通知
```

### 3. JavaScript API 规范

```typescript
// 全局命名空间: synapsh
declare namespace synapsh {
  // 版本信息
  const version: string;
  const apiVersion: string;

  // 上下文信息
  const pluginId: string;
  const pluginVersion: string;

  // SSH 相关 API
  namespace ssh {
    function getActiveSessions(): Promise<SessionInfo[]>;
    function executeCommand(sessionId: string, command: string): Promise<string>;
    function onData(sessionId: string, callback: (data: string) => void): Unsubscriber;
  }

  // 文件传输 API
  namespace sftp {
    function readFile(sessionId: string, path: string): Promise<Uint8Array>;
    function writeFile(sessionId: string, path: string, data: Uint8Array): Promise<void>;
    function listDirectory(sessionId: string, path: string): Promise<FileEntry[]>;
  }

  // UI 集成 API
  namespace ui {
    function showNotification(options: NotificationOptions): void;
    function showDialog(options: DialogOptions): Promise<DialogResult>;
    function registerDockItem(item: DockItemConfig): void;
    function setBadge(count: number): void;
  }

  // 存储 API
  namespace storage {
    function get<T>(key: string): Promise<T | undefined>;
    function set<T>(key: string, value: T): Promise<void>;
    function remove(key: string): Promise<void>;
  }

  // 网络 API
  namespace network {
    function fetch(url: string, options?: RequestInit): Promise<Response>;
    function createWebSocket(url: string): WebSocket;
  }

  // 事件系统
  namespace events {
    function on(event: string, callback: (...args: any[]) => void): Unsubscriber;
    function emit(event: string, ...args: any[]): void;
  }

  // 日志系统
  namespace logger {
    function debug(...args: any[]): void;
    function info(...args: any[]): void;
    function warn(...args: any[]): void;
    function error(...args: any[]): void;
  }
}
```

---

## 安全沙箱机制

### 1. 多层安全架构

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: Content Security Policy                            │
│ - 严格 CSP 策略限制资源加载                                  │
│ - 禁止内联脚本和 eval()                                     │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: API 权限控制                                        │
│ - 能力模型 (Capability-based)                                │
│ - 运行时权限检查                                             │
│ - API 调用审计日志                                           │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: WebView 隔离                                        │
│ - 独立进程运行                                               │
│ - 无 Node.js 访问权限                                        │
│ - 受限的浏览器环境                                           │
└─────────────────────────────────────────────────────────────┘
```

### 2. 权限系统

```typescript
// 权限定义
interface PermissionGrant {
  permission: string;
  granted: boolean;
  scope?: PermissionScope;
  expiresAt?: number; // 可选的过期时间
}

// 权限范围限制
interface PermissionScope {
  // 例如: sftp:write 权限可以限制到特定目录
  paths?: string[];
  hosts?: string[];
  // 速率限制
  rateLimit?: {
    requestsPerMinute: number;
    burstSize: number;
  };
}

// 权限申请流程
class PermissionManager {
  // 申请权限 (会触发用户确认对话框)
  async requestPermission(pluginId: string, permission: string, reason?: string): Promise<boolean>;

  // 检查权限
  async checkPermission(pluginId: string, permission: string): Promise<boolean>;

  // 撤销权限
  async revokePermission(pluginId: string, permission: string): Promise<void>;
}
```

### 3. 代码签名验证

```typescript
// 插件包签名验证
interface PluginSignature {
  algorithm: "ed25519" | "rsa-pss";
  publicKey: string;
  signature: string;
  timestamp: number;
  certificate?: DeveloperCertificate;
}

// 验证流程
async function verifyPlugin(packagePath: string): Promise<VerificationResult> {
  // 1. 验证包完整性 (SHA-256 校验)
  // 2. 验证开发者签名
  // 3. 检查证书吊销列表 (CRL)
  // 4. 验证商店签名 (可选，用于官方审核插件)
}
```

### 4. 资源限制

```typescript
interface ResourceLimits {
  // 内存限制
  maxMemoryMB: number;
  // CPU 使用率限制 (百分比)
  maxCpuPercent: number;
  // 存储限制
  maxStorageMB: number;
  // 网络请求限制
  maxRequestsPerMinute: number;
  // 并发连接数
  maxConcurrentConnections: number;
}

// 默认限制
const DEFAULT_LIMITS: ResourceLimits = {
  maxMemoryMB: 128,
  maxCpuPercent: 10,
  maxStorageMB: 100,
  maxRequestsPerMinute: 60,
  maxConcurrentConnections: 5,
};
```

---

## 插件审核与发布流程

### 1. 开发者注册

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   邮箱注册    │ -> │  身份验证    │ -> │  开发者认证   │
└──────────────┘    └──────────────┘    └──────────────┘
                                              │
                                              v
                                       ┌──────────────┐
                                       │  生成密钥对   │
                                       │ (用于签名)    │
                                       └──────────────┘
```

### 2. 插件提交流程

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  本地开发     │ -> │  打包签名    │ -> │  提交审核    │ -> │  自动检测    │
│  测试通过     │    │  (cli工具)   │    │              │    │  (安全扫描)  │
└──────────────┘    └──────────────┘    └──────────────┘    └──────┬───────┘
                                                                    │
                                                                    v
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   正式发布    │ <- │  人工审核    │ <- │  通过?       │ <- │  静态分析    │
│              │    │              │    │              │    │  行为分析    │
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘
                                              │
                                              v (否)
                                       ┌──────────────┐
                                       │  拒绝并反馈   │
                                       └──────────────┘
```

### 3. 自动化检测规则

```typescript
interface SecurityRule {
  id: string;
  name: string;
  severity: "critical" | "high" | "medium" | "low";
  description: string;

  // 静态分析规则
  pattern?: RegExp; // 正则匹配危险代码
  forbiddenImports?: string[]; // 禁止的模块

  // 行为分析规则
  sandboxTest?: (context: SandboxContext) => Promise<boolean>;
}

// 关键检测规则
const CRITICAL_RULES: SecurityRule[] = [
  {
    id: "NO_EVAL",
    name: "禁止使用 eval",
    severity: "critical",
    pattern: /\beval\s*\(/,
  },
  {
    id: "NO_DYNAMIC_IMPORT",
    name: "禁止动态导入",
    severity: "high",
    pattern: /import\s*\(/,
  },
  {
    id: "NO_CHILD_PROCESS",
    name: "禁止子进程",
    forbiddenImports: ["child_process", "node:child_process"],
  },
  {
    id: "NO_FILE_SYSTEM",
    name: "禁止直接文件系统访问",
    forbiddenImports: ["fs", "node:fs", "fs/promises"],
  },
];
```

### 4. 审核状态流转

```
DRAFT -> SUBMITTED -> PENDING_REVIEW ->
  ├─> APPROVED -> PUBLISHED
  ├─> REJECTED -> (修改后) -> SUBMITTED
  └─> SUSPENDED (发布后发现问题)
```

---

## 版本管理与更新机制

### 1. 语义化版本控制

```typescript
// 遵循 semver: MAJOR.MINOR.PATCH
interface VersionInfo {
  version: string; // 当前版本
  minAppVersion: string; // 最低 SynapSH 版本
  changelog: ChangelogEntry[];
  downloadUrl: string;
  checksum: string; // SHA-256 校验
  size: number; // 包大小 (bytes)
  releasedAt: string; // ISO 8601 日期
}

interface ChangelogEntry {
  type: "feature" | "fix" | "security" | "breaking";
  description: string;
  issueId?: string;
}
```

### 2. 更新策略

```typescript
interface UpdatePolicy {
  // 自动更新设置
  autoUpdate: boolean;
  // 更新频道
  channel: "stable" | "beta" | "alpha";
  // 更新时间窗口
  updateWindow?: {
    start: string; // "02:00"
    end: string; // "06:00"
  };
  // 延迟更新 (小时)
  delayHours?: number;
}

// 更新检查流程
class UpdateManager {
  async checkForUpdates(pluginId: string): Promise<UpdateInfo | null>;
  async downloadUpdate(pluginId: string, version: string): Promise<void>;
  async installUpdate(pluginId: string): Promise<void>;
  async rollback(pluginId: string): Promise<void>; // 回滚到上一版本
}
```

### 3. 热更新机制

```typescript
// 无需重启的热更新
interface HotUpdateConfig {
  enabled: boolean;
  // 支持的更新类型
  supportedTypes: ("assets" | "styles" | "config")[];
  // 最大热更新包大小
  maxSize: number;
}

// 热更新流程
async function applyHotUpdate(pluginId: string, update: HotUpdate) {
  // 1. 下载更新包
  // 2. 验证签名和完整性
  // 3. 备份当前版本
  // 4. 应用更新
  // 5. 通知插件重新加载
  // 6. 验证运行状态
  // 7. 成功则清理备份，失败则回滚
}
```

### 4. 版本兼容性矩阵

```typescript
interface CompatibilityMatrix {
  appVersion: string;
  pluginVersions: Record<string, string>; // pluginId -> max supported version
}

// 示例
const COMPATIBILITY: CompatibilityMatrix[] = [
  {
    appVersion: "2.0.0",
    pluginVersions: {
      "com.example.terminal-theme": "1.5.x",
      "com.example.ssh-toolkit": "2.x",
    },
  },
];
```

---

## 用户评价与反馈系统

### 1. 评分系统

```typescript
interface Review {
  id: string;
  pluginId: string;
  userId: string;

  // 评分 (1-5 星)
  rating: number;

  // 评价内容
  title?: string;
  content?: string;

  // 版本信息
  pluginVersion: string;
  appVersion: string;

  // 元数据
  createdAt: string;
  updatedAt: string;
  helpfulCount: number; // 认为有帮助的用户数

  // 开发者回复
  developerReply?: {
    content: string;
    repliedAt: string;
  };
}

// 评分统计
interface RatingStats {
  average: number; // 平均分
  totalCount: number; // 总评价数
  distribution: {
    // 分布
    5: number;
    4: number;
    3: number;
    2: number;
    1: number;
  };
}
```

### 2. 防刷机制

```typescript
interface AntiFraudConfig {
  // 同一用户只能评价一次
  oneReviewPerUser: boolean;
  // 必须使用过插件才能评价
  mustHaveUsed: boolean;
  // 最小使用时长 (分钟)
  minUsageTime: number;
  // 评价冷却期 (天)
  reviewCooldownDays: number;
}

// 可疑行为检测
interface FraudDetectionRule {
  id: string;
  name: string;
  detect: (reviews: Review[]) => SuspiciousActivity[];
}

// 示例规则
const FRAUD_RULES: FraudDetectionRule[] = [
  {
    id: "RATING_SPIKE",
    name: "评分激增检测",
    detect: (reviews) => {
      // 检测短时间内大量 5 星评价
    },
  },
  {
    id: "SAME_CONTENT",
    name: "重复内容检测",
    detect: (reviews) => {
      // 检测内容相似度
    },
  },
];
```

### 3. 反馈收集系统

```typescript
interface Feedback {
  id: string;
  pluginId: string;
  type: "bug" | "feature" | "performance" | "security";

  // 基本信息
  title: string;
  description: string;

  // 环境信息 (自动收集)
  environment: {
    appVersion: string;
    pluginVersion: string;
    os: string;
    osVersion: string;
  };

  // 状态流转
  status: "open" | "investigating" | "resolved" | "closed";
  priority?: "low" | "medium" | "high" | "critical";

  // 关联
  assignee?: string; // 分配的开发者
  relatedIssues?: string[]; // 关联问题
}

// 崩溃报告
interface CrashReport {
  pluginId: string;
  error: {
    message: string;
    stack?: string;
    source: string;
  };
  context: {
    memoryUsage: number;
    cpuUsage: number;
    uptime: number;
  };
}
```

---

## 收益分成模式

### 1. 商业模式

```typescript
type PricingModel =
  | { type: "free" }
  | { type: "paid"; price: number; currency: string }
  | { type: "subscription"; monthlyPrice: number; yearlyPrice?: number }
  | { type: "freemium"; freeFeatures: string[]; proPrice: number };

// 交易记录
interface Transaction {
  id: string;
  type: "purchase" | "refund" | "subscription_start" | "subscription_renew" | "subscription_cancel";
  pluginId: string;
  userId: string;
  amount: number;
  currency: string;
  platformFee: number; // 平台抽成
  developerRevenue: number; // 开发者收入
  timestamp: string;
}
```

### 2. 分成比例

| 类型         | 平台抽成 | 开发者收入 | 说明         |
| ------------ | -------- | ---------- | ------------ |
| 免费插件     | 0%       | 0%         | 完全免费     |
| 付费插件     | 20%      | 80%        | 一次性购买   |
| 订阅制       | 15%      | 85%        | 首年         |
| 订阅制(续费) | 10%      | 90%        | 续费奖励     |
| 捐赠模式     | 5%       | 95%        | 用户自愿捐赠 |

### 3. 结算系统

```typescript
interface Payout {
  developerId: string;
  period: {
    start: string;
    end: string;
  };
  sales: {
    total: number;
    refunds: number;
    net: number;
  };
  revenue: {
    gross: number; // 总收入
    platformFee: number;
    tax: number; // 预扣税
    net: number; // 净收入
  };
  status: "pending" | "processing" | "completed";
  payoutMethod: "bank_transfer" | "paypal" | "crypto";
}

// 结算周期
const PAYOUT_SCHEDULE = {
  minimumAmount: 100, // 最低结算金额 (USD)
  frequency: "monthly", // 结算频率
  delayDays: 30, // 账期 (退款窗口)
};
```

---

## 技术实现架构

### 1. 数据库设计

```sql
-- 插件表
CREATE TABLE plugins (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  author_id TEXT NOT NULL,
  category TEXT NOT NULL,
  current_version TEXT NOT NULL,
  download_count INTEGER DEFAULT 0,
  rating_average REAL DEFAULT 0,
  rating_count INTEGER DEFAULT 0,
  status TEXT DEFAULT 'pending', -- pending, approved, rejected, suspended
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 插件版本表
CREATE TABLE plugin_versions (
  id TEXT PRIMARY KEY,
  plugin_id TEXT NOT NULL,
  version TEXT NOT NULL,
  manifest TEXT NOT NULL, -- JSON
  package_url TEXT NOT NULL,
  checksum TEXT NOT NULL,
  min_app_version TEXT,
  changelog TEXT,
  status TEXT DEFAULT 'draft',
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (plugin_id) REFERENCES plugins(id)
);

-- 已安装插件表
CREATE TABLE installed_plugins (
  id TEXT PRIMARY KEY,
  plugin_id TEXT NOT NULL,
  installed_version TEXT NOT NULL,
  install_path TEXT NOT NULL,
  permissions TEXT, -- JSON array of granted permissions
  settings TEXT, -- JSON
  is_enabled BOOLEAN DEFAULT true,
  installed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(plugin_id)
);

-- 评价表
CREATE TABLE reviews (
  id TEXT PRIMARY KEY,
  plugin_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
  title TEXT,
  content TEXT,
  plugin_version TEXT,
  helpful_count INTEGER DEFAULT 0,
  developer_reply TEXT,
  developer_replied_at DATETIME,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(plugin_id, user_id)
);
```

### 2. 后端架构

```rust
// src-tauri/src/plugin/
mod manifest;      // 插件元数据解析
mod sandbox;       // 沙箱实现
mod permissions;   // 权限管理
mod store;         // 商店 API
mod updater;       // 更新管理

// 插件运行时
pub struct PluginRuntime {
    webview: Webview,
    manifest: PluginManifest,
    permissions: PermissionSet,
    message_channel: Channel,
}

// 插件管理器
pub struct PluginManager {
    plugins: HashMap<String, PluginInstance>,
    db: Database,
    sandbox: SandboxEngine,
}
```

### 3. 前端架构

```typescript
// src/composables/usePlugins.ts
export function usePlugins() {
  const installedPlugins = ref<InstalledPlugin[]>([]);

  async function installPlugin(pluginId: string): Promise<void>;
  async function uninstallPlugin(pluginId: string): Promise<void>;
  async function enablePlugin(pluginId: string): Promise<void>;
  async function disablePlugin(pluginId: string): Promise<void>;
  async function updatePlugin(pluginId: string): Promise<void>;

  return {
    installedPlugins,
    installPlugin,
    uninstallPlugin,
    enablePlugin,
    disablePlugin,
    updatePlugin,
  };
}

// src/components/AppStore/
// - AppStore.vue          # 应用商店主页
// - PluginCard.vue        # 插件卡片
// - PluginDetail.vue      # 插件详情
// - PluginInstall.vue     # 安装确认
// - DeveloperPortal.vue   # 开发者门户
```

---

## 开发者工具

### 1. CLI 工具

```bash
# 安装开发者工具
npm install -g @synapsh/plugin-cli

# 创建新项目
synapsh create my-plugin --template vue

# 本地开发
synapsh dev

# 构建打包
synapsh build

# 签名
synapsh sign --private-key ~/.synapsh/developer.pem

# 提交审核
synapsh submit --message "Initial release"

# 查看日志
synapsh logs
```

### 2. 调试工具

```typescript
// 开发者模式 API (仅在开发时可用)
declare namespace synapsh.dev {
  // 模拟 API 响应
  function mock(apiName: string, response: any): void;

  // 查看权限状态
  function getPermissionState(): PermissionState;

  // 性能分析
  function profile(): PerformanceProfile;

  // 热重载
  function enableHotReload(): void;
}
```

---

## 总结

本设计文档涵盖了 SynapSH 应用商店的完整技术方案，包括：

1. **开发规范**：清晰的插件结构、API 标准和权限声明
2. **安全保障**：多层沙箱、权限控制、代码签名
3. **审核流程**：自动化检测 + 人工审核的双重保障
4. **版本管理**：语义化版本、热更新、兼容性检查
5. **用户反馈**：评分系统、防刷机制、崩溃报告
6. **商业模型**：透明的分成比例和结算系统

通过这套系统，SynapSH 可以在保证安全性和稳定性的前提下，建立一个繁荣的插件生态系统。
