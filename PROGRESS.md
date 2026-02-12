# SynapSH (光析) - 开发进度

## 当前状态
- [x] 阶段一：连接与终端 (Connectivity)
- [/] 阶段二：桌面式 OS Shell + 文件管理 (Visual OS)
- [ ] 阶段三：监控与 AI (Intelligence)

## 阶段一：连接与终端 ✅
- [x] Rust SSH 会话管理 + `connect_ssh` / `write_to_pty` / `resize_pty`
- [x] 终端渲染（xterm.js + WebGL + fit）
- [x] 连接面板 UI（账号/密码/密钥）
- [x] 事件通道 `ssh-data-<sessionId>` 到前端
- [x] 机器管理页面（SQLite 持久化 + CRUD + 连接测试）

## 阶段二：桌面式 OS Shell + 文件管理
- [x] 桌面 Shell 框架（桌面图标 + Dock + 状态栏 + 窗口容器）
- [x] 文件管理 UI 骨架（路径栏 + 工具栏 + 树 + 列表）
- [x] 浏览器应用（地址栏、导航、历史记录、书签）
- [x] 浏览器代理链路优化（Chrome 非阻塞启动 + SOCKS5 活性检测重建 + 错误事件回传）
- [x] 浏览器代理连通性预检与自动重建（SOCKS5 握手探测 + 失败重试）
- [x] 浏览器交互切换为外部 Chrome 直开（移除应用内浏览器页面）
- [x] SOCKS5 转发并发优化（移除 SSH Handle 串行锁，提升多资源页面加载速度）
- [x] 浏览器单实例管理（同一 session 复用 + 应用退出自动关闭）
- [x] 代理链路性能调优（SSH 参数优化 + copy_bidirectional + 并发限流容错 + 失败目标快速拒绝）
- [x] 浏览器代理切换为系统 OpenSSH -D sidecar（单隧道复用 + sidecar 生命周期托管）
- [x] OpenSSH sidecar 稳定性修复（就绪等待 + 自动重建 + stderr 诊断）
- [x] OpenSSH 浏览器启动提速（预检改为本地 SOCKS 握手，避免目标站点阻塞）
- [x] OpenSSH 多 sidecar 并行分流（会话级连接池 + 轮询端口分配）
- [x] 系统设置模块（通用/外观/终端/连接/关于）
- [x] 顶部 Tab 栏适配原生窗口控件（macOS/Windows）+ 右侧系统外观快捷设置
- [x] Tab 交互优化：垂直居中、`+` 紧随标签、固定“机器管理”主标签、连接后按机器名建会话标签
- [x] 修复 TabBar 外观弹窗配色（按语义化 Token 统一背景/边框/文字/激活态）
- [x] 升级 russh/russh-sftp 依赖版本
- [x] 重新定义 `AGENTS.md` 美学 UI 规范（视觉层级 + 语义化 Token + 动效与可访问性约束）
- [ ] SFTP `read_dir(path)` 后端实现 + JSON 结构
- [ ] 文件操作：上传 / 下载 / 新建 / 重命名 / 删除
- [ ] `get_dir_size(path)` 递归统计（用于可视化）
- [ ] Sunburst 可视化视图

## 阶段三：监控与 AI
- [x] /proc 监控拉取与解析（CPU / 内存 / 网卡 / 进程）
- [x] 监控面板（ECharts 仪表盘）
- [x] 活动监视器增强：
  - [x] 扩展 ProcessInfo 结构体，添加详细进程字段（状态、线程数、内存、优先级等）
  - [x] 优化进程数据获取命令，使用 ps 获取更详细信息
  - [x] 实现 kill_process 命令，支持多种信号（SIGTERM/SIGKILL/SIGSTOP/SIGCONT）
  - [x] 重构 ProcessList 组件，支持多列排序和搜索
  - [x] 创建 ProcessDetail 详情抽屉组件
  - [x] 集成进程终止功能到前端界面
  - [x] 优化 CPU 图表，显示核心数和统计信息
- [ ] Sidecar `llama-server` 启动
- [ ] 本地 HTTP 调用链打通
