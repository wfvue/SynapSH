use anyhow::Result;
use log::info;
use russh::client::Handle;
use russh::keys::PublicKey;
use russh::{ChannelId, Disconnect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;
use std::sync::OnceLock;
use tauri::Emitter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch, Mutex};

mod db;
use db::{Database, Machine, MachineInput};

mod database_manager;
use database_manager::{DatabaseDetectionResult, DatabaseType, InstallOptions};

// 文件类型定义
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FileEntryType {
    Directory,
    File,
    Symlink,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    name: String,
    path: String,
    #[serde(rename = "type")]
    entry_type: FileEntryType,
    size: u64,
    modified_time: Option<String>,
    created_time: Option<String>,
    permissions: String,
    owner: String,
    group: String,
    is_hidden: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileListResult {
    path: String,
    entries: Vec<FileEntry>,
    parent_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTransferProgress {
    file_name: String,
    bytes_transferred: u64,
    total_bytes: u64,
    percentage: f64,
}

// SSH 会话管理
pub struct SSHSession {
    handle: Arc<Mutex<Handle<SSHClient>>>,
    channel: russh::Channel<russh::client::Msg>,
    #[allow(dead_code)]
    #[allow(dead_code)]
    shell_channel_id: Arc<Mutex<Option<ChannelId>>>,
    http_proxy: Option<HttpProxy>,
    sftp_session: Arc<Mutex<Option<russh_sftp::client::SftpSession>>>,
}

struct HttpProxy {
    port: u16,
    shutdown: watch::Sender<bool>,
}

impl HttpProxy {
    async fn start(
        handle: Arc<Mutex<Handle<SSHClient>>>,
        app: tauri::AppHandle,
        session_id: String,
    ) -> Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let port = listener.local_addr()?.port();
        let (shutdown, mut shutdown_rx) = watch::channel(false);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                    accept = listener.accept() => {
                        match accept {
                            Ok((stream, originator)) => {
                                let handle = handle.clone();
                                let app = app.clone();
                                let session_id = session_id.clone();
                                tokio::spawn(async move {
                                    if let Err(err) = handle_http_client(
                                        stream,
                                        originator,
                                        handle,
                                        app,
                                        session_id,
                                    )
                                    .await
                                    {
                                        log::debug!("HTTP client error: {err}");
                                    }
                                });
                            }
                            Err(err) => {
                                log::warn!("HTTP accept error: {err}");
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(Self { port, shutdown })
    }
}

impl Drop for HttpProxy {
    fn drop(&mut self) {
        let _ = self.shutdown.send(true);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pid: u32,
    name: String,
    cpu: f64,
    memory: f64,
    user: String,
    // 进程状态
    status: String,       // 进程状态 (R/S/Z/D/T等)
    status_desc: String,  // 状态描述
    start_time: String,   // 启动时间
    elapsed_time: String, // 运行时长 (TIME字段)
    vsz: u64,             // 虚拟内存 (KB)
    rss: u64,             // 物理内存 (KB)
    command: String,      // 完整命令行
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrowserLaunchOptions {
    profile_mode: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BrowserProxyError {
    session_id: String,
    host: String,
    port: u16,
    message: String,
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
        let shell_channel_id = Arc::new(Mutex::new(None));
        let client = SSHClient::new(tx, shell_channel_id.clone());

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
        let channel = handle.channel_open_session().await?;
        channel
            .request_pty(false, "xterm-256color", 80, 24, 0, 0, &[])
            .await?;
        channel.request_shell(false).await?;

        let channel_id = channel.id();
        *shell_channel_id.lock().await = Some(channel_id);
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

        Ok(SSHSession {
            handle: Arc::new(Mutex::new(handle)),
            channel,
            shell_channel_id,
            http_proxy: None,
            sftp_session: Arc::new(Mutex::new(None)),
        })
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
        let handle = self.handle.lock().await;
        handle.disconnect(Disconnect::ByApplication, "", "").await?;
        Ok(())
    }

    /// 执行一次性命令并返回输出
    pub async fn exec_command(&self, command: &str) -> Result<String> {
        let mut channel = {
            let handle = self.handle.lock().await;
            let channel = handle.channel_open_session().await?;
            channel.exec(true, command).await?;
            channel
        };

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

    pub async fn check_direct_tcpip(&self, host: &str, port: u16) -> Result<()> {
        let handle = self.handle.lock().await;
        let channel = handle
            .channel_open_direct_tcpip(host.to_string(), port.into(), "127.0.0.1".to_string(), 0u32)
            .await?;
        let _ = channel.close().await;
        Ok(())
    }

    pub async fn ensure_http_proxy(
        &mut self,
        app: tauri::AppHandle,
        session_id: String,
    ) -> Result<u16> {
        if let Some(proxy) = &self.http_proxy {
            return Ok(proxy.port);
        }

        let proxy = HttpProxy::start(self.handle.clone(), app, session_id).await?;
        let port = proxy.port;
        self.http_proxy = Some(proxy);
        Ok(port)
    }

    /// 获取或创建 SFTP 会话
    /// 返回 MutexGuard 以确保互斥访问（SFTP 通道不支持并发写入）
    pub async fn get_sftp(
        &self,
    ) -> Result<tokio::sync::MutexGuard<'_, Option<russh_sftp::client::SftpSession>>> {
        let start = std::time::Instant::now();
        log::info!("[SFTP] get_sftp: acquiring lock...");
        let mut guard = self.sftp_session.lock().await;
        log::info!("[SFTP] get_sftp: lock acquired in {:?}", start.elapsed());

        if guard.is_none() {
            log::info!("[SFTP] Initializing new SFTP session...");
            let t1 = std::time::Instant::now();
            let handle = self.handle.lock().await;
            log::info!("[SFTP] handle lock acquired in {:?}", t1.elapsed());

            let t2 = std::time::Instant::now();
            let channel = handle.channel_open_session().await?;
            log::info!("[SFTP] channel_open_session in {:?}", t2.elapsed());

            let t3 = std::time::Instant::now();
            channel.request_subsystem(false, "sftp").await?;
            log::info!("[SFTP] request_subsystem in {:?}", t3.elapsed());

            let t4 = std::time::Instant::now();
            let sftp = russh_sftp::client::SftpSession::new(channel.into_stream()).await?;
            log::info!("[SFTP] SftpSession::new in {:?}", t4.elapsed());

            *guard = Some(sftp);
            log::info!("[SFTP] Total SFTP init: {:?}", start.elapsed());
        }

        Ok(guard)
    }

    /// 列出目录内容
    pub async fn list_directory(&self, path: &str) -> Result<FileListResult> {
        let total_start = std::time::Instant::now();
        log::info!("[SFTP] list_directory: {}", path);

        let mut guard = self.get_sftp().await?;
        log::info!("[SFTP] get_sftp done in {:?}", total_start.elapsed());
        let sftp = guard.as_ref().unwrap();

        // 读取目录
        let t_readdir = std::time::Instant::now();
        let entries = match sftp.read_dir(path).await {
            Ok(e) => e,
            Err(e) => {
                log::warn!("SFTP read_dir failed, invalidating session: {}", e);
                *guard = None;
                return Err(e.into());
            }
        };
        log::info!("[SFTP] read_dir completed in {:?}", t_readdir.elapsed());

        let mut file_entries: Vec<FileEntry> = Vec::new();
        let t_metadata = std::time::Instant::now();
        let mut metadata_count = 0;

        for entry in entries {
            let name = entry.file_name();
            let full_path = format!("{}/{}", path.trim_end_matches('/'), name);

            // 直接使用 DirEntry 自带的 metadata，无需额外网络请求
            let attrs = entry.metadata();
            let file_type = entry.file_type();
            metadata_count += 1;

            let entry_type = match file_type {
                russh_sftp::protocol::FileType::Dir => FileEntryType::Directory,
                russh_sftp::protocol::FileType::File => FileEntryType::File,
                russh_sftp::protocol::FileType::Symlink => FileEntryType::Symlink,
                _ => FileEntryType::Unknown,
            };

            let permissions = format!("{:o}", attrs.permissions.unwrap_or(0) & 0o777);

            file_entries.push(FileEntry {
                name: name.clone(),
                path: full_path,
                entry_type,
                size: attrs.size.unwrap_or(0),
                modified_time: attrs.mtime.map(|t| {
                    chrono::DateTime::from_timestamp(t as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                }),
                created_time: None,
                permissions,
                owner: attrs.uid.map(|u| u.to_string()).unwrap_or_default(),
                group: attrs.gid.map(|g| g.to_string()).unwrap_or_default(),
                is_hidden: name.starts_with('.'),
            });
        }
        log::info!(
            "[SFTP] metadata for {} files in {:?}",
            metadata_count,
            t_metadata.elapsed()
        );

        // 排序
        file_entries.sort_by(|a, b| match (&a.entry_type, &b.entry_type) {
            (FileEntryType::Directory, FileEntryType::File) => std::cmp::Ordering::Less,
            (FileEntryType::File, FileEntryType::Directory) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        let parent_path = if path == "/" || path.is_empty() {
            None
        } else {
            let path_obj = std::path::Path::new(path);
            path_obj.parent().map(|p| p.to_string_lossy().to_string())
        };

        log::info!("[SFTP] list_directory total: {:?}", total_start.elapsed());

        Ok(FileListResult {
            path: path.to_string(),
            entries: file_entries,
            parent_path,
        })
    }

    /// 创建目录
    pub async fn create_directory(&self, path: &str) -> Result<()> {
        let mut guard = self.get_sftp().await?;
        let sftp = guard.as_ref().unwrap();

        if let Err(e) = sftp.create_dir(path).await {
            log::warn!("SFTP create_dir failed: {}", e);
            *guard = None;
            return Err(e.into());
        }
        Ok(())
    }

    /// 删除文件
    pub async fn remove_file(&self, path: &str) -> Result<()> {
        let mut guard = self.get_sftp().await?;
        let sftp = guard.as_ref().unwrap();

        if let Err(e) = sftp.remove_file(path).await {
            log::warn!("SFTP remove_file failed: {}", e);
            *guard = None;
            return Err(e.into());
        }
        Ok(())
    }

    /// 删除目录
    pub async fn remove_directory(&self, path: &str) -> Result<()> {
        let mut guard = self.get_sftp().await?;
        let sftp = guard.as_ref().unwrap();

        if let Err(e) = sftp.remove_dir(path).await {
            log::warn!("SFTP remove_dir failed: {}", e);
            *guard = None;
            return Err(e.into());
        }
        Ok(())
    }

    /// 重命名/移动文件
    pub async fn rename(&self, old_path: &str, new_path: &str) -> Result<()> {
        let mut guard = self.get_sftp().await?;
        let sftp = guard.as_ref().unwrap();

        if let Err(e) = sftp.rename(old_path, new_path).await {
            log::warn!("SFTP rename failed: {}", e);
            *guard = None;
            return Err(e.into());
        }
        Ok(())
    }

    /// 下载文件内容
    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let mut guard = self.get_sftp().await?;
        let sftp = guard.as_ref().unwrap();

        let mut file = match sftp.open(path).await {
            Ok(f) => f,
            Err(e) => {
                log::warn!("SFTP open failed: {}", e);
                *guard = None;
                return Err(e.into());
            }
        };

        let mut contents = Vec::new();
        if let Err(e) = tokio::io::AsyncReadExt::read_to_end(&mut file, &mut contents).await {
            log::warn!("SFTP read content failed: {}", e);
            *guard = None;
            return Err(e.into());
        }
        Ok(contents)
    }

    /// 上传文件
    pub async fn write_file(&self, path: &str, contents: &[u8]) -> Result<()> {
        let mut guard = self.get_sftp().await?;
        let sftp = guard.as_ref().unwrap();

        let mut file = match sftp.create(path).await {
            Ok(f) => f,
            Err(e) => {
                log::warn!("SFTP create file failed: {}", e);
                *guard = None;
                return Err(e.into());
            }
        };

        if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut file, contents).await {
            log::warn!("SFTP write content failed: {}", e);
            *guard = None;
            return Err(e.into());
        }
        Ok(())
    }

    /// 设置文件权限 (chmod)
    pub async fn set_permissions(&self, path: &str, mode: u32) -> Result<()> {
        let mut guard = self.get_sftp().await?;
        let sftp = guard.as_ref().unwrap();

        let mut attrs = russh_sftp::protocol::FileAttributes::default();
        attrs.permissions = Some(mode);

        if let Err(e) = sftp.set_metadata(path, attrs).await {
            log::warn!("SFTP set_metadata failed: {}", e);
            *guard = None;
            return Err(e.into());
        }
        Ok(())
    }
}
// ... existing SSHClient struct ...
// SSH 客户端处理器
struct SSHClient {
    tx: mpsc::Sender<Vec<u8>>,
    shell_channel_id: Arc<Mutex<Option<ChannelId>>>,
}

impl SSHClient {
    fn new(tx: mpsc::Sender<Vec<u8>>, shell_channel_id: Arc<Mutex<Option<ChannelId>>>) -> Self {
        Self {
            tx,
            shell_channel_id,
        }
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
        channel: ChannelId,
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
        let shell_channel_id = self.shell_channel_id.clone();
        async move {
            if *shell_channel_id.lock().await == Some(channel) {
                let _ = tx.send(data).await;
            }
            Ok(())
        }
    }

    fn extended_data(
        &mut self,
        channel: ChannelId,
        _ext: u32,
        data: &[u8],
        _session: &mut russh::client::Session,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let data = data.to_vec();
        let tx = self.tx.clone();
        let shell_channel_id = self.shell_channel_id.clone();
        async move {
            if *shell_channel_id.lock().await == Some(channel) {
                let _ = tx.send(data).await;
            }
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

fn normalize_url(input: &str) -> Result<tauri::Url, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("URL 不能为空".to_string());
    }
    let url = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{}", trimmed)
    };
    tauri::Url::parse(&url).map_err(|e| e.to_string())
}

fn browser_label(session_id: &str) -> String {
    let mut label = String::from("browser-");
    for ch in session_id.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '/' | ':' | '_') {
            label.push(ch);
        } else {
            label.push('_');
        }
    }
    label
}

fn chrome_profile_dir(session_id: &str, profile_mode: &str) -> Result<std::path::PathBuf, String> {
    let base = std::env::temp_dir()
        .join("synapsh-chrome")
        .join(browser_label(session_id));
    let dir = if profile_mode == "new" {
        base.join(format!("profile-{}", uuid::Uuid::new_v4()))
    } else {
        base.join("profile")
    };
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

async fn handle_http_client(
    mut stream: TcpStream,
    originator: SocketAddr,
    handle: Arc<Mutex<Handle<SSHClient>>>,
    app: tauri::AppHandle,
    session_id: String,
) -> Result<()> {
    // 读取完整的 HTTP 请求头部（直到空行）
    let mut request = Vec::new();
    let mut buf = [0u8; 1];

    // 读取请求行和头部
    loop {
        stream.read_exact(&mut buf).await?;
        request.push(buf[0]);

        // 检查是否到达头部结束标记 \r\n\r\n
        if request.len() >= 4 {
            if request.ends_with(&[b'\r', b'\n', b'\r', b'\n']) {
                break;
            }
        }
    }

    // 解析 CONNECT 请求
    let request_str = String::from_utf8_lossy(&request);
    let lines: Vec<&str> = request_str.lines().collect();

    if lines.is_empty() {
        return Err(anyhow::anyhow!("Empty request"));
    }

    let first_line = lines[0];
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    if parts.is_empty() || parts[0] != "CONNECT" {
        let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
        stream.write_all(response.as_bytes()).await?;
        return Err(anyhow::anyhow!(
            "Invalid HTTP CONNECT request: {}",
            first_line
        ));
    }

    if parts.len() < 2 {
        let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
        stream.write_all(response.as_bytes()).await?;
        return Err(anyhow::anyhow!("Missing target host:port"));
    }

    // 解析 host:port
    let target = parts[1];
    let (host, port_str) = match target.split_once(':') {
        Some((h, p)) => (h, p),
        None => (target, "443"),
    };

    let port = match port_str.parse::<u16>() {
        Ok(p) => p,
        Err(_) => 443,
    };

    log::debug!("HTTP CONNECT: {}:{} from {}", host, port, originator);

    // 通过 SSH 隧道建立连接
    let mut channel = match {
        let handle = handle.lock().await;
        handle
            .channel_open_direct_tcpip(
                host,
                port.into(),
                originator.ip().to_string(),
                originator.port().into(),
            )
            .await
    } {
        Ok(channel) => channel,
        Err(err) => {
            log::error!("Failed to open SSH channel to {}: {}: {}", host, port, err);
            let response = "HTTP/1.1 502 Bad Gateway\r\n\r\n";
            stream.write_all(response.as_bytes()).await?;
            let event = BrowserProxyError {
                session_id,
                host: host.to_string(),
                port,
                message: format!("连接远端目标失败：{err}"),
            };
            let _ = app.emit("browser-proxy-error", event);
            return Err(anyhow::anyhow!("Open channel failed: {err}"));
        }
    };

    // 发送成功响应
    let response = "HTTP/1.1 200 Connection Established\r\nProxy-Agent: SynapSH/1.0\r\n\r\n";
    if let Err(err) = stream.write_all(response.as_bytes()).await {
        log::error!("Failed to send HTTP 200 response: {}", err);
        return Err(err.into());
    }

    // 开始双向数据转发
    if let Err(err) = proxy_data(stream, &mut channel).await {
        log::debug!("Proxy data transfer ended: {}", err);
        let event = BrowserProxyError {
            session_id,
            host: host.to_string(),
            port,
            message: format!("传输中断：{err}"),
        };
        let _ = app.emit("browser-proxy-error", event);
        return Err(err);
    }
    Ok(())
}

async fn proxy_data(
    mut stream: TcpStream,
    channel: &mut russh::Channel<russh::client::Msg>,
) -> Result<()> {
    let mut client_closed = false;
    let mut buf = vec![0u8; 16 * 1024];

    loop {
        tokio::select! {
            read_res = stream.read(&mut buf), if !client_closed => {
                match read_res {
                    Ok(0) => {
                        client_closed = true;
                        let _ = channel.eof().await;
                    }
                    Ok(n) => {
                        channel.data(&buf[..n]).await?;
                    }
                    Err(err) => return Err(err.into()),
                }
            }
            msg = channel.wait() => {
                match msg {
                    Some(russh::ChannelMsg::Data { data }) => {
                        stream.write_all(&data).await?;
                    }
                    Some(russh::ChannelMsg::Eof) | Some(russh::ChannelMsg::Close) | None => {
                        break;
                    }
                    Some(russh::ChannelMsg::ExitStatus { .. }) => {}
                    Some(russh::ChannelMsg::WindowAdjusted { .. }) => {}
                    Some(_) => {}
                }
            }
        }
    }

    let _ = channel.close().await;
    let _ = stream.shutdown().await;
    Ok(())
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
        .map(|s| s.split('/').last().unwrap_or(s).to_string())
        .unwrap_or_else(|| "unknown".to_string())
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

    info!("connect_ssh 命令返回 Ok(())");
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

#[tauri::command]
async fn browser_open(
    session_id: String,
    url: String,
    options: Option<BrowserLaunchOptions>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let target_url = normalize_url(&url)?;
    let host = target_url
        .host_str()
        .ok_or_else(|| "无法解析 URL 主机".to_string())?;
    let port = target_url
        .port_or_known_default()
        .ok_or_else(|| "无法解析 URL 端口".to_string())?;
    let profile_mode = options
        .as_ref()
        .and_then(|opt| opt.profile_mode.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("session");

    let session_arc = {
        let sessions = get_sessions();
        let sessions = sessions.lock().await;
        sessions
            .get(&session_id)
            .cloned()
            .ok_or_else(|| "Session not found".to_string())?
    };

    let mut session = session_arc.lock().await;
    if let Err(err) = session.check_direct_tcpip(host, port).await {
        return Err(format!(
            "SSH 端口转发失败：{}。请确认远端允许 AllowTcpForwarding 并且能访问目标站点。",
            err
        ));
    }

    let proxy_port = session
        .ensure_http_proxy(app, session_id.clone())
        .await
        .map_err(|e| format!("启动 HTTP 代理失败：{e}"))?;
    drop(session);

    let proxy_arg = format!("--proxy-server=http://127.0.0.1:{proxy_port}");
    let profile_dir = chrome_profile_dir(&session_id, profile_mode)?;
    let profile_arg = format!("--user-data-dir={}", profile_dir.display());

    log::info!("启动 Chrome，代理端口: {}, URL: {}", proxy_port, target_url);

    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open")
            .args([
                "-na",
                "Google Chrome",
                "--args",
                &proxy_arg,
                &profile_arg,
                "--disable-quic",
                "--disable-features=VizDisplayCompositor",
                "--disable-background-networking",
                "--disable-default-apps",
                "--disable-extensions",
                "--disable-sync",
                "--disable-translate",
                "--no-first-run",
                "--no-default-browser-check",
                "--new-window",
                target_url.as_str(),
            ])
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("打开 Chrome 失败，请确认已安装 Google Chrome".to_string());
        }
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (proxy_arg, profile_arg, target_url);
        Err("当前仅实现 macOS 的 Chrome 启动方案".to_string())
    }
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
    let client = SSHClient::new(tx, Arc::new(Mutex::new(None)));

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
    log::info!("Getting system stats for session: {}", session_id);

    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    if let Some(session_arc) = sessions.get(&session_id) {
        let session = session_arc.lock().await;

        // 并行或者串行执行命令
        // 为了简单起见，我们使用单一的复合命令来减少 RTT
        // 注意：不同发行版命令可能不同，这里假设是标准 Linux 环境
        // 使用 ps 命令获取详细的进程信息，格式化为管道分隔
        // 使用独特的分隔符避免与命令输出冲突
        let cmd = r#"
            echo "@@@SECTION:CPU@@@"; top -bn1 | head -n 5; 
            echo "@@@SECTION:MEM@@@"; free -b; 
            echo "@@@SECTION:DISK@@@"; df -B1 -x tmpfs -x devtmpfs; 
            echo "@@@SECTION:NET@@@"; cat /proc/net/dev;
            echo "@@@SECTION:PROC@@@"; ps aux --sort=-%cpu | head -n 21 | awk 'BEGIN {print "PID|USER|CPU|MEM|VSZ|RSS|STAT|START|TIME|COMMAND"} NR>1 {printf "%s|%s|%s|%s|%s|%s|%s|%s|%s|%s\n", $2, $1, $3, $4, $5, $6, $8, $9, $10, substr($0, index($0,$11))}' ;
            echo "@@@SECTION:SYS@@@"; hostname; uptime -p; uname -r; nproc; awk '{print $1" "$2" "$3}' /proc/loadavg
        "#;

        let output = session.exec_command(cmd).await.map_err(|e| {
            log::error!("Exec command failed: {}", e);
            e.to_string()
        })?;

        log::debug!("System stats output length: {}", output.len());
        log::debug!("Raw output preview: {}", &output[..output.len().min(500)]);

        // 解析输出 - 使用更独特的分隔符
        let sections: Vec<&str> = output.split("@@@SECTION:").collect();
        log::debug!("Found {} sections", sections.len());

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
            let section = section.trim();
            if section.is_empty() {
                continue;
            }
            log::debug!("Processing section: {}", &section[..section.len().min(30)]);

            if section.starts_with("CPU@@@") || section.starts_with("CPU") {
                stats.cpu_percent = SSHSession::parse_cpu(section);
            } else if section.starts_with("MEM@@@") || section.starts_with("MEM") {
                stats.memory = SSHSession::parse_memory(section);
                stats.system.total_memory = stats.memory.total;
            } else if section.starts_with("DISK@@@") || section.starts_with("DISK") {
                stats.disks = SSHSession::parse_disks(section);
            } else if section.starts_with("NET@@@") || section.starts_with("NET") {
                // 解析网络流量 (简单累加所有接口)
                let mut rx = 0;
                let mut tx = 0;
                for line in section.lines() {
                    if line.contains(":") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 10 {
                            // parts[0] is "eth0:" or "eth0", parts[1] is RX bytes
                            let _rx_str = if parts[0].ends_with(":") {
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
            } else if section.starts_with("PROC@@@") || section.starts_with("PROC") {
                // 解析进程信息
                // 格式: PID|USER|CPU|MEM|VSZ|RSS|STAT|START|TIME|COMMAND
                let content = section
                    .strip_prefix("PROC@@@")
                    .unwrap_or(section.strip_prefix("PROC").unwrap_or(section))
                    .trim();
                let lines: Vec<&str> = content.lines().collect();
                log::debug!("PROC section lines count: {}", lines.len());

                for (idx, line) in lines.iter().enumerate().skip(1) {
                    if line.trim().is_empty() {
                        continue;
                    }

                    let parts: Vec<&str> = line.split('|').collect();
                    log::debug!("Line {} parts: {:?}", idx, parts);

                    if parts.len() >= 10 {
                        let pid_str = parts[0].trim();
                        // 跳过表头行
                        if pid_str == "PID" || pid_str.parse::<u32>().is_err() {
                            continue;
                        }

                        let status = parts[6].trim().to_string();
                        let status_desc = get_status_description(&status);

                        stats.processes.push(ProcessInfo {
                            pid: pid_str.parse().unwrap_or(0u32),
                            user: parts[1].trim().to_string(),
                            cpu: parts[2].trim().parse().unwrap_or(0.0),
                            memory: parts[3].trim().parse().unwrap_or(0.0),
                            name: extract_process_name(parts[9]),
                            status: status.clone(),
                            status_desc,
                            start_time: parts[7].trim().to_string(),
                            elapsed_time: parts[8].trim().to_string(),
                            vsz: parts[4].trim().parse().unwrap_or(0u64),
                            rss: parts[5].trim().parse().unwrap_or(0u64),
                            command: parts[9].trim().to_string(),
                        });
                    }
                }
                log::debug!("Parsed {} processes", stats.processes.len());
            } else if section.starts_with("SYS@@@") || section.starts_with("SYS") {
                // 移除 "SYS@@@" 前缀并解析剩余内容
                let content = section
                    .strip_prefix("SYS@@@")
                    .unwrap_or(section.strip_prefix("SYS").unwrap_or(section))
                    .trim();
                let lines: Vec<&str> = content.lines().collect();
                log::debug!("SYS section lines: {:?}", lines);

                if lines.len() >= 1 && !lines[0].is_empty() {
                    stats.system.hostname = lines[0].trim().to_string();
                }
                if lines.len() >= 2 && !lines[1].is_empty() {
                    stats.system.uptime = lines[1].trim().to_string();
                }
                if lines.len() >= 3 && !lines[2].is_empty() {
                    stats.system.kernel_version = lines[2].trim().to_string();
                }
                if lines.len() >= 4 && !lines[3].is_empty() {
                    stats.system.cpu_cores = lines[3].trim().parse().unwrap_or(1u32);
                }
                if lines.len() >= 5 && !lines[4].is_empty() {
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

// 文件操作 Commands
#[tauri::command]
async fn list_files(session_id: String, path: String) -> Result<FileListResult, String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    if let Some(session_arc) = sessions.get(&session_id) {
        let session = session_arc.lock().await;
        session.list_directory(&path).await.map_err(|e| {
            log::error!("Failed to list directory: {}", e);
            e.to_string()
        })
    } else {
        Err("Session not found".to_string())
    }
}

#[tauri::command]
async fn create_folder(session_id: String, path: String) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    if let Some(session_arc) = sessions.get(&session_id) {
        let session = session_arc.lock().await;
        session
            .create_directory(&path)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Session not found".to_string())
    }
}

#[tauri::command]
async fn delete_file_or_folder(
    session_id: String,
    path: String,
    is_directory: bool,
) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    if let Some(session_arc) = sessions.get(&session_id) {
        let session = session_arc.lock().await;
        if is_directory {
            session
                .remove_directory(&path)
                .await
                .map_err(|e| e.to_string())
        } else {
            session.remove_file(&path).await.map_err(|e| e.to_string())
        }
    } else {
        Err("Session not found".to_string())
    }
}

#[tauri::command]
async fn rename_file(session_id: String, old_path: String, new_path: String) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    if let Some(session_arc) = sessions.get(&session_id) {
        let session = session_arc.lock().await;
        session
            .rename(&old_path, &new_path)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Session not found".to_string())
    }
}

#[tauri::command]
async fn download_file(session_id: String, remote_path: String) -> Result<String, String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    if let Some(session_arc) = sessions.get(&session_id) {
        let session = session_arc.lock().await;
        let contents = session
            .read_file(&remote_path)
            .await
            .map_err(|e| e.to_string())?;
        // 返回 base64 编码的文件内容
        Ok(base64_encode(&contents))
    } else {
        Err("Session not found".to_string())
    }
}

#[tauri::command]
async fn upload_file(
    session_id: String,
    remote_path: String,
    base64_content: String,
) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    if let Some(session_arc) = sessions.get(&session_id) {
        let session = session_arc.lock().await;
        let contents = base64_decode(&base64_content).map_err(|e| e)?;
        session
            .write_file(&remote_path, &contents)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Session not found".to_string())
    }
}

#[tauri::command]
async fn create_file(
    session_id: String,
    path: String,
    content: Option<String>,
) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    if let Some(session_arc) = sessions.get(&session_id) {
        let session = session_arc.lock().await;
        let data = content.unwrap_or_default();
        session
            .write_file(&path, data.as_bytes())
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Session not found".to_string())
    }
}

#[tauri::command]
async fn chmod_file(session_id: String, path: String, mode: u32) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    if let Some(session_arc) = sessions.get(&session_id) {
        let session = session_arc.lock().await;
        session
            .set_permissions(&path, mode)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Session not found".to_string())
    }
}

// base64 解码辅助函数
fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD
        .decode(data)
        .map_err(|e| e.to_string())
}

// ==================== 数据库管理 Commands ====================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetectDatabasesParams {
    session_id: String,
}

#[tauri::command]
async fn detect_databases(params: DetectDatabasesParams) -> Result<Vec<DatabaseDetectionResult>, String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    let session_arc = sessions
        .get(&params.session_id)
        .cloned()
        .ok_or("Session not found")?;

    // 使用 exec_command 直接执行检测命令
    let results = detect_databases_direct(&session_arc).await?;
    
    Ok(results)
}

async fn detect_databases_direct(
    session_arc: &Arc<Mutex<SSHSession>>,
) -> Result<Vec<DatabaseDetectionResult>, String> {
    let mut results = Vec::new();
    
    // MySQL 检测
    if let Ok(result) = detect_mysql_direct(session_arc).await {
        results.push(result);
    }
    
    // PostgreSQL 检测
    if let Ok(result) = detect_postgresql_direct(session_arc).await {
        results.push(result);
    }
    
    // Redis 检测
    if let Ok(result) = detect_redis_direct(session_arc).await {
        results.push(result);
    }
    
    // MongoDB 检测
    if let Ok(result) = detect_mongodb_direct(session_arc).await {
        results.push(result);
    }
    
    // MariaDB 检测
    if let Ok(result) = detect_mariadb_direct(session_arc).await {
        results.push(result);
    }
    
    Ok(results)
}

async fn detect_mysql_direct(session_arc: &Arc<Mutex<SSHSession>>) -> Result<DatabaseDetectionResult, String> {
    let session = session_arc.lock().await;
    
    // 检查版本
    let version_output = session.exec_command("mysql --version 2>/dev/null || echo 'NOT_FOUND'").await
        .map_err(|e| e.to_string())?;
    
    if version_output.contains("NOT_FOUND") {
        return Ok(DatabaseDetectionResult {
            db_type: DatabaseType::MySQL,
            installed: false,
            version: None,
            status: database_manager::DatabaseStatus::NotInstalled,
            port: None,
            install_path: None,
        });
    }
    
    let version = extract_version_from_output(&version_output, &["Ver", "version"]);
    
    // 检查服务状态
    let status_cmd = "sudo systemctl is-active mysql 2>/dev/null || sudo service mysql status 2>&1 | grep -q running && echo 'active' || echo 'inactive'";
    let status_output = session.exec_command(status_cmd).await.unwrap_or_default();
    let status = if status_output.trim() == "active" {
        database_manager::DatabaseStatus::Running
    } else {
        database_manager::DatabaseStatus::Stopped
    };
    
    // 获取端口
    let port_output = session.exec_command("mysql -u root -e \"SHOW VARIABLES LIKE 'port';\" 2>/dev/null | tail -1 | awk '{print $2}'").await.unwrap_or_default();
    let port = port_output.trim().parse::<u16>().ok();
    
    // 获取安装路径
    let path_output = session.exec_command("which mysql 2>/dev/null || echo ''").await.unwrap_or_default();
    let install_path = path_output.trim().to_string().into();
    
    Ok(DatabaseDetectionResult {
        db_type: DatabaseType::MySQL,
        installed: true,
        version,
        status,
        port: port.or(Some(3306)),
        install_path,
    })
}

async fn detect_postgresql_direct(session_arc: &Arc<Mutex<SSHSession>>) -> Result<DatabaseDetectionResult, String> {
    let session = session_arc.lock().await;
    
    let version_output = session.exec_command("psql --version 2>/dev/null || echo 'NOT_FOUND'").await
        .map_err(|e| e.to_string())?;
    
    if version_output.contains("NOT_FOUND") {
        return Ok(DatabaseDetectionResult {
            db_type: DatabaseType::PostgreSQL,
            installed: false,
            version: None,
            status: database_manager::DatabaseStatus::NotInstalled,
            port: None,
            install_path: None,
        });
    }
    
    let version = extract_version_from_output(&version_output, &["psql", "PostgreSQL"]);
    
    let status_cmd = "sudo systemctl is-active postgresql 2>/dev/null || sudo service postgresql status 2>&1 | grep -q running && echo 'active' || echo 'inactive'";
    let status_output = session.exec_command(status_cmd).await.unwrap_or_default();
    let status = if status_output.trim() == "active" {
        database_manager::DatabaseStatus::Running
    } else {
        database_manager::DatabaseStatus::Stopped
    };
    
    let port_output = session.exec_command("sudo -u postgres psql -c \"SHOW port;\" 2>/dev/null | tail -3 | head -1 | tr -d ' '").await.unwrap_or_default();
    let port = port_output.trim().parse::<u16>().ok();
    
    let path_output = session.exec_command("which psql 2>/dev/null || echo ''").await.unwrap_or_default();
    let install_path = path_output.trim().to_string().into();
    
    Ok(DatabaseDetectionResult {
        db_type: DatabaseType::PostgreSQL,
        installed: true,
        version,
        status,
        port: port.or(Some(5432)),
        install_path,
    })
}

async fn detect_redis_direct(session_arc: &Arc<Mutex<SSHSession>>) -> Result<DatabaseDetectionResult, String> {
    let session = session_arc.lock().await;
    
    let version_output = session.exec_command("redis-server --version 2>/dev/null || echo 'NOT_FOUND'").await
        .map_err(|e| e.to_string())?;
    
    if version_output.contains("NOT_FOUND") {
        return Ok(DatabaseDetectionResult {
            db_type: DatabaseType::Redis,
            installed: false,
            version: None,
            status: database_manager::DatabaseStatus::NotInstalled,
            port: None,
            install_path: None,
        });
    }
    
    let version = extract_version_from_output(&version_output, &["v=", "Redis server"]);
    
    let status_cmd = "sudo systemctl is-active redis-server 2>/dev/null || sudo systemctl is-active redis 2>/dev/null || sudo service redis-server status 2>&1 | grep -q running && echo 'active' || echo 'inactive'";
    let status_output = session.exec_command(status_cmd).await.unwrap_or_default();
    let status = if status_output.trim() == "active" {
        database_manager::DatabaseStatus::Running
    } else {
        database_manager::DatabaseStatus::Stopped
    };
    
    let port_output = session.exec_command("redis-cli CONFIG GET port 2>/dev/null | tail -1").await.unwrap_or_default();
    let port = port_output.trim().parse::<u16>().ok();
    
    let path_output = session.exec_command("which redis-server 2>/dev/null || echo ''").await.unwrap_or_default();
    let install_path = path_output.trim().to_string().into();
    
    Ok(DatabaseDetectionResult {
        db_type: DatabaseType::Redis,
        installed: true,
        version,
        status,
        port: port.or(Some(6379)),
        install_path,
    })
}

async fn detect_mongodb_direct(session_arc: &Arc<Mutex<SSHSession>>) -> Result<DatabaseDetectionResult, String> {
    let session = session_arc.lock().await;
    
    let version_output = session.exec_command("mongod --version 2>/dev/null || echo 'NOT_FOUND'").await
        .map_err(|e| e.to_string())?;
    
    if version_output.contains("NOT_FOUND") {
        return Ok(DatabaseDetectionResult {
            db_type: DatabaseType::MongoDB,
            installed: false,
            version: None,
            status: database_manager::DatabaseStatus::NotInstalled,
            port: None,
            install_path: None,
        });
    }
    
    let version = extract_version_from_output(&version_output, &["db version", "v"]);
    
    let status_cmd = "sudo systemctl is-active mongod 2>/dev/null || sudo service mongod status 2>&1 | grep -q running && echo 'active' || echo 'inactive'";
    let status_output = session.exec_command(status_cmd).await.unwrap_or_default();
    let status = if status_output.trim() == "active" {
        database_manager::DatabaseStatus::Running
    } else {
        database_manager::DatabaseStatus::Stopped
    };
    
    let path_output = session.exec_command("which mongod 2>/dev/null || echo ''").await.unwrap_or_default();
    let install_path = path_output.trim().to_string().into();
    
    Ok(DatabaseDetectionResult {
        db_type: DatabaseType::MongoDB,
        installed: true,
        version,
        status,
        port: Some(27017),
        install_path,
    })
}

async fn detect_mariadb_direct(session_arc: &Arc<Mutex<SSHSession>>) -> Result<DatabaseDetectionResult, String> {
    let session = session_arc.lock().await;
    
    let version_output = session.exec_command("mariadb --version 2>/dev/null || mysql --version 2>/dev/null || echo 'NOT_FOUND'").await
        .map_err(|e| e.to_string())?;
    
    if version_output.contains("NOT_FOUND") || !version_output.to_lowercase().contains("mariadb") {
        return Ok(DatabaseDetectionResult {
            db_type: DatabaseType::MariaDB,
            installed: false,
            version: None,
            status: database_manager::DatabaseStatus::NotInstalled,
            port: None,
            install_path: None,
        });
    }
    
    let version = extract_version_from_output(&version_output, &["Ver", "version"]);
    
    let status_cmd = "sudo systemctl is-active mariadb 2>/dev/null || sudo service mariadb status 2>&1 | grep -q running && echo 'active' || echo 'inactive'";
    let status_output = session.exec_command(status_cmd).await.unwrap_or_default();
    let status = if status_output.trim() == "active" {
        database_manager::DatabaseStatus::Running
    } else {
        database_manager::DatabaseStatus::Stopped
    };
    
    let port_output = session.exec_command("mariadb -u root -e \"SHOW VARIABLES LIKE 'port';\" 2>/dev/null | tail -1 | awk '{print $2}'").await.unwrap_or_default();
    let port = port_output.trim().parse::<u16>().ok();
    
    let path_output = session.exec_command("which mariadb 2>/dev/null || which mysql 2>/dev/null || echo ''").await.unwrap_or_default();
    let install_path = path_output.trim().to_string().into();
    
    Ok(DatabaseDetectionResult {
        db_type: DatabaseType::MariaDB,
        installed: true,
        version,
        status,
        port: port.or(Some(3306)),
        install_path,
    })
}

fn extract_version_from_output(output: &str, keywords: &[&str]) -> Option<String> {
    for line in output.lines() {
        for keyword in keywords {
            if let Some(pos) = line.find(keyword) {
                let after = &line[pos + keyword.len()..];
                // 简单提取版本号
                let parts: Vec<&str> = after.split_whitespace().collect();
                for part in parts {
                    if part.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        return Some(part.trim_matches(|c: char| !c.is_ascii_digit() && c != '.').to_string());
                    }
                }
            }
        }
    }
    None
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallDatabaseParams {
    session_id: String,
    db_type: DatabaseType,
    options: InstallOptions,
}

#[tauri::command]
async fn install_database(params: InstallDatabaseParams) -> Result<String, String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    let session_arc = sessions
        .get(&params.session_id)
        .cloned()
        .ok_or("Session not found")?;

    let session = session_arc.lock().await;
    
    // 检测操作系统
    let os_info = detect_os_info(&session).await.map_err(|e| e.to_string())?;
    
    // 生成安装脚本
    let install_script = generate_install_script(&params.db_type, &os_info, &params.options);
    
    // 执行安装
    let output = session.exec_command(&install_script).await.map_err(|e| e.to_string())?;
    
    Ok(output)
}

async fn detect_os_info(session: &SSHSession) -> Result<OsInfo> {
    let os_release = session.exec_command("cat /etc/os-release 2>/dev/null || echo 'ID=unknown'").await?;
    
    let mut id = "unknown".to_string();
    let mut like = None;

    for line in os_release.lines() {
        if line.starts_with("ID=") {
            id = line.trim_start_matches("ID=").trim_matches('"').to_string();
        } else if line.starts_with("ID_LIKE=") {
            like = Some(line.trim_start_matches("ID_LIKE=").trim_matches('"').to_string());
        }
    }

    let package_manager = if id.contains("debian") || id.contains("ubuntu") || like.as_ref().map(|l| l.contains("debian")).unwrap_or(false) {
        PackageManager::Apt
    } else if id.contains("centos") || id.contains("rhel") || id.contains("fedora") || like.as_ref().map(|l| l.contains("rhel")).unwrap_or(false) {
        PackageManager::Yum
    } else {
        PackageManager::Unknown
    };

    Ok(OsInfo {
        id,
        package_manager,
    })
}

#[derive(Debug, Clone)]
struct OsInfo {
    id: String,
    package_manager: PackageManager,
}

#[derive(Debug, Clone)]
enum PackageManager {
    Apt,
    Yum,
    Unknown,
}

fn generate_install_script(db_type: &DatabaseType, os: &OsInfo, options: &InstallOptions) -> String {
    let port = options.port.unwrap_or_else(|| db_type.default_port());
    let password = options.root_password.as_deref().unwrap_or("root");
    
    match db_type {
        DatabaseType::MySQL => generate_mysql_install(os, port, password),
        DatabaseType::PostgreSQL => generate_postgresql_install(os, port),
        DatabaseType::Redis => generate_redis_install(os, port),
        DatabaseType::MongoDB => generate_mongodb_install(os, port),
        DatabaseType::MariaDB => generate_mariadb_install(os, port, password),
        _ => "echo 'Unsupported database type'".to_string(),
    }
}

fn generate_mysql_install(os: &OsInfo, port: u16, password: &str) -> String {
    match os.package_manager {
        PackageManager::Apt => format!(
            r#"export DEBIAN_FRONTEND=noninteractive && 
sudo apt-get update && 
sudo apt-get install -y mysql-server && 
sudo systemctl start mysql && 
sudo systemctl enable mysql && 
sudo mysql -e "ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY '{}'; FLUSH PRIVILEGES;" && 
echo "MySQL installed successfully on port {}""#,
            password, port
        ),
        PackageManager::Yum => format!(
            r#"sudo yum install -y mysql-server && 
sudo systemctl start mysqld && 
sudo systemctl enable mysqld && 
echo "MySQL installed successfully on port {}""#,
            port
        ),
        _ => "echo 'Unsupported package manager'".to_string(),
    }
}

fn generate_postgresql_install(os: &OsInfo, port: u16) -> String {
    match os.package_manager {
        PackageManager::Apt => format!(
            r#"sudo apt-get update && 
sudo apt-get install -y postgresql postgresql-contrib && 
sudo systemctl start postgresql && 
sudo systemctl enable postgresql && 
sudo -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'postgres';" && 
echo "PostgreSQL installed successfully on port {}""#,
            port
        ),
        PackageManager::Yum => format!(
            r#"sudo yum install -y postgresql-server postgresql-contrib && 
sudo postgresql-setup initdb 2>/dev/null || true && 
sudo systemctl start postgresql && 
sudo systemctl enable postgresql && 
echo "PostgreSQL installed successfully on port {}""#,
            port
        ),
        _ => "echo 'Unsupported package manager'".to_string(),
    }
}

fn generate_redis_install(os: &OsInfo, port: u16) -> String {
    match os.package_manager {
        PackageManager::Apt => format!(
            r#"sudo apt-get update && 
sudo apt-get install -y redis-server && 
sudo sed -i 's/^#*port .*/port {}/' /etc/redis/redis.conf 2>/dev/null || true && 
sudo systemctl restart redis-server && 
sudo systemctl enable redis-server && 
echo "Redis installed successfully on port {}""#,
            port, port
        ),
        PackageManager::Yum => format!(
            r#"sudo yum install -y redis && 
sudo systemctl start redis && 
sudo systemctl enable redis && 
echo "Redis installed successfully on port {}""#,
            port
        ),
        _ => "echo 'Unsupported package manager'".to_string(),
    }
}

fn generate_mongodb_install(os: &OsInfo, port: u16) -> String {
    match os.package_manager {
        PackageManager::Apt => format!(
            r#"curl -fsSL https://pgp.mongodb.com/server-7.0.asc | sudo gpg -o /usr/share/keyrings/mongodb-server-7.0.gpg --dearmor 2>/dev/null && 
echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-7.0.gpg ] https://repo.mongodb.org/apt/ubuntu $(lsb_release -cs 2>/dev/null || echo 'jammy')/mongodb-org/7.0 multiverse" | sudo tee /etc/apt/sources.list.d/mongodb-org-7.0.list && 
sudo apt-get update && 
sudo apt-get install -y mongodb-org && 
sudo systemctl start mongod && 
sudo systemctl enable mongod && 
echo "MongoDB installed successfully on port {}""#,
            port
        ),
        PackageManager::Yum => format!(
            r#"echo '[mongodb-org-7.0]
name=MongoDB Repository
baseurl=https://repo.mongodb.org/yum/redhat/\$releasever/mongodb-org/7.0/x86_64/
gpgcheck=1
enabled=1
gpgkey=https://pgp.mongodb.com/server-7.0.asc' | sudo tee /etc/yum.repos.d/mongodb-org-7.0.repo && 
sudo yum install -y mongodb-org && 
sudo systemctl start mongod && 
sudo systemctl enable mongod && 
echo "MongoDB installed successfully on port {}""#,
            port
        ),
        _ => "echo 'Unsupported package manager'".to_string(),
    }
}

