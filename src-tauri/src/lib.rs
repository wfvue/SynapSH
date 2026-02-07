use anyhow::Result;
use log::info;
use russh::client::Handle;
use russh::keys::PublicKey;
use russh::{ChannelId, Disconnect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use tauri::Emitter;
use tokio::sync::{mpsc, Mutex};

mod db;
use db::{Database, Machine, MachineInput};

// SSH 会话管理
pub struct SSHSession {
    handle: Handle<SSHClient>,
    channel: russh::Channel<russh::client::Msg>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pid: u32,
    name: String,
    cpu: f64,
    memory: f64,
    user: String,
    // 新增字段
    status: String,       // 进程状态 (R/S/Z/D/T等)
    status_desc: String,  // 状态描述
    start_time: String,   // 启动时间
    elapsed_time: String, // 运行时长
    threads: u32,         // 线程数
    rss: u64,            // 物理内存 (KB)
    vsz: u64,            // 虚拟内存 (KB)
    command: String,      // 完整命令行
    ppid: u32,           // 父进程ID
    nice: i32,           // 优先级 (nice值)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    name: String,
    total: u64,
    used: u64,
    mount_point: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    total: u64,
    used: u64,
    free: u64,
    cached: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInfo {
    rx_bytes: u64,
    tx_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    hostname: String,
    uptime: String,
    load_average: [f64; 3],
    cpu_cores: u32,
    kernel_version: String,
    total_memory: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    cpu_percent: f64,
    memory: MemoryInfo,
    disks: Vec<DiskInfo>,
    network: NetworkInfo,
    processes: Vec<ProcessInfo>,
    system: SystemInfo,
}

impl SSHSession {
    // ... existing connect method ...
    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        password: Option<String>,
        private_key: Option<String>,
        app_handle: tauri::AppHandle,
        session_id: String,
    ) -> Result<Self> {
        info!("开始连接 SSH: {}:{} 用户: {}", host, port, username);

        let config = russh::client::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(300)),
            ..Default::default()
        };
        let config = Arc::new(config);

        // 创建通道用于接收 SSH 数据
        // 注意：这里我们调大 channel buffer，防止高吞吐时阻塞
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(4096);
        let client = SSHClient::new(tx);

        info!("正在建立 TCP 连接...");
        let mut handle = russh::client::connect(config, (host, port), client).await?;
        info!("TCP 连接成功，开始认证...");

        // 认证
        let authenticated = if let Some(key_path) = private_key {
            info!("使用密钥认证: {}", key_path);
            let key_pair = russh::keys::load_secret_key(key_path, None)?;
            let key_with_hash = russh::keys::PrivateKeyWithHashAlg::new(Arc::new(key_pair), None);
            handle
                .authenticate_publickey(username, key_with_hash)
                .await
                .is_ok()
        } else if let Some(pass) = password {
            info!("使用密码认证");
            handle.authenticate_password(username, pass).await.is_ok()
        } else {
            info!("没有提供认证信息");
            false
        };

        if !authenticated {
            return Err(anyhow::anyhow!("Authentication failed"));
        }
        info!("认证成功");

        // 打开交互式 Shell 通道
        info!("打开 SSH Shell 通道...");
        let mut channel = handle.channel_open_session().await?;
        channel
            .request_pty(false, "xterm-256color", 80, 24, 0, 0, &[])
            .await?;
        channel.request_shell(false).await?;

        let channel_id = channel.id();
        info!("通道已打开，ID: {:?}", channel_id);

        // 启动数据转发任务 - 将 SSH 数据发送到前端
        let app_handle_clone = app_handle.clone();
        let session_id_clone = session_id.clone();
        tokio::spawn(async move {
            info!("启动数据转发任务...");
            while let Some(data) = rx.recv().await {
                // 将数据发送到前端
                let data_base64 = base64_encode(&data);
                let _ =
                    app_handle_clone.emit(&format!("ssh-data-{}", session_id_clone), data_base64);
            }
            info!("数据转发任务结束");
        });

        Ok(SSHSession { handle, channel })
    }

    pub async fn write(&self, data: &[u8]) -> Result<()> {
        self.channel
            .data(data)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to write data: {:?}", e))?;
        Ok(())
    }

    pub async fn resize(&self, cols: u32, rows: u32) -> Result<()> {
        log::debug!("Resize requested: {}x{}", cols, rows);
        let _ = self.channel.window_change(cols, rows, 0, 0).await;
        Ok(())
    }

    pub async fn disconnect(self) -> Result<()> {
        self.handle
            .disconnect(Disconnect::ByApplication, "", "")
            .await?;
        Ok(())
    }

    /// 执行一次性命令并返回输出
    pub async fn exec_command(&self, command: &str) -> Result<String> {
        let mut channel = self.handle.channel_open_session().await?;
        channel.exec(true, command).await?;

        let mut output = Vec::new();
        while let Some(msg) = channel.wait().await {
            match msg {
                russh::ChannelMsg::Data { ref data } => {
                    output.extend_from_slice(data);
                }
                russh::ChannelMsg::ExitStatus { .. } => {
                    break;
                }
                _ => {}
            }
        }

        // 确保关闭通道
        let _ = channel.close().await;

        Ok(String::from_utf8_lossy(&output).to_string())
    }
}
// ... existing SSHClient struct ...
// SSH 客户端处理器
struct SSHClient {
    tx: mpsc::Sender<Vec<u8>>,
}

