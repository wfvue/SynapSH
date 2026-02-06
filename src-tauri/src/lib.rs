use anyhow::Result;
use russh::client::{self, Handle, Msg};
use russh::{Channel, ChannelId, ChannelMsg, Disconnect};
use russh_keys::key::PublicKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
use tokio::sync::{mpsc, Mutex};

// SSH 会话管理
pub struct SSHSession {
    handle: Handle<SSHClient>,
    channel_id: ChannelId,
    tx: mpsc::Sender<Vec<u8>>,
    rx: Arc<Mutex<mpsc::Receiver<Vec<u8>>>>,
}

impl SSHSession {
    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        password: Option<String>,
        private_key: Option<String>,
    ) -> Result<Self> {
        let config = russh::client::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(300)),
            ..Default::default()
        };
        let config = Arc::new(config);

        let (tx, rx) = mpsc::channel::<Vec<u8>>(1024);
        let client = SSHClient::new(tx.clone());

        let handle = russh::client::connect(config, (host, port), client).await?;

        // 认证
        let authenticated = if let Some(key_path) = private_key {
            let key_pair = russh_keys::load_secret_key(key_path, None)?;
            handle.authenticate_publickey(username, Arc::new(key_pair)).await?
        } else if let Some(pass) = password {
            handle.authenticate_password(username, pass).await?
        } else {
            false
        };

        if !authenticated {
            return Err(anyhow::anyhow!("Authentication failed"));
        }

        // 打开通道
        let mut channel = handle.channel_open_session().await?;
        channel.request_pty(false, "xterm-256color", 80, 24, 0, 0, &[]).await?;
        channel.request_shell(false).await?;

        let channel_id = channel.id();

        // 启动读取任务
        let rx_mutex = Arc::new(Mutex::new(rx));
        let rx_clone = rx_mutex.clone();

        tokio::spawn(async move {
            let mut rx = rx_clone.lock().await;
            while let Some(data) = rx.recv().await {
                // 数据通过前端事件发送
            }
        });

        Ok(SSHSession {
            handle,
            channel_id,
            tx,
            rx: rx_mutex,
        })
    }

    pub async fn write(&self, data: &[u8]) -> Result<()> {
        self.handle.data(self.channel_id, bytes::Bytes::copy_from_slice(data)).await?;
        Ok(())
    }

    pub async fn resize(&self, cols: u32, rows: u32) -> Result<()> {
        self.handle
            .window_change(self.channel_id, cols, rows, 0, 0)
            .await?;
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

#[async_trait::async_trait]
impl russh::client::Handler for SSHClient {
    type Error = anyhow::Error;

    async fn check_server_key(&mut self, server_public_key: &PublicKey) -> Result<bool> {
        // TODO: 实现主机密钥验证
        Ok(true)
    }

    async fn data(&mut self, channel: ChannelId, data: &[u8]) -> Result<()> {
        let _ = self.tx.send(data.to_vec()).await;
        Ok(())
    }

    async fn extended_data(&mut self, channel: ChannelId, ext: u32, data: &[u8]) -> Result<()> {
        let _ = self.tx.send(data.to_vec()).await;
        Ok(())
    }
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
    let session = SSHSession::connect(
        &params.host,
        params.port,
        &params.username,
        params.password,
        params.private_key,
    )
    .await
    .map_err(|e| e.to_string())?;

    let sessions = get_sessions();
    let mut sessions = sessions.lock().await;
    sessions.insert(session_id, Arc::new(Mutex::new(session)));

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
    
    if let Some(session) = sessions.remove(&session_id) {
        let session = session.lock().await;
        // 由于 session 是 MutexGuard，我们需要先释放它
        // 这里简化处理
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
