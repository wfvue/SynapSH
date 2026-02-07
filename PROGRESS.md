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
- [ ] SFTP `read_dir(path)` 后端实现 + JSON 结构
- [ ] 文件操作：上传 / 下载 / 新建 / 重命名 / 删除
- [ ] `get_dir_size(path)` 递归统计（用于可视化）
- [ ] Sunburst 可视化视图

## 阶段三：监控与 AI
- [ ] /proc 监控拉取与解析（CPU / 内存 / 网卡）
- [ ] 监控面板（ECharts 仪表盘）
- [ ] Sidecar `llama-server` 启动
- [ ] 本地 HTTP 调用链打通