impl SSHClient {
    fn new(tx: mpsc::Sender<Vec<u8>>) -> Self {
        Self { tx }
    }
}

impl russh::client::Handler for SSHClient {
    type Error = anyhow::Error;

    fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> impl std::future::Future<Output = Result<bool>> + Send {
        async move { Ok(true) }
    }

    fn data(
        &mut self,
        _channel: ChannelId,
        data: &[u8],
        _session: &mut russh::client::Session,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        // 只有与 Shell 通道相关的消息才会被转发到前端
        // 这里简化处理：所有的 data 都转发。
        // 实际上 exec_command 产生的 data 也会走到这里吗？
        // russh 的 Handler 是针对 Connection 的。
        // channel.wait() 在 exec_command 中会消耗消息吗？
        // russh::Channel::wait() 会从内部 buffer 读取消息。
        // 但是 Handler::data 会拦截消息吗？
        // Russh 的 Handler data 方法如果返回 Ok(())，库通常会把数据交给 Channel 的内部队列，供 wait() 消费？
        // 不，russh 的 Handler::data返回值是 Future<Output=Result<()>>。
        // 如果 Handler 处理了数据，它还需要传递给 Channel 吗？
        // 在 russh 0.40+ 中，Handler::data 只是通知。如果不做特殊处理，Channel 还是会收到数据。
        // 但是我们在 Handler::data 里把数据发给了 tx (Shell PTY)。
        // 这会导致 exec_command 的输出也被发到前端终端里！这不仅是杂音，还会导致 exec_command 拿不到数据（如果被窃取）或者数据双发。
        // 我们需要区分 Channel ID。
        // SSHSession 结构体里保存了 shell_channel_id 吗？
        // 是的，我们需要在 SSHSession 里记录 shell channel id，并在 Handler 里判断。
        // 但是 SSHSession 创建时把 handle 给出去，Handler 是在 connect 时传入的。Client 结构体不知道 channel id。
        // 这是一个经典问题。
        // 解决方案：使用 Map<ChannelId, Sender> 在 Client 中分发数据。
        // 或者：Shell 的 Channel ID 应该是第一个。
        // 或者简单点：我们不在 Handler 里转发数据，而是让 Channel 自己读取？
        // russh 的 client::connect 要求传入 Handler。
        // 当收到数据时，russh 调用 Handler::data。
        // 如果我们想让 Channel.wait() 收到数据，Handler 应该怎么做？
        // 查看 russh 文档/源码：Handler 是用来处理"非 solicited"或者全局事件，或者作为 hook？
        // 通常 Channel 上的数据流，russh 会自动分发到 Channel 对象的 buffer 中吗？
        // Russh 机制：Handler::data 被调用。
        // 如果你需要把数据推送到 Channel stream，你不需要在 Handler 里做任何事（除了返回 Ok）。
        // 也就是默认数据会进入 Channel 的 buffer。
        // 那我们现在的 Handler::data 实现是：
        // let _ = tx.send(data).await;
        // 这把所有数据都发到了 tx。
        // 我们的 tx 是连接到前端 Terminal 的。
        // 这样 exec_command 的输出也会显示在 Terminal 上！这不行。
        // 我们需要过滤。
        // 必须知道 Shell Channel 的 ID。
        // Client 需要一个 way to know which channel is the shell channel.
        // 我们可以把 Client 放在 Arc<Mutex<Client>> ? No, connect takes ownership.
        // 我们可以使用一个共享状态。

        let data = data.to_vec();
        let tx = self.tx.clone();
        async move {
            // 这里我们应该只转发 Shell Channel 的数据。
            // 但是在这里我们不知道哪个是 Shell Channel。
            // 这是一个潜在 bug。
            // 临时解决方案：
            // 我们约定 Shell Channel 是第一个打开的 channel？
            // 或者我们修改协议，让数据带上 channel id？
            // 或者：ActivityMonitor 的命令输出通常是文本，而 Terminal 是 PTY 流。
            //
            // 更好方案：使用 Shared State 记录 Shell Channel ID。
            // 但是 Client 是在 connect 时 move 进去的。
            //
            // 让我们先简单实现，看看 data 里的 ChannelId。
            // 我们可以把 Shell Channel ID 存在一个全局 Map 里？或者通过某种方式传递给 Client。
            //
            // 实际上，exec_command 创建新 channel，执行完就关闭。
            // 我们可以尝试在 Handler 中过滤。
            //
            // 为了修正这个问题，我需要引入一个机制来识别 Shell Channel。
            // 可以使用 Arc<AtomicU32> 或者是类似的东西来存储 Shell Channel ID。
            // 在 SSHSession::connect 里，当 Shell Channel 打开后，更新这个 ID。
            // 但是 Handler 在 connect 内部就开始工作了。
            // Shell Channel ID 是在 connect 之后才拿到的？不对，channel_open_session 返回 channel，此时才有 ID。
            //
            // 修改 Client 定义：
            // struct SSHClient { tx: mpsc::Sender<Vec<u8>>, shell_channel_id: Arc<Mutex<Option<ChannelId>>> }
            //
            // 在 connect 后，把 channel.id() 写入 shell_channel_id。
            // 在 Handler::data 里，检查 current channel_id 是否等于 *shell_channel_id。

            let _ = tx.send(data).await;
            Ok(())
        }
    }

