# 📂 SynapSH (光析) - 技术架构与开发规范 v1.0

## 1. 核心技术栈 (The Tech Stack)

为了实现 **极致性能** + **唯美前端** + **端侧隐私**，我们锁定以下组合：

### **前端 (UI 层)**

* **框架:** `Vue 3.6` + `TypeScript` + `Vite 8` (极速开发)
* **UI 组件库:** `shadcn UI`
* **终端内核:** `xterm.js` + `xterm-addon-webgl` (必须开启 WebGL 加速渲染) + `xterm-addon-fit`
* **可视化引擎:** `ECharts` (2D 图表，如旭日图、波形图) + `Three.js` (可选，用于首页 3D 服务器状态球)
* **状态管理:** `Pinia` (管理多 Tab 的连接状态)

* **包管理器:** `pnpm` (高效依赖管理)


### **后端 (Rust 层)**

* **应用框架:** `Tauri v2` 
* **SSH 核心:** `russh` (纯 Rust 异步 SSH 库) + `russh-keys`
* **文件传输:** `russh-sftp` (SFTP 子系统支持)
* **异步运行时:** `tokio` (处理高并发连接)
* **本地数据库:** `sqlx` (配合 SQLite) 或 `sled` (纯 Rust 嵌入式 KV 数据库，更轻量，适合存配置)

---


## 3. 核心功能实现路线图 (Implementation Path)

我们把开发分为三个阶段，**先跑通，再跑快，最后跑帅**。

### 阶段一：连接与终端 (The Connectivity)

**目标：** 能连上服务器，能敲命令，不卡顿。

1. **依赖配置 (`Cargo.toml`):**
```toml
[dependencies]
tauri = { version = "1", features = ["shell", "fs", "sql"] }
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
russh = "0.40"     # 注意检查最新版本
russh-keys = "0.40"
anyhow = "1.0"     # 错误处理
base64 = "0.21"

```


2. **后端逻辑 (`connection.rs`):**
* 创建一个 `struct SSHSession`，里面持有 `russh::client::Handle`。
* 使用 `tokio::sync::Mutex` 将 Session 包装起来，存入一个全局的 `HashMap<String, Arc<Mutex<SSHSession>>>` (用 UUID 做 Key)。
* **关键点：** 实现一个 Tauri Command `connect_ssh(ip, user, key) -> session_id`。


3. **前端逻辑 (`Terminal.vue`):**
* 初始化 `xterm.js`。
* 监听 `onData` 事件 -> 调用 Rust `write_to_pty`。
* 监听 Rust 发来的 `pty_data` 事件 -> 写入 `xterm.write()`。



### 阶段二：可视化文件管理 (The Visuals)

**目标：** 实现类似 Finder/资源管理器的 SFTP 界面。

1. **后端逻辑 (`sftp.rs`):**
* 复用阶段一建立的 SSH 连接。
* 调用 `session.channel_open_session()` 请求 `sftp` 子系统。
* 封装 `read_dir(path)`: 也就是发送 `READDIR` 包，收到二进制后，转成 JSON 结构体（包含 `name`, `size`, `is_dir`, `mod_time`）。
* **杀手级特性：** 实现 `get_dir_size(path)`。递归遍历目录，计算大小。这是画“旭日图”的基础。


2. **前端逻辑 (`FileManager.vue`):**
* 使用 Naive UI 的 Data Table 展示列表。
* 集成 `ECharts` Sunburst 图，数据源来自 `get_dir_size` 的返回结果。



### 阶段三：零侵入监控与 AI (The Intelligence)

**目标：** 仪表盘跳动，AI 本地跑通。

1. **监控实现 (`monitor/parser.rs`):**
* **Rust 端定时任务：** `tokio::spawn` 一个 Loop，每 2 秒执行一次：
```rust
// 伪代码
let output = session.exec("cat /proc/stat /proc/meminfo /proc/net/dev").await?;
let metrics = parser::parse_linux_metrics(output); // 自己写正则解析
app_handle.emit_all("update-metrics", metrics);

```


* **前端：** 接收事件，更新 Vue 的响应式变量，驱动 ECharts 重新渲染。


2. **端侧 AI 集成 (Sidecar 模式):**
* **策略：** 不要尝试在 Rust 里编译 C++ 的 `llama.cpp`，这在 Windows 上会让你痛不欲生。
* **做法：**
1. 下载编译好的 `llama-server` (llama.cpp 的官方 http server 示例) 可执行文件。
2. 在 `tauri.conf.json` 配置 `shell > sidecar`。
3. Rust 启动时，自动拉起这个 Sidecar 进程（监听 localhost:8080）。
4. 前端或者 Rust 通过标准 HTTP 请求 `POST http://127.0.0.1:8080/completion` 来获取 AI 回复。
5. **隐私保证：** 这个 HTTP 请求完全在本地回环网络发生，不经过网卡。

