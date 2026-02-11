use anyhow::{Context, Result};
use log::info;
use russh::client::Handle;
use russh::keys::PublicKey;
use russh::{ChannelId, Disconnect};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use tauri::Emitter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch, Mutex};

use crate::files::{FileEntry, FileEntryType, FileListResult};

// SSH 会话管理
pub struct SSHSession {
    // 回退到 Arc<Mutex> 以解决 russh Handle 不支持 Clone 的问题
    // 但我们会优化锁的粒度，确保 IO 并发
    pub handle: Arc<Mutex<Handle<SSHClient>>>,
    pub channel: russh::Channel<russh::client::Msg>,
    #[allow(dead_code)]
    pub shell_channel_id: Arc<Mutex<Option<ChannelId>>>,
    pub socks5_proxy: Option<Socks5Proxy>,
    pub sftp_session: Arc<Mutex<Option<russh_sftp::client::SftpSession>>>,
}

pub struct Socks5Proxy {
    pub port: u16,
    pub shutdown: watch::Sender<bool>,
}

impl Socks5Proxy {
    async fn start(
        handle: Arc<Mutex<Handle<SSHClient>>>,
        _app: tauri::AppHandle,
        _session_id: String,
    ) -> Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let port = listener.local_addr()?.port();
        let (shutdown, mut shutdown_rx) = watch::channel(false);