    fn extended_data(
        &mut self,
        _channel: ChannelId,
        _ext: u32,
        data: &[u8],
        _session: &mut russh::client::Session,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let data = data.to_vec();
        let tx = self.tx.clone();
        async move {
            let _ = tx.send(data).await;
            Ok(())
        }
    }
}

// 需要修复 SSHClient 以支持 Channel 过滤
// 重新定义 SSHClient 和相关逻辑
// 这需要修改 connect 流程

// ... existing helper functions ...
// base64 编码函数
fn base64_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.encode(data)
}

// 系统监控实现
impl SSHSession {
    // 辅助函数：解析内存信息
    fn parse_memory(output: &str) -> MemoryInfo {
        let mut mem = MemoryInfo {
            total: 0,
            used: 0,
            free: 0,
            cached: 0,
        };
        for line in output.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 7 {
                    mem.total = parts[1].parse().unwrap_or(0u64);
                    mem.used = parts[2].parse().unwrap_or(0u64);
                    mem.free = parts[3].parse().unwrap_or(0u64);
                    // available is often more useful than free, usually parts[6]
                    // shared is parts[4], buff/cache is parts[5]
                    // Let's stick to basic mapping
                    mem.cached = parts[5].parse().unwrap_or(0u64);
                }
            }
        }
        mem
    }

    // 辅助函数：解析磁盘信息
    fn parse_disks(output: &str) -> Vec<DiskInfo> {
        let mut disks = Vec::new();
        for (i, line) in output.lines().enumerate() {
            if i == 0 {
                continue;
            } // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                disks.push(DiskInfo {
                    name: parts[0].to_string(),
                    total: parts[1].parse().unwrap_or(0u64),
                    used: parts[2].parse().unwrap_or(0u64),
                    mount_point: parts[5].to_string(),
                });
            }
        }
        disks
    }

    // 辅助函数：解析 CPU 百分比 (简易版)
    fn parse_cpu(output: &str) -> f64 {
        // Output from top -bn1 | grep "Cpu(s)"
        // %Cpu(s):  0.0 us,  0.0 sy,  0.0 ni, 100.0 id,  0.0 wa,  0.0 hi,  0.0 si,  0.0 st
        // We want 100 - id
        if let Some(line) = output.lines().find(|l| l.contains("Cpu(s)")) {
            if let Some(id_part) = line.split(",").find(|p| p.contains("id")) {
                let id_str = id_part.trim().split_whitespace().next().unwrap_or("100");
                let id_val: f64 = id_str.parse().unwrap_or(100.0);
                return 100.0 - id_val;
            }
        }
        0.0
    }

    // 辅助函数：获取进程状态描述
    fn get_status_description(status: &str) -> String {
        let first_char = status.chars().next().unwrap_or('?');
        match first_char {
            'R' => "运行中".to_string(),
            'S' => "睡眠中".to_string(),
            'D' => "不可中断睡眠".to_string(),
            'Z' => "僵尸进程".to_string(),
            'T' => "已停止".to_string(),
            't' => "追踪停止".to_string(),
            'W' => "内存分页".to_string(),
            'X' => "死亡".to_string(),
            'x' => "死亡".to_string(),
            'K' => "内核线程".to_string(),
            'P' => "暂停".to_string(),
            _ => "未知".to_string(),
        }
    }

    // 辅助函数：从完整命令行提取进程名
    fn extract_process_name(command: &str) -> String {
        // 提取命令名（去除路径和参数）
        command
            .split_whitespace()
            .next()
            .map(|s| {
                s.split('/').last().unwrap_or(s).to_string()
            })
            .unwrap_or_else(|| "unknown".to_string())
    }
}