fn generate_mariadb_install(os: &OsInfo, port: u16, password: &str) -> String {
    match os.package_manager {
        PackageManager::Apt => format!(
            r#"sudo apt-get update && 
sudo apt-get install -y mariadb-server && 
sudo systemctl start mariadb && 
sudo systemctl enable mariadb && 
sudo mysql -e "ALTER USER 'root'@'localhost' IDENTIFIED BY '{}'; FLUSH PRIVILEGES;" && 
echo "MariaDB installed successfully on port {}""#,
            password, port
        ),
        PackageManager::Yum => format!(
            r#"sudo yum install -y mariadb-server && 
sudo systemctl start mariadb && 
sudo systemctl enable mariadb && 
echo "MariaDB installed successfully on port {}""#,
            port
        ),
        _ => "echo 'Unsupported package manager'".to_string(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManageServiceParams {
    session_id: String,
    service_name: String,
    action: String, // "start", "stop", "restart", "enable"
}

#[tauri::command]
async fn manage_database_service(params: ManageServiceParams) -> Result<String, String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    let session_arc = sessions
        .get(&params.session_id)
        .cloned()
        .ok_or("Session not found")?;

    let session = session_arc.lock().await;
    
    let cmd = match params.action.as_str() {
        "start" => format!("sudo systemctl start {} 2>&1 || sudo service {} start 2>&1", params.service_name, params.service_name),
        "stop" => format!("sudo systemctl stop {} 2>&1 || sudo service {} stop 2>&1", params.service_name, params.service_name),
        "restart" => format!("sudo systemctl restart {} 2>&1 || sudo service {} restart 2>&1", params.service_name, params.service_name),
        "enable" => format!("sudo systemctl enable {} 2>&1", params.service_name),
        _ => return Err(format!("Unknown action: {}", params.action)),
    };

    session.exec_command(&cmd).await.map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetDatabaseConfigParams {
    session_id: String,
    db_type: DatabaseType,
}

#[tauri::command]
async fn get_database_config(params: GetDatabaseConfigParams) -> Result<String, String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    let session_arc = sessions
        .get(&params.session_id)
        .cloned()
        .ok_or("Session not found")?;

    let session = session_arc.lock().await;
    
    let config_path = match params.db_type {
        DatabaseType::MySQL | DatabaseType::MariaDB => "/etc/mysql/my.cnf /etc/my.cnf",
        DatabaseType::PostgreSQL => "/etc/postgresql/*/main/postgresql.conf",
        DatabaseType::Redis => "/etc/redis/redis.conf /etc/redis.conf",
        DatabaseType::MongoDB => "/etc/mongod.conf",
        _ => return Err(format!("暂不支持获取 {:?} 配置", params.db_type)),
    };

    let cmd = format!("cat {} 2>/dev/null | head -500", config_path);
    session.exec_command(&cmd).await.map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateDatabaseConfigParams {
    session_id: String,
    db_type: DatabaseType,
    config_content: String,
}

#[tauri::command]
async fn update_database_config(params: UpdateDatabaseConfigParams) -> Result<String, String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    let session_arc = sessions
        .get(&params.session_id)
        .cloned()
        .ok_or("Session not found")?;

    let session = session_arc.lock().await;
    
    let config_path = match params.db_type {
        DatabaseType::MySQL | DatabaseType::MariaDB => "/etc/mysql/my.cnf",
        DatabaseType::Redis => "/etc/redis/redis.conf",
        DatabaseType::MongoDB => "/etc/mongod.conf",
        _ => return Err(format!("暂不支持修改 {:?} 配置", params.db_type)),
    };

    // 备份原配置
    let backup_cmd = format!("sudo cp {} {}.backup.$(date +%Y%m%d_%H%M%S) 2>&1", config_path, config_path);
    let _ = session.exec_command(&backup_cmd).await;

    // 写入新配置 (使用 base64 避免转义问题)
    let base64_content = base64_encode(params.config_content.as_bytes());
    let cmd = format!("echo '{}' | base64 -d | sudo tee {} > /dev/null 2>&1", base64_content, config_path);
    
    session.exec_command(&cmd).await.map_err(|e| e.to_string())
}

// 需要在 run() 中注册新命令
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            connect_ssh,
            write_to_pty,
            resize_pty,
            disconnect_ssh,
            browser_open,
            list_machines,
            add_machine,
            update_machine,
            delete_machine,
            test_connection,
            get_system_stats,
            kill_process,
            list_files,
            create_folder,
            create_file,
            delete_file_or_folder,
            rename_file,
            download_file,
            upload_file,
            chmod_file,
            // 数据库管理命令
            detect_databases,
            install_database,
            manage_database_service,
            get_database_config,
            update_database_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