        log::info!("SOCKS5 Proxy started on port {}", port);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                    accept = listener.accept() => {
                        match accept {
                            Ok((stream, _addr)) => {
                                let handle = handle.clone();
                                tokio::spawn(async move {
                                    if let Err(err) = handle_socks5_client(stream, handle).await {
                                        log::debug!("SOCKS5 client error: {err}");
                                    }
                                });
                            }
                            Err(err) => {
                                log::warn!("SOCKS5 accept error: {err}");
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

impl Drop for Socks5Proxy {
    fn drop(&mut self) {
        let _ = self.shutdown.send(true);
    }
}

impl SSHSession {
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

        // 启动数据转发任务
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
            socks5_proxy: None,
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

    #[allow(dead_code)]
    pub async fn disconnect(self) -> Result<()> {
        let handle = self.handle.lock().await;
        let result = handle.disconnect(Disconnect::ByApplication, "", "").await;
        Ok(result?)
    }

    /// 执行一次性命令并返回输出
    pub async fn exec_command(&self, command: &str) -> Result<String> {
        let mut channel = {
            let handle = self.handle.lock().await;
            let channel: russh::Channel<russh::client::Msg> = handle.channel_open_session().await?;
            channel
        }; // Lock released here!

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

    pub async fn ensure_socks5_proxy(
        &mut self,
        app: tauri::AppHandle,
        session_id: String,
    ) -> Result<u16> {
        if let Some(proxy) = &self.socks5_proxy {
            return Ok(proxy.port);
        }

        let proxy = Socks5Proxy::start(self.handle.clone(), app, session_id).await?;
        let port = proxy.port;
        self.socks5_proxy = Some(proxy);
        Ok(port)
    }

    /// 获取或创建 SFTP 会话
    pub async fn get_sftp(
        &self,
    ) -> Result<tokio::sync::MutexGuard<'_, Option<russh_sftp::client::SftpSession>>> {
        let start = std::time::Instant::now();
        let mut guard = self.sftp_session.lock().await;

        if guard.is_none() {
            log::info!("[SFTP] Initializing new SFTP session...");
            // Lock scope optimization
            let channel = {
                let handle = self.handle.lock().await;
                handle.channel_open_session().await?
            };

            channel.request_subsystem(false, "sftp").await?;
            let sftp = russh_sftp::client::SftpSession::new(channel.into_stream()).await?;
            *guard = Some(sftp);
            log::info!("[SFTP] Total SFTP init: {:?}", start.elapsed());
        }

        Ok(guard)
    }

    /// 列出目录内容
    pub async fn list_directory(&self, path: &str) -> Result<FileListResult> {
        let mut guard = self.get_sftp().await?;
        let sftp = guard.as_ref().unwrap();

        // 读取目录
        let entries = match sftp.read_dir(path).await {
            Ok(e) => e,
            Err(e) => {
                log::warn!("SFTP read_dir failed, invalidating session: {}", e);
                *guard = None;
                return Err(e.into());
            }
        };

        let mut file_entries: Vec<FileEntry> = Vec::new();

        for entry in entries {
            let name = entry.file_name();
            let full_path = format!("{}/{}", path.trim_end_matches('/'), name);

            // 直接使用 DirEntry 自带的 metadata，无需额外网络请求
            let attrs = entry.metadata();
            let file_type = entry.file_type();

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

        let attrs = russh_sftp::protocol::FileAttributes {
            permissions: Some(mode),
            ..Default::default()
        };

        if let Err(e) = sftp.set_metadata(path, attrs).await {
            log::warn!("SFTP set_metadata failed: {}", e);
            *guard = None;
            return Err(e.into());
        }
        Ok(())
    }
}

// SSH 客户端处理器
#[derive(Clone)]
pub struct SSHClient {
    tx: mpsc::Sender<Vec<u8>>,
    shell_channel_id: Arc<Mutex<Option<ChannelId>>>,
}

impl SSHClient {
    pub fn new(tx: mpsc::Sender<Vec<u8>>, shell_channel_id: Arc<Mutex<Option<ChannelId>>>) -> Self {
        Self {
            tx,
            shell_channel_id,
        }
    }
}

impl russh::client::Handler for SSHClient {
    type Error = anyhow::Error;

    async fn check_server_key(&mut self, _server_public_key: &PublicKey) -> Result<bool> {
        Ok(true)
    }

    // 处理标准输出
    fn data(
        &mut self,
        channel: ChannelId,
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

    // 处理标准错误或其他扩展数据
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

// SOCKS5 协议常量
const SOCKS5_VERSION: u8 = 0x05;
const SOCKS5_AUTH_METHOD_NONE: u8 = 0x00;
const SOCKS5_CMD_CONNECT: u8 = 0x01;
const SOCKS5_ADDR_TYPE_IPV4: u8 = 0x01;
const SOCKS5_ADDR_TYPE_DOMAIN: u8 = 0x03;
const SOCKS5_ADDR_TYPE_IPV6: u8 = 0x04;

// 工业级 SOCKS5 处理器
async fn handle_socks5_client(
    mut client_stream: TcpStream,
    ssh_handle: Arc<Mutex<Handle<SSHClient>>>,
) -> Result<()> {
    // 1. 握手阶段 (Handshake)
    let mut header = [0u8; 2];
    client_stream.read_exact(&mut header).await?;

    if header[0] != SOCKS5_VERSION {
        return Err(anyhow::anyhow!("Unsupported SOCKS version: {}", header[0]));
    }

    let n_methods = header[1] as usize;
    let mut methods = vec![0u8; n_methods];
    client_stream.read_exact(&mut methods).await?;

    if !methods.contains(&SOCKS5_AUTH_METHOD_NONE) {
        client_stream.write_all(&[SOCKS5_VERSION, 0xFF]).await?;
        return Err(anyhow::anyhow!("No supported auth method"));
    }

    // 接受无认证
    client_stream
        .write_all(&[SOCKS5_VERSION, SOCKS5_AUTH_METHOD_NONE])
        .await?;

    // 2. 请求阶段 (Request)
    let mut request_header = [0u8; 4];
    client_stream.read_exact(&mut request_header).await?;

    if request_header[0] != SOCKS5_VERSION {
        return Err(anyhow::anyhow!("Invalid version in request"));
    }
    if request_header[1] != SOCKS5_CMD_CONNECT {
        return Err(anyhow::anyhow!(
            "Unsupported command: {}",
            request_header[1]
        ));
    }

    let addr_type = request_header[3];
    let (target_host, target_port) = match addr_type {
        SOCKS5_ADDR_TYPE_IPV4 => {
            let mut addr = [0u8; 4];
            client_stream.read_exact(&mut addr).await?;
            let mut port = [0u8; 2];
            client_stream.read_exact(&mut port).await?;
            (
                IpAddr::V4(Ipv4Addr::from(addr)).to_string(),
                u16::from_be_bytes(port),
            )
        }
        SOCKS5_ADDR_TYPE_DOMAIN => {
            let mut len = [0u8; 1];
            client_stream.read_exact(&mut len).await?;
            let domain_len = len[0] as usize;
            let mut domain = vec![0u8; domain_len];
            client_stream.read_exact(&mut domain).await?;
            let mut port = [0u8; 2];
            client_stream.read_exact(&mut port).await?;
            (
                String::from_utf8(domain).context("Invalid domain")?,
                u16::from_be_bytes(port),
            )
        }
        SOCKS5_ADDR_TYPE_IPV6 => {
            let mut addr = [0u8; 16];
            client_stream.read_exact(&mut addr).await?;
            let mut port = [0u8; 2];
            client_stream.read_exact(&mut port).await?;
            (
                IpAddr::V6(Ipv6Addr::from(addr)).to_string(),
                u16::from_be_bytes(port),
            )
        }
        _ => return Err(anyhow::anyhow!("Unknown address type")),
    };

    log::debug!("SOCKS5 CONNECT request to {}:{}", target_host, target_port);

    // 3. 建立 SSH 隧道
    // 关键优化：锁的作用域极小，只在建立通道时持有
    let channel = {
        let handle = ssh_handle.lock().await;
        match handle
            .channel_open_direct_tcpip(
                target_host.clone(),
                target_port as u32,
                "127.0.0.1".to_string(),
                0,
            )
            .await
        {
            Ok(c) => c,
            Err(e) => {
                // 连接失败响应
                let _ = client_stream
                    .write_all(&[SOCKS5_VERSION, 0x04, 0x00, addr_type, 0, 0, 0, 0, 0, 0])
                    .await;
                return Err(e.into());
            }
        }
    }; // Lock released

    // 连接成功响应
    client_stream
        .write_all(&[SOCKS5_VERSION, 0x00, 0x00, addr_type, 0, 0, 0, 0, 0, 0])
        .await?;

    // 4. 双向转发 (Zero-Copy)
    let (mut client_read, mut client_write) = client_stream.into_split();
    let channel_stream = channel.into_stream();
    let (mut channel_read, mut channel_write) = tokio::io::split(channel_stream);

    // Client -> Server
    let client_to_server = tokio::spawn(async move {
        let _ = tokio::io::copy(&mut client_read, &mut channel_write).await;
        let _ = channel_write.shutdown().await;
    });

    // Server -> Client
    let server_to_client = tokio::spawn(async move {
        let _ = tokio::io::copy(&mut channel_read, &mut client_write).await;
        let _ = client_write.shutdown().await;
    });

    let _ = tokio::try_join!(client_to_server, server_to_client);

    Ok(())
}

// base64 编码辅助函数
pub fn base64_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.encode(data)
}
