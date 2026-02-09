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
- [x] 系统设置模块（通用/外观/终端/连接/关于）
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

