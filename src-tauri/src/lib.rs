use anyhow::Result;
use russh::client::Handle;
use russh::{ChannelId, Disconnect};
use russh::keys::PublicKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use russh::CryptoVec;
use log::info;
use tauri::Emitter;

// SSH 会话管理
pub struct SSHSession {
    handle: Handle<SSHClient>,
    channel_id: ChannelId,
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
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(1024);
        let client = SSHClient::new(tx);

        info!("正在建立 TCP 连接...");
        let mut handle = russh::client::connect(config, (host, port), client).await?;
        info!("TCP 连接成功，开始认证...");

        // 认证
        let authenticated = if let Some(key_path) = private_key {
            info!("使用密钥认证: {}", key_path);
            let key_pair = russh::keys::load_secret_key(key_path, None)?;
            let key_with_hash = russh::keys::PrivateKeyWithHashAlg::new(
                Arc::new(key_pair),
                None
            );
            handle.authenticate_publickey(username, key_with_hash).await.is_ok()
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

        // 打开通道
        info!("打开 SSH 通道...");
        let channel = handle.channel_open_session().await?;
        channel.request_pty(false, "xterm-256color", 80, 24, 0, 0, &[]).await?;
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
                let _ = app_handle_clone.emit(&format!("ssh-data-{}", session_id_clone), data_base64);
            }
            info!("数据转发任务结束");
        });

        Ok(SSHSession {
            handle,
            channel_id,
        })
    }

    pub async fn write(&self, data: &[u8]) -> Result<()> {
        let crypto_vec = CryptoVec::from_slice(data);
        self.handle.data(self.channel_id, crypto_vec).await
            .map_err(|e| anyhow::anyhow!("Failed to write data: {:?}", e))?;
        Ok(())
    }

    pub async fn resize(&self, cols: u32, rows: u32) -> Result<()> {
        log::debug!("Resize requested: {}x{}", cols, rows);
        Ok(())
    }

    pub async fn disconnect(self) -> Result<()> {
        self.handle.disconnect(Disconnect::ByApplication, "", "").await?;
        Ok(())
    }
}

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

    fn check_server_key(&mut self, _server_public_key: &PublicKey) -> impl std::future::Future<Output = Result<bool>> + Send {
        async move {
            Ok(true)
        }
    }

    fn data(&mut self, _channel: ChannelId, data: &[u8], _session: &mut russh::client::Session) -> impl std::future::Future<Output = Result<()>> + Send {
        let data = data.to_vec();
        let tx = self.tx.clone();
        async move {
            let _ = tx.send(data).await;
            Ok(())
        }
    }

    fn extended_data(&mut self, _channel: ChannelId, _ext: u32, data: &[u8], _session: &mut russh::client::Session) -> impl std::future::Future<Output = Result<()>> + Send {
        let data = data.to_vec();
        let tx = self.tx.clone();
        async move {
            let _ = tx.send(data).await;
            Ok(())
        }
    }
}

// base64 编码函数
fn base64_encode(data: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(data)
}

// 全局会话管理
use std::sync::OnceLock;

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
    info!("收到连接请求: session_id={}, params={:?}", session_id, params);
    
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
        session.write(data.as_bytes()).await.map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
async fn resize_pty(session_id: String, cols: u32, rows: u32) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;
    
    if let Some(session) = sessions.get(&session_id) {
        let session = session.lock().await;
        session.resize(cols, rows).await.map_err(|e| e.to_string())?;
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
            disconnect_ssh
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