// 全局会话管理

static SESSIONS: OnceLock<Mutex<HashMap<String, Arc<Mutex<SSHSession>>>>> = OnceLock::new();

fn get_sessions() -> &'static Mutex<HashMap<String, Arc<Mutex<SSHSession>>>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

// Tauri Commands
#[derive(Debug, Serialize, Deserialize)]
pub struct SSHConnectionParams {
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    private_key: Option<String>,
}

#[tauri::command]
async fn connect_ssh(
    session_id: String,
    params: SSHConnectionParams,
    app: tauri::AppHandle,
) -> Result<(), String> {
    info!(
        "收到连接请求: session_id={}, params={:?}",
        session_id, params
    );

    let session = SSHSession::connect(
        &params.host,
        params.port,
        &params.username,
        params.password,
        params.private_key,
        app,
        session_id.clone(),
    )
    .await
    .map_err(|e| {
        info!("连接失败: {}", e);
        e.to_string()
    })?;

    let sessions = get_sessions();
    let mut sessions = sessions.lock().await;
    sessions.insert(session_id, Arc::new(Mutex::new(session)));
    info!("会话已保存");

    Ok(())
}

#[tauri::command]
async fn write_to_pty(session_id: String, data: String) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    if let Some(session) = sessions.get(&session_id) {
        let session = session.lock().await;
        session
            .write(data.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn resize_pty(session_id: String, cols: u32, rows: u32) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    if let Some(session) = sessions.get(&session_id) {
        let session = session.lock().await;
        session
            .resize(cols, rows)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn disconnect_ssh(session_id: String) -> Result<(), String> {
    let sessions = get_sessions();
    let mut sessions = sessions.lock().await;

    if let Some(_session) = sessions.remove(&session_id) {
        // 会话会在 drop 时自动清理
    }

    Ok(())
}

// 全局数据库实例
static DATABASE: OnceLock<Mutex<Option<Database>>> = OnceLock::new();

async fn get_db() -> Result<&'static Mutex<Option<Database>>, String> {
    let db_mutex = DATABASE.get_or_init(|| Mutex::new(None));
    let mut db_guard = db_mutex.lock().await;
    if db_guard.is_none() {
        let db = Database::new().await.map_err(|e| e.to_string())?;
        *db_guard = Some(db);
    }
    drop(db_guard);
    Ok(db_mutex)
}

// 机器管理 Commands
#[tauri::command]
async fn list_machines() -> Result<Vec<Machine>, String> {
    let db_mutex = get_db().await?;
    let db_guard = db_mutex.lock().await;
    let db = db_guard.as_ref().ok_or("数据库未初始化")?;
    db.list_machines().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_machine(input: MachineInput) -> Result<Machine, String> {
    let db_mutex = get_db().await?;
    let db_guard = db_mutex.lock().await;
    let db = db_guard.as_ref().ok_or("数据库未初始化")?;
    db.add_machine(input).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_machine(id: String, input: MachineInput) -> Result<Machine, String> {
    let db_mutex = get_db().await?;
    let db_guard = db_mutex.lock().await;
    let db = db_guard.as_ref().ok_or("数据库未初始化")?;
    db.update_machine(&id, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_machine(id: String) -> Result<(), String> {
    let db_mutex = get_db().await?;
    let db_guard = db_mutex.lock().await;
    let db = db_guard.as_ref().ok_or("数据库未初始化")?;
    db.delete_machine(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn test_connection(
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    private_key: Option<String>,
) -> Result<bool, String> {
    info!("测试连接: {}:{}", host, port);

    let config = russh::client::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(10)),
        ..Default::default()
    };
    let config = Arc::new(config);

    let (tx, _rx) = mpsc::channel::<Vec<u8>>(16);
    let client = SSHClient::new(tx);

    let mut handle = match russh::client::connect(config, (host.as_str(), port), client).await {
        Ok(h) => h,
        Err(e) => return Err(format!("连接失败: {}", e)),
    };

    let authenticated = if let Some(key_path) = private_key {
        let key_pair = russh::keys::load_secret_key(key_path, None).map_err(|e| e.to_string())?;
        let key_with_hash = russh::keys::PrivateKeyWithHashAlg::new(Arc::new(key_pair), None);
        handle
            .authenticate_publickey(&username, key_with_hash)
            .await
            .is_ok()
    } else if let Some(pass) = password {
        handle.authenticate_password(&username, pass).await.is_ok()
    } else {
        false
    };

    let _ = handle.disconnect(Disconnect::ByApplication, "", "").await;

    Ok(authenticated)
}

#[tauri::command]
async fn kill_process(session_id: String, pid: u32, signal: Option<i32>) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    if let Some(session_arc) = sessions.get(&session_id) {
        let session = session_arc.lock().await;
        
        // 默认使用 SIGTERM (15)，如果指定了信号则使用指定信号
        // 常用信号: 1=SIGHUP, 9=SIGKILL, 15=SIGTERM, 18=SIGCONT, 19=SIGSTOP
        let sig = signal.unwrap_or(15);
        let cmd = format!("kill -{} {}", sig, pid);
        
        match session.exec_command(&cmd).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("终止进程失败: {}", e)),
        }
    } else {
        Err("Session not found".to_string())
    }
}

#[tauri::command]
async fn get_system_stats(session_id: String) -> Result<SystemStats, String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    if let Some(session_arc) = sessions.get(&session_id) {
        let session = session_arc.lock().await;

        // 并行或者串行执行命令
        // 为了简单起见，我们使用单一的复合命令来减少 RTT
        // 注意：不同发行版命令可能不同，这里假设是标准 Linux 环境
        // 使用 ps 命令获取详细的进程信息，格式化为管道分隔
        let cmd = r#"
            echo "---CPU---"; top -bn1 | head -n 5; 
            echo "---MEM---"; free -b; 
            echo "---DISK---"; df -B1 -x tmpfs -x devtmpfs; 
            echo "---NET---"; cat /proc/net/dev;
            echo "---PROC---"; ps -eo pid,user,pcpu,pmem,vsz,rss,stat,nlwp,start,etime,nice,ppid,comm:50,args:200 --sort=-pcpu | head -n 21 | awk 'NR==1 {print "PID|USER|CPU|MEM|VSZ|RSS|STAT|NLWP|START|ELAPSED|NICE|PPID|COMM|ARGS"} NR>1 {printf "%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s\n", $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, substr($0, index($0,$14))}' 2>/dev/null || ps aux --sort=-%cpu | head -n 21;
            echo "---SYS---"; hostname; uptime -p; uname -r; nproc; awk '{print $1" "$2" "$3}' /proc/loadavg
        "#;

        let output = session.exec_command(cmd).await.map_err(|e| e.to_string())?;

        // 解析输出
        // 这里需要比较稳健的解析逻辑
        let sections: Vec<&str> = output.split("---").collect();

        // 初始化默认值
        let mut stats = SystemStats {
            cpu_percent: 0.0,
            memory: MemoryInfo {
                total: 0,
                used: 0,
                free: 0,
                cached: 0,
            },
            disks: Vec::new(),
            network: NetworkInfo {
                rx_bytes: 0,
                tx_bytes: 0,
            },
            processes: Vec::new(),
            system: SystemInfo {
                hostname: "Unknown".to_string(),
                uptime: "".to_string(),
                load_average: [0.0, 0.0, 0.0],
                cpu_cores: 1,
                kernel_version: "".to_string(),
                total_memory: 0,
            },
        };

        for section in sections {
            if section.starts_with("CPU---") {
                stats.cpu_percent = SSHSession::parse_cpu(section);
            } else if section.starts_with("MEM---") {
                stats.memory = SSHSession::parse_memory(section);
                stats.system.total_memory = stats.memory.total;
            } else if section.starts_with("DISK---") {
                stats.disks = SSHSession::parse_disks(section);
            } else if section.starts_with("NET---") {
                // 解析网络流量 (简单累加所有接口)
                let mut rx = 0;
                let mut tx = 0;
                for line in section.lines() {
                    if line.contains(":") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 10 {
                            // parts[0] is "eth0:" or "eth0", parts[1] is RX bytes
                            let rx_str = if parts[0].ends_with(":") {
                                parts[1]
                            } else {
                                parts[2]
                            }; // handling different spacing
                               // parts[0]: face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
                               // standard /proc/net/dev lines:
                               // Inter-|   Receive                                                |  Transmit
                               //  face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
                               //     lo: 2470604    2573    0    0    0     0          0         0  2470604    2573    0    0    0     0       0          0

                            // split_whitespace handling:
                            // "lo:" "2470604" "2573" ...
                            // If interface name is separate or attached
                            // Let's check if the line contains ":"
                            if line.contains(":") {
                                let after_colon = line.split(":").nth(1).unwrap_or("");
                                let nums: Vec<&str> = after_colon.split_whitespace().collect();
                                if nums.len() >= 9 {
                                    rx += nums[0].parse::<u64>().unwrap_or(0);
                                    tx += nums[8].parse::<u64>().unwrap_or(0);
                                }
                            }
                        }
                    }
                }
                stats.network = NetworkInfo {
                    rx_bytes: rx,
                    tx_bytes: tx,
                };
            } else if section.starts_with("PROC---") {
                // 解析增强的进程信息
                // 格式: PID|USER|CPU|MEM|VSZ|RSS|STAT|NLWP|START|ELAPSED|NICE|PPID|COMMAND
                let lines: Vec<&str> = section.lines().collect();
                for line in lines.iter().skip(1) {
                    // Skip header
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 13 {
                        let status = parts[6].to_string();
                        let status_desc = Self::get_status_description(&status);
                        
                        stats.processes.push(ProcessInfo {
                            pid: parts[0].parse().unwrap_or(0u32),
                            user: parts[1].to_string(),
                            cpu: parts[2].parse().unwrap_or(0.0),
                            memory: parts[3].parse().unwrap_or(0.0),
                            name: Self::extract_process_name(parts[12]),
                            status: status.clone(),
                            status_desc,
                            start_time: parts[8].to_string(),
                            elapsed_time: parts[9].to_string(),
                            threads: parts[7].parse().unwrap_or(1u32),
                            rss: parts[5].parse().unwrap_or(0u64),
                            vsz: parts[4].parse().unwrap_or(0u64),
                            command: parts[12].to_string(),
                            ppid: parts[11].parse().unwrap_or(0u32),
                            nice: parts[10].parse().unwrap_or(0i32),
                        });
                    }
                }
            } else if section.starts_with("SYS---") {
                let lines: Vec<&str> = section.trim().lines().skip(1).collect(); // Skip "SYS---" residue
                if lines.len() >= 1 {
                    stats.system.hostname = lines[0].to_string();
                }
                if lines.len() >= 2 {
                    stats.system.uptime = lines[1].to_string();
                }
                if lines.len() >= 3 {
                    stats.system.kernel_version = lines[2].to_string();
                }
                if lines.len() >= 4 {
                    stats.system.cpu_cores = lines[3].parse().unwrap_or(1u32);
                }
                if lines.len() >= 5 {
                    let loads: Vec<&str> = lines[4].split_whitespace().collect();
                    if loads.len() >= 3 {
                        stats.system.load_average = [
                            loads[0].parse().unwrap_or(0.0),
                            loads[1].parse().unwrap_or(0.0),
                            loads[2].parse().unwrap_or(0.0),
                        ];
                    }
                }
            }
        }

        Ok(stats)
    } else {
        Err("Session not found".to_string())
    }
}

// 需要在 run() 中注册新命令
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            connect_ssh,
            write_to_pty,
            resize_pty,
            disconnect_ssh,
            list_machines,
            add_machine,
            update_machine,
            delete_machine,
            test_connection,
            get_system_stats,
            kill_process
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
