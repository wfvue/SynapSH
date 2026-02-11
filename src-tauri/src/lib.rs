//! Tauri 后端命令入口，负责 SSH 会话与桌面应用能力编排。

use anyhow::Result;
use log::info;
use russh::Disconnect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

mod db;
use db::{Database, Machine, MachineInput};

mod database_manager;
use database_manager::{
    CommandExecutor, DatabaseDetectionResult, DatabaseManager, DatabaseType, InstallOptions,
};

mod ssh;
use ssh::{SSHClient, SSHSession};

mod files;
use files::FileListResult;

mod monitor;
use monitor::{ProcessInfo, SystemStats};

mod utils;
use utils::{base64_decode, base64_encode, chrome_profile_dir, normalize_url};

struct SessionExecutor<'a>(&'a SSHSession);

#[async_trait::async_trait]
impl<'a> CommandExecutor for SessionExecutor<'a> {
    async fn execute(&self, command: &str) -> Result<String> {
        self.0
            .exec_command(command)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }
}

// Models moved to modules

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrowserLaunchOptions {
    profile_mode: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct BrowserProxyError {
    session_id: String,
    host: String,
    port: u16,
    message: String,
}

fn emit_browser_proxy_error(
    app: &tauri::AppHandle,
    session_id: &str,
    host: &str,
    port: u16,
    message: impl Into<String>,
) {
    let payload = BrowserProxyError {
        session_id: session_id.to_string(),
        host: host.to_string(),
        port,
        message: message.into(),
    };
    if let Err(err) = app.emit("browser-proxy-error", payload) {
        log::warn!("发送 browser-proxy-error 事件失败: {}", err);
    }
}

async fn verify_socks5_proxy_connectivity(proxy_port: u16) -> Result<(), String> {
    let timeout = std::time::Duration::from_millis(800);
    let mut stream = tokio::time::timeout(timeout, TcpStream::connect(("127.0.0.1", proxy_port)))
        .await
        .map_err(|_| "连接本地 SOCKS5 代理超时".to_string())?
        .map_err(|e| format!("连接本地 SOCKS5 代理失败: {e}"))?;

    tokio::time::timeout(timeout, stream.write_all(&[0x05, 0x01, 0x00]))
        .await
        .map_err(|_| "发送 SOCKS5 握手超时".to_string())?
        .map_err(|e| format!("发送 SOCKS5 握手失败: {e}"))?;

    let mut method_reply = [0u8; 2];
    tokio::time::timeout(timeout, stream.read_exact(&mut method_reply))
        .await
        .map_err(|_| "读取 SOCKS5 握手响应超时".to_string())?
        .map_err(|e| format!("读取 SOCKS5 握手响应失败: {e}"))?;

    if method_reply != [0x05, 0x00] {
        return Err(format!(
            "SOCKS5 握手响应异常: [{:#x}, {:#x}]",
            method_reply[0], method_reply[1]
        ));
    }

    Ok(())
}

// Implementations moved to ssh.rs

// Helper functions removed (moved to monitor.rs / ssh.rs / utils.rs)

static SESSIONS: OnceLock<Mutex<HashMap<String, Arc<Mutex<SSHSession>>>>> = OnceLock::new();
static BROWSER_PROCESSES: OnceLock<StdMutex<HashMap<String, BrowserProcess>>> = OnceLock::new();
static PROXY_SIDECARS: OnceLock<StdMutex<HashMap<String, ProxySidecarPool>>> = OnceLock::new();
static SESSION_CONNECTIONS: OnceLock<Mutex<HashMap<String, SessionConnectionInfo>>> =
    OnceLock::new();
const OPENSH_SIDECAR_POOL_DEFAULT: usize = 2;
const OPENSH_SIDECAR_POOL_MAX: usize = 4;

struct BrowserProcess {
    pid: u32,
    child: Child,
}

struct ProxySidecar {
    pid: u32,
    port: u16,
    child: Child,
    askpass_script: Option<PathBuf>,
    stderr_log: Option<PathBuf>,
}

struct ProxySidecarPool {
    sidecars: Vec<ProxySidecar>,
    next_index: usize,
}

#[derive(Clone)]
struct SessionConnectionInfo {
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    private_key: Option<String>,
}

fn get_sessions() -> &'static Mutex<HashMap<String, Arc<Mutex<SSHSession>>>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_browser_processes() -> &'static StdMutex<HashMap<String, BrowserProcess>> {
    BROWSER_PROCESSES.get_or_init(|| StdMutex::new(HashMap::new()))
}

fn get_proxy_sidecars() -> &'static StdMutex<HashMap<String, ProxySidecarPool>> {
    PROXY_SIDECARS.get_or_init(|| StdMutex::new(HashMap::new()))
}

fn get_session_connections() -> &'static Mutex<HashMap<String, SessionConnectionInfo>> {
    SESSION_CONNECTIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn openssh_sidecar_pool_size() -> usize {
    std::env::var("SYNAPSH_SIDECAR_POOL_SIZE")
        .ok()
        .and_then(|raw| raw.parse::<usize>().ok())
        .map(|size| size.clamp(1, OPENSH_SIDECAR_POOL_MAX))
        .unwrap_or(OPENSH_SIDECAR_POOL_DEFAULT)
}

fn cleanup_proxy_sidecar_temp_files(process: &mut ProxySidecar) {
    if let Some(path) = process.askpass_script.take() {
        if let Err(err) = fs::remove_file(&path) {
            log::warn!("清理 SSH_ASKPASS 临时脚本失败: path={:?}, error={}", path, err);
        }
    }
    if let Some(path) = process.stderr_log.take() {
        if let Err(err) = fs::remove_file(&path) {
            log::warn!("清理 OpenSSH sidecar 日志失败: path={:?}, error={}", path, err);
        }
    }
}

fn read_sidecar_log_tail(path: &PathBuf) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if lines.is_empty() {
        return None;
    }
    let start = lines.len().saturating_sub(4);
    Some(lines[start..].join(" | "))
}

fn sidecar_exit_message(process: &ProxySidecar, status: std::process::ExitStatus) -> String {
    let mut message = format!("OpenSSH sidecar 已退出: status={status}");
    if let Some(path) = &process.stderr_log {
        if let Some(tail) = read_sidecar_log_tail(path) {
            message.push_str(&format!(", stderr={tail}"));
        }
    }
    message
}

fn prune_stale_sidecars(pool: &mut ProxySidecarPool) {
    let mut idx = 0usize;
    while idx < pool.sidecars.len() {
        if is_proxy_sidecar_running(&mut pool.sidecars[idx]) {
            idx += 1;
            continue;
        }
        let mut stale = pool.sidecars.remove(idx);
        cleanup_proxy_sidecar_temp_files(&mut stale);
    }

    if pool.sidecars.is_empty() {
        pool.next_index = 0;
    } else {
        pool.next_index %= pool.sidecars.len();
    }
}

fn take_exited_sidecar_message(session_id: &str, proxy_port: u16) -> Option<String> {
    let mut guard = get_proxy_sidecars()
        .lock()
        .expect("proxy sidecar mutex poisoned");

    let (position, status) = match guard.get_mut(session_id) {
        Some(pool) => {
            let index = pool
                .sidecars
                .iter()
                .position(|process| process.port == proxy_port)?;
            match pool.sidecars[index].child.try_wait() {
                Ok(Some(status)) => (index, status),
                Ok(None) => return None,
                Err(err) => return Some(format!("检测 OpenSSH sidecar 状态失败: {err}")),
            }
        }
        None => return None,
    };

    let mut remove_session = false;
    let message = match guard.get_mut(session_id) {
        Some(pool) => {
            let mut exited = pool.sidecars.remove(position);
            let message = sidecar_exit_message(&exited, status);
            cleanup_proxy_sidecar_temp_files(&mut exited);

            if pool.sidecars.is_empty() {
                remove_session = true;
            } else if pool.next_index >= pool.sidecars.len() {
                pool.next_index %= pool.sidecars.len();
            }

            message
        }
        None => return None,
    };

    if remove_session {
        guard.remove(session_id);
    }

    Some(message)
}

async fn wait_sidecar_listener_ready(
    session_id: &str,
    proxy_port: u16,
    timeout: std::time::Duration,
) -> Result<(), String> {
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        if let Some(reason) = take_exited_sidecar_message(session_id, proxy_port) {
            return Err(reason);
        }

        let connect_result = tokio::time::timeout(
            std::time::Duration::from_millis(250),
            TcpStream::connect(("127.0.0.1", proxy_port)),
        )
        .await;

        if matches!(connect_result, Ok(Ok(_))) {
            return Ok(());
        }

        if tokio::time::Instant::now() >= deadline {
            return Err(format!(
                "等待 OpenSSH -D sidecar 监听本地端口超时: 127.0.0.1:{proxy_port}"
            ));
        }

        tokio::time::sleep(std::time::Duration::from_millis(180)).await;
    }
}

fn is_browser_running(process: &mut BrowserProcess) -> bool {
    match process.child.try_wait() {
        Ok(None) => true,
        Ok(Some(status)) => {
            log::info!(
                "检测到浏览器进程已退出: pid={}, status={}",
                process.pid,
                status
            );
            false
        }
        Err(err) => {
            log::warn!("检测浏览器进程状态失败: pid={}, error={}", process.pid, err);
            false
        }
    }
}

fn stop_browser_process(process: &mut BrowserProcess) {
    if !is_browser_running(process) {
        return;
    }

    if let Err(err) = process.child.kill() {
        log::warn!("结束浏览器进程失败: pid={}, error={}", process.pid, err);
    }
    if let Err(err) = process.child.wait() {
        log::warn!("等待浏览器进程退出失败: pid={}, error={}", process.pid, err);
    } else {
        log::info!("浏览器进程已结束: pid={}", process.pid);
    }
}

fn is_proxy_sidecar_running(process: &mut ProxySidecar) -> bool {
    match process.child.try_wait() {
        Ok(None) => true,
        Ok(Some(status)) => {
            log::warn!("{}", sidecar_exit_message(process, status));
            false
        }
        Err(err) => {
            log::warn!(
                "检测 OpenSSH sidecar 状态失败: pid={}, error={}",
                process.pid,
                err
            );
            false
        }
    }
}

fn stop_proxy_sidecar(process: &mut ProxySidecar) {
    if !is_proxy_sidecar_running(process) {
        cleanup_proxy_sidecar_temp_files(process);
        return;
    }

    if let Err(err) = process.child.kill() {
        log::warn!("结束 OpenSSH sidecar 失败: pid={}, error={}", process.pid, err);
    }
    if let Err(err) = process.child.wait() {
        log::warn!(
            "等待 OpenSSH sidecar 退出失败: pid={}, error={}",
            process.pid,
            err
        );
    } else {
        log::info!("OpenSSH sidecar 已结束: pid={}", process.pid);
    }
    cleanup_proxy_sidecar_temp_files(process);
}

fn close_browser_for_session(session_id: &str) {
    let mut guard = get_browser_processes()
        .lock()
        .expect("browser process mutex poisoned");
    if let Some(mut process) = guard.remove(session_id) {
        stop_browser_process(&mut process);
    }
}

fn close_proxy_sidecar_for_session(session_id: &str) {
    let mut guard = get_proxy_sidecars()
        .lock()
        .expect("proxy sidecar mutex poisoned");
    if let Some(mut pool) = guard.remove(session_id) {
        for mut process in pool.sidecars.drain(..) {
            stop_proxy_sidecar(&mut process);
        }
    }
}

fn close_all_browser_processes() {
    let mut guard = get_browser_processes()
        .lock()
        .expect("browser process mutex poisoned");
    for (_, mut process) in guard.drain() {
        stop_browser_process(&mut process);
    }
}

fn close_all_proxy_sidecars() {
    let mut guard = get_proxy_sidecars()
        .lock()
        .expect("proxy sidecar mutex poisoned");
    for (_, mut pool) in guard.drain() {
        for mut process in pool.sidecars.drain(..) {
            stop_proxy_sidecar(&mut process);
        }
    }
}

fn reserve_local_port() -> Result<u16, String> {
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).map_err(|e| format!("分配本地端口失败: {e}"))?;
    listener
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|e| format!("读取本地端口失败: {e}"))
}

fn sshpass_available() -> bool {
    Command::new("sshpass")
        .arg("-V")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn create_ssh_askpass_script() -> Result<PathBuf, String> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("读取系统时间失败: {e}"))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("synapsh-ssh-askpass-{ts}.sh"));
    let script = "#!/bin/sh\nprintf '%s\\n' \"$SYNAPSH_SSH_PASSWORD\"\n";
    fs::write(&path, script).map_err(|e| format!("写入 SSH_ASKPASS 脚本失败: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)
            .map_err(|e| format!("读取 SSH_ASKPASS 脚本权限失败: {e}"))?
            .permissions();
        perms.set_mode(0o700);
        fs::set_permissions(&path, perms)
            .map_err(|e| format!("设置 SSH_ASKPASS 脚本权限失败: {e}"))?;
    }

    Ok(path)
}

fn create_sidecar_stderr_log() -> Result<PathBuf, String> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("读取系统时间失败: {e}"))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("synapsh-openssh-sidecar-{ts}.log"));
    fs::write(&path, "").map_err(|e| format!("创建 sidecar 日志文件失败: {e}"))?;
    Ok(path)
}

fn spawn_openssh_sidecar(
    info: &SessionConnectionInfo,
    local_port: u16,
) -> Result<ProxySidecar, String> {
    let use_password_auth = info.private_key.is_none() && info.password.is_some();
    let mut askpass_script: Option<PathBuf> = None;
    let stderr_log = create_sidecar_stderr_log()?;

    let mut args: Vec<String> = vec![
        "-N".to_string(),
        "-D".to_string(),
        format!("127.0.0.1:{local_port}"),
        "-p".to_string(),
        info.port.to_string(),
        "-o".to_string(),
        "ExitOnForwardFailure=yes".to_string(),
        "-o".to_string(),
        "ServerAliveInterval=20".to_string(),
        "-o".to_string(),
        "ServerAliveCountMax=3".to_string(),
        "-o".to_string(),
        "ConnectTimeout=8".to_string(),
        "-o".to_string(),
        "LogLevel=ERROR".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=no".to_string(),
        "-o".to_string(),
        "UserKnownHostsFile=/dev/null".to_string(),
        "-o".to_string(),
        "GlobalKnownHostsFile=/dev/null".to_string(),
    ];

    if let Some(key_path) = &info.private_key {
        args.push("-i".to_string());
        args.push(key_path.clone());
        args.push("-o".to_string());
        args.push("IdentitiesOnly=yes".to_string());
        args.push("-o".to_string());
        args.push("BatchMode=yes".to_string());
    }

    if use_password_auth {
        args.push("-o".to_string());
        args.push("PreferredAuthentications=password,keyboard-interactive".to_string());
        args.push("-o".to_string());
        args.push("PubkeyAuthentication=no".to_string());
    }

    args.push(format!("{}@{}", info.username, info.host));

    let mut cmd = if use_password_auth {
        let password = info
            .password
            .as_ref()
            .ok_or_else(|| "密码认证参数缺失".to_string())?;
        if sshpass_available() {
            let mut c = Command::new("sshpass");
            c.arg("-p").arg(password).arg("ssh");
            c
        } else {
            let script_path = create_ssh_askpass_script()?;
            log::info!(
                "未检测到 sshpass，使用 SSH_ASKPASS 启动 OpenSSH -D sidecar: {:?}",
                script_path
            );
            let mut c = Command::new("ssh");
            c.env("SSH_ASKPASS_REQUIRE", "force");
            c.env("SSH_ASKPASS", &script_path);
            c.env("DISPLAY", ":0");
            c.env("SYNAPSH_SSH_PASSWORD", password);
            c.env("LANG", "C");
            askpass_script = Some(script_path);
            c
        }
    } else {
        Command::new("ssh")
    };

    cmd.args(&args);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    let stderr_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&stderr_log)
        .map_err(|e| format!("打开 sidecar 日志文件失败: {e}"))?;
    cmd.stderr(stderr_file);

    let child = cmd.spawn().map_err(|e| {
        if let Some(path) = askpass_script.take() {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_file(&stderr_log);
        format!("启动 OpenSSH sidecar 失败: {e}")
    })?;
    let pid = child.id();

    Ok(ProxySidecar {
        pid,
        port: local_port,
        child,
        askpass_script,
        stderr_log: Some(stderr_log),
    })
}

fn ensure_openssh_sidecar(session_id: &str, info: &SessionConnectionInfo) -> Result<u16, String> {
    let mut guard = get_proxy_sidecars()
        .lock()
        .expect("proxy sidecar mutex poisoned");
    let desired = openssh_sidecar_pool_size();
    let pool = guard
        .entry(session_id.to_string())
        .or_insert_with(|| ProxySidecarPool {
            sidecars: Vec::new(),
            next_index: 0,
        });

    prune_stale_sidecars(pool);

    while pool.sidecars.len() > desired {
        if let Some(mut extra) = pool.sidecars.pop() {
            stop_proxy_sidecar(&mut extra);
        }
    }

    while pool.sidecars.len() < desired {
        let local_port = reserve_local_port()?;
        let process = spawn_openssh_sidecar(info, local_port)?;
        let slot = pool.sidecars.len();
        log::info!(
            "启动 OpenSSH -D sidecar: session={}, slot={}/{}, pid={}, port={}",
            session_id,
            slot + 1,
            desired,
            process.pid,
            process.port
        );
        pool.sidecars.push(process);
    }

    if pool.sidecars.is_empty() {
        return Err("OpenSSH sidecar 池为空，无法分配代理端口".to_string());
    }
    if pool.next_index >= pool.sidecars.len() {
        pool.next_index = 0;
    }

    let selected = pool.next_index;
    pool.next_index = (pool.next_index + 1) % pool.sidecars.len();
    Ok(pool.sidecars[selected].port)
}

// Tauri Commands
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        params.password.clone(),
        params.private_key.clone(),
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
    sessions.insert(session_id.clone(), Arc::new(Mutex::new(session)));
    drop(sessions);

    let session_connections = get_session_connections();
    let mut session_connections = session_connections.lock().await;
    session_connections.insert(
        session_id.clone(),
        SessionConnectionInfo {
            host: params.host,
            port: params.port,
            username: params.username,
            password: params.password,
            private_key: params.private_key,
        },
    );
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
    drop(sessions);

    let session_connections = get_session_connections();
    let mut session_connections = session_connections.lock().await;
    session_connections.remove(&session_id);
    drop(session_connections);

    close_proxy_sidecar_for_session(&session_id);
    close_browser_for_session(&session_id);

    Ok(())
}

#[tauri::command]
async fn browser_open(
    session_id: String,
    url: String,
    options: Option<BrowserLaunchOptions>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let browser_open_start = std::time::Instant::now();
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
    let is_new_window = profile_mode == "new";

    let session_connection = {
        let session_connections = get_session_connections();
        let session_connections = session_connections.lock().await;
        session_connections
            .get(&session_id)
            .cloned()
            .ok_or_else(|| "Session connection not found".to_string())?
    };

    let mut proxy_port = match ensure_openssh_sidecar(&session_id, &session_connection) {
        Ok(port) => port,
        Err(e) => {
            let message = format!("启动 OpenSSH -D sidecar 失败：{e}");
            emit_browser_proxy_error(&app, &session_id, host, port, message.clone());
            return Err(message);
        }
    };
    log::debug!(
        "OpenSSH -D sidecar 已就绪，prepare_step=ensure_sidecar, elapsed_ms={}",
        browser_open_start.elapsed().as_millis()
    );

    if let Err(e) =
        wait_sidecar_listener_ready(&session_id, proxy_port, std::time::Duration::from_secs(3))
            .await
    {
        let message = format!("OpenSSH -D sidecar 就绪失败：{e}");
        emit_browser_proxy_error(&app, &session_id, host, port, message.clone());
        return Err(message);
    }

    let mut preflight_ok = false;
    let mut last_preflight_error = String::new();
    let mut restarted_once = false;
    for attempt in 1..=4 {
        if let Some(reason) = take_exited_sidecar_message(&session_id, proxy_port) {
            last_preflight_error = reason;
            if !restarted_once {
                restarted_once = true;
                proxy_port = match ensure_openssh_sidecar(&session_id, &session_connection) {
                    Ok(new_port) => new_port,
                    Err(restart_err) => {
                        let message = format!("OpenSSH -D sidecar 重建失败：{restart_err}");
                        emit_browser_proxy_error(&app, &session_id, host, port, message.clone());
                        return Err(message);
                    }
                };
                if let Err(wait_err) = wait_sidecar_listener_ready(
                    &session_id,
                    proxy_port,
                    std::time::Duration::from_secs(3),
                )
                .await
                {
                    let message = format!("OpenSSH -D sidecar 重建后就绪失败：{wait_err}");
                    emit_browser_proxy_error(&app, &session_id, host, port, message.clone());
                    return Err(message);
                }
                continue;
            }
            break;
        }

        match verify_socks5_proxy_connectivity(proxy_port).await {
            Ok(()) => {
                preflight_ok = true;
                break;
            }
            Err(e) => {
                last_preflight_error = e;
                log::warn!(
                    "OpenSSH -D 代理预检失败（第 {attempt} 次）: {}",
                    last_preflight_error
                );
                if attempt == 3 && !restarted_once {
                    restarted_once = true;
                    close_proxy_sidecar_for_session(&session_id);
                    proxy_port = match ensure_openssh_sidecar(&session_id, &session_connection) {
                        Ok(new_port) => new_port,
                        Err(restart_err) => {
                            let message = format!("OpenSSH -D sidecar 重建失败：{restart_err}");
                            emit_browser_proxy_error(
                                &app,
                                &session_id,
                                host,
                                port,
                                message.clone(),
                            );
                            return Err(message);
                        }
                    };
                    if let Err(wait_err) = wait_sidecar_listener_ready(
                        &session_id,
                        proxy_port,
                        std::time::Duration::from_secs(3),
                    )
                    .await
                    {
                        let message = format!("OpenSSH -D sidecar 重建后就绪失败：{wait_err}");
                        emit_browser_proxy_error(&app, &session_id, host, port, message.clone());
                        return Err(message);
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(120)).await;
            }
        }
    }

    if !preflight_ok {
        let message = format!("OpenSSH -D 代理预检失败：{last_preflight_error}");
        emit_browser_proxy_error(&app, &session_id, host, port, message.clone());
        return Err(message);
    }
    log::info!(
        "浏览器代理准备完成: session={}, elapsed_ms={}",
        session_id,
        browser_open_start.elapsed().as_millis()
    );

    let profile_dir = chrome_profile_dir(&session_id, profile_mode)?;
    let track_singleton = profile_mode == "session";
    let reuse_existing = if track_singleton {
        let mut processes = get_browser_processes()
            .lock()
            .expect("browser process mutex poisoned");
        let alive = processes
            .get_mut(&session_id)
            .map(is_browser_running)
            .unwrap_or(false);
        if !alive {
            processes.remove(&session_id);
        }
        alive
    } else {
        false
    };

    log::info!("启动 Chrome，代理端口: {}, URL: {}", proxy_port, target_url);

    #[cfg(target_os = "macos")]
    {
        let chrome_path = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
        let mut cmd = Command::new(chrome_path);

        cmd.arg(format!("--proxy-server=socks5://127.0.0.1:{proxy_port}"));
        cmd.arg(format!("--user-data-dir={}", profile_dir.display()));
        cmd.arg("--disable-quic");
        cmd.arg("--disable-features=VizDisplayCompositor,OptimizationGuideModelDownloading,OptimizationHintsFetching,AutofillServerCommunication,MediaRouter");
        cmd.arg("--disable-background-networking");
        cmd.arg("--disable-default-apps");
        cmd.arg("--disable-component-update");
        cmd.arg("--disable-domain-reliability");
        cmd.arg("--disable-client-side-phishing-detection");
        cmd.arg("--safebrowsing-disable-auto-update");
        cmd.arg("--disable-extensions");
        cmd.arg("--disable-sync");
        cmd.arg("--disable-translate");
        cmd.arg("--no-first-run");
        cmd.arg("--no-default-browser-check");
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        if is_new_window {
            cmd.arg("--new-window");
        }
        cmd.arg(target_url.as_str());

        log::info!(
            "启动 Chrome，代理端口: {}, URL: {}, 模式: {}",
            proxy_port,
            target_url,
            if is_new_window {
                "新建窗口"
            } else {
                "复用"
            }
        );
        log::debug!("执行命令: {:?}", cmd);

        let child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                let message = format!("执行 Chrome 失败: {}", e);
                emit_browser_proxy_error(&app, &session_id, host, port, message.clone());
                return Err(message);
            }
        };
        log::info!(
            "Chrome 启动命令完成: session={}, elapsed_ms={}",
            session_id,
            browser_open_start.elapsed().as_millis()
        );

        if track_singleton && !reuse_existing {
            let pid = child.id();
            let mut processes = get_browser_processes()
                .lock()
                .expect("browser process mutex poisoned");
            processes.insert(session_id.clone(), BrowserProcess { pid, child });
            log::info!("记录浏览器单实例进程: session={}, pid={}", session_id, pid);
        } else {
            // 复用场景下，当前启动命令仅用于把 URL 发送给已存在实例。
            drop(child);
        }

        if track_singleton && reuse_existing {
            log::debug!("复用浏览器单实例: session={}", session_id);
        }

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let message = "当前仅实现 macOS 的 Chrome 启动方案".to_string();
        emit_browser_proxy_error(&app, &session_id, host, port, message.clone());
        Err(message)
    }
}

#[tauri::command]
async fn browser_get_proxy_port(session_id: String) -> Result<Option<u16>, String> {
    let mut sidecars = get_proxy_sidecars()
        .lock()
        .map_err(|_| "proxy sidecar mutex poisoned".to_string())?;

    let mut remove_pool = false;
    let mut port = None;
    if let Some(pool) = sidecars.get_mut(&session_id) {
        prune_stale_sidecars(pool);
        if let Some(process) = pool.sidecars.first() {
            port = Some(process.port);
        } else {
            remove_pool = true;
        }
    }

    if remove_pool {
        sidecars.remove(&session_id);
    }

    Ok(port)
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

        let sections: Vec<&str> = output.split("@@@SECTION:").collect();

        let mut stats = SystemStats {
            cpu_percent: 0.0,
            memory: monitor::MemoryInfo {
                total: 0,
                used: 0,
                free: 0,
                cached: 0,
            },
            disks: Vec::new(),
            network: monitor::NetworkInfo {
                rx_bytes: 0,
                tx_bytes: 0,
            },
            processes: Vec::new(),
            system: monitor::SystemInfo {
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

            if section.starts_with("CPU@@@") || section.starts_with("CPU") {
                stats.cpu_percent = monitor::parse_cpu(section);
            } else if section.starts_with("MEM@@@") || section.starts_with("MEM") {
                stats.memory = monitor::parse_memory(section);
                stats.system.total_memory = stats.memory.total;
            } else if section.starts_with("DISK@@@") || section.starts_with("DISK") {
                stats.disks = monitor::parse_disks(section);
            } else if section.starts_with("NET@@@") || section.starts_with("NET") {
                let mut rx = 0;
                let mut tx = 0;
                for line in section.lines() {
                    if line.contains(":") {
                        let _parts: Vec<&str> = line.split_whitespace().collect();
                        // Logic from original code
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
                stats.network = monitor::NetworkInfo {
                    rx_bytes: rx,
                    tx_bytes: tx,
                };
            } else if section.starts_with("PROC@@@") || section.starts_with("PROC") {
                let content = section
                    .strip_prefix("PROC@@@")
                    .unwrap_or(section.strip_prefix("PROC").unwrap_or(section))
                    .trim();
                let lines: Vec<&str> = content.lines().collect();

                for line in lines.iter().skip(1) {
                    if line.trim().is_empty() {
                        continue;
                    }
                    let parts: Vec<&str> = line.split('|').collect();

                    if parts.len() >= 10 {
                        let pid_str = parts[0].trim();
                        if pid_str == "PID" || pid_str.parse::<u32>().is_err() {
                            continue;
                        }

                        let status = parts[6].trim().to_string();
                        let status_desc = monitor::get_status_description(&status);

                        stats.processes.push(ProcessInfo {
                            pid: pid_str.parse().unwrap_or(0u32),
                            user: parts[1].trim().to_string(),
                            cpu: parts[2].trim().parse().unwrap_or(0.0),
                            memory: parts[3].trim().parse().unwrap_or(0.0),
                            name: monitor::extract_process_name(parts[9]),
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
            } else if section.starts_with("SYS@@@") || section.starts_with("SYS") {
                let content = section
                    .strip_prefix("SYS@@@")
                    .unwrap_or(section.strip_prefix("SYS").unwrap_or(section))
                    .trim();
                let lines: Vec<&str> = content.lines().collect();

                if !lines.is_empty() && !lines[0].is_empty() {
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
        let contents = base64_decode(&base64_content)?;
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

// Helper removed (using imported utils::base64_decode)

// ==================== 数据库管理 Commands ====================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetectDatabasesParams {
    session_id: String,
}

#[tauri::command]
async fn detect_databases(
    params: DetectDatabasesParams,
) -> Result<Vec<DatabaseDetectionResult>, String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    let session_arc = sessions
        .get(&params.session_id)
        .cloned()
        .ok_or("Session not found")?;

    let session = session_arc.lock().await;
    let executor = SessionExecutor(&session);

    DatabaseManager::detect_all_databases(&executor)
        .await
        .map_err(|e| e.to_string())
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
    let executor = SessionExecutor(&session);

    DatabaseManager::install_database(&executor, &params.db_type, &params.options)
        .await
        .map_err(|e| e.to_string())
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
    let executor = SessionExecutor(&session);

    let result = match params.action.as_str() {
        "start" => DatabaseManager::start_service(&executor, &params.service_name).await,
        "stop" => DatabaseManager::stop_service(&executor, &params.service_name).await,
        "restart" => DatabaseManager::restart_service(&executor, &params.service_name).await,
        "enable" => DatabaseManager::enable_service(&executor, &params.service_name).await,
        _ => return Err(format!("Unknown action: {}", params.action)),
    };

    result.map_err(|e| e.to_string())
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
    let backup_cmd = format!(
        "sudo cp {} {}.backup.$(date +%Y%m%d_%H%M%S) 2>&1",
        config_path, config_path
    );
    let _ = session.exec_command(&backup_cmd).await;

    // 写入新配置 (使用 base64 避免转义问题)
    let base64_content = base64_encode(params.config_content.as_bytes());
    let cmd = format!(
        "echo '{}' | base64 -d | sudo tee {} > /dev/null 2>&1",
        base64_content, config_path
    );

    session.exec_command(&cmd).await.map_err(|e| e.to_string())
}

// ==================== 数据库实例管理 Commands ====================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetDatabasesParams {
    session_id: String,
    db_type: DatabaseType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseInfo {
    id: String,
    name: String,
    username: String,
    password: String,
    backup_count: i32,
    location: String,
    comment: String,
    status: String,
    created_at: String,
}

#[tauri::command]
async fn get_databases(params: GetDatabasesParams) -> Result<Vec<DatabaseInfo>, String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    let session_arc = sessions
        .get(&params.session_id)
        .cloned()
        .ok_or("Session not found")?;

    let session = session_arc.lock().await;

    // 根据数据库类型执行不同的查询命令
    let cmd = match params.db_type {
        DatabaseType::MySQL | DatabaseType::MariaDB => {
            // 查询所有数据库（排除系统数据库）
            r#"mysql -u root -e "SELECT schema_name FROM information_schema.schemata WHERE schema_name NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');" 2>/dev/null || echo "NO_MYSQL""#.to_string()
        }
        DatabaseType::PostgreSQL => {
            r#"sudo -u postgres psql -c "SELECT datname FROM pg_database WHERE datistemplate = false AND datname NOT IN ('postgres');" 2>/dev/null | grep -v "^\s*$" | grep -v "datname" | grep -v "row" | sed 's/^[[:space:]]*//' || echo "NO_PGSQL""#.to_string()
        }
        DatabaseType::Redis => {
            // Redis 没有数据库用户概念，返回实例信息
            r#"redis-cli INFO server 2>/dev/null | grep -E "redis_version|tcp_port" | tr '\n' ' ' || echo "NO_REDIS""#.to_string()
        }
        DatabaseType::MongoDB => {
            r#"mongosh --quiet --eval "db.adminCommand('listDatabases').databases.map(d => d.name).join('\n')" 2>/dev/null || mongo --quiet --eval "db.adminCommand('listDatabases').databases.map(d => d.name).join('\n')" 2>/dev/null || echo "NO_MONGODB""#.to_string()
        }
        _ => return Err(format!("暂不支持 {:?} 的数据库列表查询", params.db_type)),
    };

    let output = session
        .exec_command(&cmd)
        .await
        .map_err(|e| e.to_string())?;

    if output.contains("NO_MYSQL")
        || output.contains("NO_PGSQL")
        || output.contains("NO_REDIS")
        || output.contains("NO_MONGODB")
    {
        return Ok(vec![]);
    }

    let mut databases = Vec::new();

    match params.db_type {
        DatabaseType::MySQL | DatabaseType::MariaDB => {
            // 解析 MySQL 输出: 每行一个数据库名
            for line in output.lines() {
                let dbname = line.trim();
                if !dbname.is_empty() && !dbname.starts_with("schema_name") && !dbname.contains("-")
                {
                    // 查询该数据库的用户（如果有）
                    let user_cmd = format!(
                        r#"mysql -u root -e "SELECT user, host FROM mysql.db WHERE db='{}' AND user NOT IN ('root', 'mysql.infoschema', 'mysql.session', 'mysql.sys') LIMIT 1;" 2>/dev/null | tail -1"#,
                        dbname
                    );
                    let user_output = session.exec_command(&user_cmd).await.unwrap_or_default();
                    let (username, host) = if user_output.contains('\t') {
                        let parts: Vec<&str> = user_output.split('\t').collect();
                        (
                            parts[0].trim().to_string(),
                            parts
                                .get(1)
                                .map(|s| s.trim().to_string())
                                .unwrap_or_else(|| "localhost".to_string()),
                        )
                    } else {
                        ("root".to_string(), "localhost".to_string())
                    };

                    databases.push(DatabaseInfo {
                        id: dbname.to_string(),
                        name: dbname.to_string(),
                        username,
                        password: "**********".to_string(),
                        backup_count: 0,
                        location: if host == "localhost" {
                            "本地数据库".to_string()
                        } else {
                            host
                        },
                        comment: dbname.to_string(),
                        status: "running".to_string(),
                        created_at: "".to_string(),
                    });
                }
            }
        }
        DatabaseType::PostgreSQL => {
            // 解析 PostgreSQL 输出
            for line in output.lines() {
                let dbname = line.trim();
                if !dbname.is_empty() && !dbname.contains("---") && !dbname.contains("(") {
                    databases.push(DatabaseInfo {
                        id: dbname.to_string(),
                        name: dbname.to_string(),
                        username: "postgres".to_string(),
                        password: "**********".to_string(),
                        backup_count: 0,
                        location: "本地数据库".to_string(),
                        comment: dbname.to_string(),
                        status: "running".to_string(),
                        created_at: "".to_string(),
                    });
                }
            }
        }
        DatabaseType::MongoDB => {
            // 解析 MongoDB 输出
            for line in output.lines() {
                let dbname = line.trim();
                if !dbname.is_empty()
                    && dbname != "admin"
                    && dbname != "local"
                    && dbname != "config"
                {
                    databases.push(DatabaseInfo {
                        id: dbname.to_string(),
                        name: dbname.to_string(),
                        username: "".to_string(),
                        password: "**********".to_string(),
                        backup_count: 0,
                        location: "本地数据库".to_string(),
                        comment: dbname.to_string(),
                        status: "running".to_string(),
                        created_at: "".to_string(),
                    });
                }
            }
        }
        DatabaseType::Redis => {
            // Redis 只有一个实例，显示为一条记录
            let version = output
                .lines()
                .find(|l| l.contains("redis_version"))
                .and_then(|l| l.split(':').nth(1))
                .unwrap_or("unknown");
            databases.push(DatabaseInfo {
                id: "redis".to_string(),
                name: format!("Redis {}", version),
                username: "".to_string(),
                password: "**********".to_string(),
                backup_count: 0,
                location: "本地数据库".to_string(),
                comment: "Redis 实例".to_string(),
                status: "running".to_string(),
                created_at: "".to_string(),
            });
        }
        _ => {}
    }

    Ok(databases)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateDatabaseParams {
    session_id: String,
    db_type: DatabaseType,
    name: String,
    username: String,
    password: String,
    comment: Option<String>,
    access: Option<String>,
    charset: Option<String>,
}

#[tauri::command]
async fn create_database(params: CreateDatabaseParams) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    let session_arc = sessions
        .get(&params.session_id)
        .cloned()
        .ok_or("Session not found")?;

    let session = session_arc.lock().await;

    let access = params.access.unwrap_or_else(|| "localhost".to_string());
    let charset = params.charset.unwrap_or_else(|| "utf8mb4".to_string());

    let cmd = match params.db_type {
        DatabaseType::MySQL | DatabaseType::MariaDB => {
            format!(
                r#"mysql -u root -e "CREATE DATABASE IF NOT EXISTS \`{}\` CHARACTER SET {} COLLATE {}_unicode_ci; CREATE USER IF NOT EXISTS '{}'@'{}' IDENTIFIED BY '{}'; GRANT ALL PRIVILEGES ON \`{}\`.* TO '{}'@'{}'; FLUSH PRIVILEGES;" 2>&1"#,
                params.name,
                charset,
                charset,
                params.username,
                access,
                params.password,
                params.name,
                params.username,
                access
            )
        }
        DatabaseType::PostgreSQL => {
            let comment = params.comment.unwrap_or_default();
            format!(
                r#"sudo -u postgres psql -c "CREATE DATABASE \"{}\";" 2>&1 && sudo -u postgres psql -c "CREATE USER \"{}\" WITH PASSWORD '{}';" 2>&1 && sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE \"{}\" TO \"{}\";" 2>&1 && sudo -u postgres psql -d \"{}\" -c \"COMMENT ON DATABASE \"{}\" IS '{}';\" 2>&1"#,
                params.name,
                params.username,
                params.password,
                params.name,
                params.username,
                params.name,
                params.name,
                comment
            )
        }
        DatabaseType::MongoDB => {
            format!(
                r#"mongosh --quiet --eval "db.getSiblingDB('{}').createCollection('init')" 2>/dev/null || mongo --quiet --eval "db.getSiblingDB('{}').createCollection('init')" 2>&1"#,
                params.name, params.name
            )
        }
        DatabaseType::Redis => {
            return Err("Redis 不支持创建独立数据库，请使用 SELECT 命令切换 DB".to_string());
        }
        DatabaseType::SqlServer => {
            return Err("SQLServer support is not yet implemented".to_string());
        }
        DatabaseType::Sqlite => {
            // SQLite 创建数据库就是创建一个空文件
            let cmd = format!(
                r#"sqlite3 /var/lib/sqlite/{}.db "VACUUM;" 2>&1 || mkdir -p /var/lib/sqlite && sqlite3 /var/lib/sqlite/{}.db "VACUUM;" 2>&1"#,
                params.name, params.name
            );
            let output = session
                .exec_command(&cmd)
                .await
                .map_err(|e| e.to_string())?;
            if output.to_lowercase().contains("error") {
                return Err(format!("创建数据库失败: {}", output));
            }
            return Ok(());
        }
        _ => return Err(format!("暂不支持创建 {:?} 数据库", params.db_type)),
    };

    let output = session
        .exec_command(&cmd)
        .await
        .map_err(|e| e.to_string())?;

    if output.to_lowercase().contains("error") || output.to_lowercase().contains("failed") {
        return Err(format!("创建数据库失败: {}", output));
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChangePasswordParams {
    session_id: String,
    db_type: DatabaseType,
    db_id: String,
    username: String,
    new_password: String,
}

#[tauri::command]
async fn change_database_password(params: ChangePasswordParams) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    let session_arc = sessions
        .get(&params.session_id)
        .cloned()
        .ok_or("Session not found")?;

    let session = session_arc.lock().await;

    let cmd = match params.db_type {
        DatabaseType::MySQL | DatabaseType::MariaDB => {
            // 检查是否是管理员账号修改
            let is_admin = params.username == "root";
            if is_admin {
                // 修改 root 密码
                format!(
                    r#"mysql -u root -e "ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY '{}'; ALTER USER 'root'@'%' IDENTIFIED WITH mysql_native_password BY '{}'; FLUSH PRIVILEGES;" 2>&1 || mysql -u root -e "SET PASSWORD FOR 'root'@'localhost' = PASSWORD('{}'); SET PASSWORD FOR 'root'@'%' = PASSWORD('{}'); FLUSH PRIVILEGES;" 2>&1 || echo "MySQL_PASSWORD_CHANGE_FAILED""#,
                    params.new_password,
                    params.new_password,
                    params.new_password,
                    params.new_password
                )
            } else {
                // 修改普通用户密码
                format!(
                    r#"mysql -u root -e "ALTER USER '{}'@'localhost' IDENTIFIED BY '{}'; FLUSH PRIVILEGES;" 2>&1 || mysql -u root -e "SET PASSWORD FOR '{}'@'localhost' = PASSWORD('{}'); FLUSH PRIVILEGES;" 2>&1"#,
                    params.username, params.new_password, params.username, params.new_password
                )
            }
        }
        DatabaseType::PostgreSQL => {
            format!(
                r#"sudo -u postgres psql -c "ALTER USER \"{}\" WITH PASSWORD '{}';" 2>&1"#,
                params.username, params.new_password
            )
        }
        DatabaseType::MongoDB => {
            // MongoDB 修改管理员密码或普通用户密码
            if params.username == "admin" {
                format!(
                    r#"mongosh admin --quiet --eval "db.changeUserPassword('admin', '{}')" 2>/dev/null || mongo admin --quiet --eval "db.changeUserPassword('admin', '{}')" 2>&1 || mongosh admin --quiet --eval "db.createUser({{user: 'admin', pwd: '{}', roles: [{{role: 'userAdminAnyDatabase', db: 'admin'}}, {{role: 'readWriteAnyDatabase', db: 'admin'}}]}})" 2>/dev/null || mongo admin --quiet --eval "db.createUser({{user: 'admin', pwd: '{}', roles: [{{role: 'userAdminAnyDatabase', db: 'admin'}}, {{role: 'readWriteAnyDatabase', db: 'admin'}}]}})" 2>&1"#,
                    params.new_password,
                    params.new_password,
                    params.new_password,
                    params.new_password
                )
            } else {
                format!(
                    r#"mongosh --quiet --eval "db.getSiblingDB('{}').changeUserPassword('{}', '{}')" 2>/dev/null || mongo --quiet --eval "db.getSiblingDB('{}').changeUserPassword('{}', '{}')" 2>&1"#,
                    params.db_id,
                    params.username,
                    params.new_password,
                    params.db_id,
                    params.username,
                    params.new_password
                )
            }
        }
        DatabaseType::Redis => {
            // Redis 修改密码需要修改配置文件
            format!(
                r#"sudo sed -i 's/^#*requirepass .*/requirepass {}/' /etc/redis/redis.conf 2>/dev/null || sudo sed -i 's/^#*requirepass .*/requirepass {}/' /etc/redis.conf 2>/dev/null; sudo systemctl restart redis-server 2>&1 || sudo service redis-server restart 2>&1 || sudo systemctl restart redis 2>&1 || echo "RESTART_MANUALLY""#,
                params.new_password, params.new_password
            )
        }
        _ => return Err(format!("暂不支持修改 {:?} 数据库密码", params.db_type)),
    };

    let output = session
        .exec_command(&cmd)
        .await
        .map_err(|e| e.to_string())?;

    if output.to_lowercase().contains("error") && !output.to_lowercase().contains("warning") {
        return Err(format!("修改密码失败: {}", output));
    }

    if output.contains("MySQL_PASSWORD_CHANGE_FAILED") {
        return Err("MySQL 密码修改失败，请检查 root 是否有密码保护或使用 sudo".to_string());
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateDatabaseParams {
    session_id: String,
    db_type: DatabaseType,
    db_id: String,
    comment: String,
}

#[tauri::command]
async fn update_database(params: UpdateDatabaseParams) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    let session_arc = sessions
        .get(&params.session_id)
        .cloned()
        .ok_or("Session not found")?;

    let session = session_arc.lock().await;

    let cmd = match params.db_type {
        DatabaseType::PostgreSQL => {
            format!(
                r#"sudo -u postgres psql -c \"COMMENT ON DATABASE \"{}\" IS '{}';\" 2>&1"#,
                params.db_id,
                params.comment.replace("'", "''")
            )
        }
        DatabaseType::MySQL | DatabaseType::MariaDB => {
            // MySQL 没有标准的数据库备注功能，可以存储在自定义表或忽略
            return Ok(());
        }
        _ => return Ok(()),
    };

    let output = session
        .exec_command(&cmd)
        .await
        .map_err(|e| e.to_string())?;

    if output.to_lowercase().contains("error") {
        return Err(format!("更新数据库失败: {}", output));
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteDatabaseParams {
    session_id: String,
    db_type: DatabaseType,
    db_id: String,
    username: String,
}

#[tauri::command]
async fn delete_database(params: DeleteDatabaseParams) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    let session_arc = sessions
        .get(&params.session_id)
        .cloned()
        .ok_or("Session not found")?;

    let session = session_arc.lock().await;

    let cmd = match params.db_type {
        DatabaseType::MySQL | DatabaseType::MariaDB => {
            // 从 db_id 中提取数据库名
            let parts: Vec<&str> = params.db_id.splitn(2, '_').collect();
            let db_name = if parts.len() > 1 {
                parts[1]
            } else {
                &params.db_id
            };
            format!(
                r#"mysql -u root -e "DROP DATABASE IF EXISTS \`{}\`; DROP USER IF EXISTS '{}'@'localhost'; DROP USER IF EXISTS '{}'@'%'; FLUSH PRIVILEGES;" 2>&1"#,
                db_name, params.username, params.username
            )
        }
        DatabaseType::PostgreSQL => {
            format!(
                r#"sudo -u postgres psql -c "DROP DATABASE IF EXISTS \"{}\";" 2>&1 && sudo -u postgres psql -c "DROP USER IF EXISTS \"{}\";" 2>&1"#,
                params.db_id, params.username
            )
        }
        DatabaseType::MongoDB => {
            format!(
                r#"mongosh --quiet --eval "db.getSiblingDB('{}').dropDatabase()" 2>/dev/null || mongo --quiet --eval "db.getSiblingDB('{}').dropDatabase()" 2>&1"#,
                params.db_id, params.db_id
            )
        }
        _ => return Err(format!("暂不支持删除 {:?} 数据库", params.db_type)),
    };

    let output = session
        .exec_command(&cmd)
        .await
        .map_err(|e| e.to_string())?;

    if output.to_lowercase().contains("error") {
        return Err(format!("删除数据库失败: {}", output));
    }

    Ok(())
}

// ==================== 数据库 Schema 管理 Commands ====================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetDatabaseSchemasParams {
    session_id: String,
    db_type: DatabaseType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DbSchema {
    id: String,
    name: String,
    username: String,
    password: String,
    backup_count: i32,
    location: String,
    comment: String,
    created_at: String,
}

#[tauri::command]
async fn get_database_schemas(params: GetDatabaseSchemasParams) -> Result<Vec<DbSchema>, String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    let session_arc = sessions
        .get(&params.session_id)
        .cloned()
        .ok_or("Session not found")?;

    let session = session_arc.lock().await;

    let cmd = match params.db_type {
        DatabaseType::MySQL | DatabaseType::MariaDB => {
            // 查询所有数据库及其用户
            r#"mysql -u root -e "SELECT DISTINCT db.db, db.user, db.host FROM mysql.db db WHERE db.db NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys') AND db.user NOT IN ('root', 'mysql.infoschema', 'mysql.session', 'mysql.sys') ORDER BY db.db;" 2>/dev/null || echo "NO_DATA""#.to_string()
        }
        DatabaseType::PostgreSQL => {
            r#"sudo -u postgres psql -c "SELECT datname FROM pg_database WHERE datistemplate = false AND datname NOT IN ('postgres');" 2>/dev/null | grep -v "^\s*$" | grep -v "datname" | grep -v "row" | sed 's/^[[:space:]]*//' || echo "NO_DATA""#.to_string()
        }
        DatabaseType::MongoDB => {
            r#"mongosh --quiet --eval "db.adminCommand('listDatabases').databases.filter(d => !['admin','local','config'].includes(d.name)).map(d => d.name).join('\n')" 2>/dev/null || mongo --quiet --eval "db.adminCommand('listDatabases').databases.filter(d => !['admin','local','config'].includes(d.name)).map(d => d.name).join('\n')" 2>/dev/null || echo "NO_DATA""#.to_string()
        }
        DatabaseType::Redis => {
            // Redis 没有数据库概念，返回空列表
            return Ok(vec![]);
        }
        DatabaseType::SqlServer => {
            return Err("SQLServer support is not yet implemented".to_string());
        }
        DatabaseType::Sqlite => {
            // SQLite 是文件型数据库，列出 .db 文件
            return Ok(vec![]);
        }
        _ => return Err(format!("暂不支持 {:?} 的数据库列表查询", params.db_type)),
    };

    let output = session
        .exec_command(&cmd)
        .await
        .map_err(|e| e.to_string())?;

    if output.contains("NO_DATA") || output.trim().is_empty() {
        return Ok(vec![]);
    }

    let mut schemas = Vec::new();

    match params.db_type {
        DatabaseType::MySQL | DatabaseType::MariaDB => {
            for line in output.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 3 {
                    let dbname = parts[0].trim();
                    let username = parts[1].trim();
                    let host = parts[2].trim();

                    schemas.push(DbSchema {
                        id: format!("{}_{}", username, dbname),
                        name: dbname.to_string(),
                        username: username.to_string(),
                        password: "**********".to_string(),
                        backup_count: 0,
                        location: if host == "localhost" {
                            "本地数据库".to_string()
                        } else {
                            host.to_string()
                        },
                        comment: dbname.to_string(),
                        created_at: "".to_string(),
                    });
                }
            }
        }
        DatabaseType::PostgreSQL => {
            for line in output.lines() {
                let dbname = line.trim();
                if !dbname.is_empty() && !dbname.contains("---") && !dbname.contains("(") {
                    schemas.push(DbSchema {
                        id: dbname.to_string(),
                        name: dbname.to_string(),
                        username: "postgres".to_string(),
                        password: "**********".to_string(),
                        backup_count: 0,
                        location: "本地数据库".to_string(),
                        comment: dbname.to_string(),
                        created_at: "".to_string(),
                    });
                }
            }
        }
        DatabaseType::MongoDB => {
            for line in output.lines() {
                let dbname = line.trim();
                if !dbname.is_empty() {
                    schemas.push(DbSchema {
                        id: dbname.to_string(),
                        name: dbname.to_string(),
                        username: "".to_string(),
                        password: "**********".to_string(),
                        backup_count: 0,
                        location: "本地数据库".to_string(),
                        comment: dbname.to_string(),
                        created_at: "".to_string(),
                    });
                }
            }
        }
        _ => {}
    }

    Ok(schemas)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateDatabaseSchemaParams {
    session_id: String,
    db_type: DatabaseType,
    name: String,
    username: String,
    password: String,
    comment: Option<String>,
    access: Option<String>,
    charset: Option<String>,
}

#[tauri::command]
async fn create_database_schema(params: CreateDatabaseSchemaParams) -> Result<(), String> {
    let sessions = get_sessions();
    let sessions = sessions.lock().await;

    let session_arc = sessions
        .get(&params.session_id)
        .cloned()
        .ok_or("Session not found")?;

    let session = session_arc.lock().await;

    let access = params.access.unwrap_or_else(|| "localhost".to_string());
    let charset = params.charset.unwrap_or_else(|| "utf8mb4".to_string());

    let cmd = match params.db_type {
        DatabaseType::MySQL | DatabaseType::MariaDB => {
            format!(
                r#"mysql -u root -e "CREATE DATABASE IF NOT EXISTS \`{}\` CHARACTER SET {} COLLATE {}_unicode_ci; CREATE USER IF NOT EXISTS '{}'@'{}' IDENTIFIED BY '{}'; GRANT ALL PRIVILEGES ON \`{0}\`.* TO '{}'@'{1}'; FLUSH PRIVILEGES;" 2>&1"#,
                params.name,
                charset,
                charset,
                params.username,
                access,
                params.password,
                params.username
            )
        }
        DatabaseType::PostgreSQL => {
            format!(
                r#"sudo -u postgres psql -c "CREATE DATABASE \"{}\";" 2>&1 && sudo -u postgres psql -c "CREATE USER \"{}\" WITH PASSWORD '{}';" 2>&1 && sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE \"{}\" TO \"{}\";" 2>&1"#,
                params.name, params.username, params.password, params.name, params.username
            )
        }
        DatabaseType::MongoDB => {
            // MongoDB 创建数据库就是创建一个集合并插入数据
            format!(
                r#"mongosh --quiet --eval "db.getSiblingDB('{}').createCollection('__init__')" 2>/dev/null || mongo --quiet --eval "db.getSiblingDB('{}').createCollection('__init__')" 2>&1"#,
                params.name, params.name
            )
        }
        _ => return Err(format!("暂不支持创建 {:?} 数据库", params.db_type)),
    };

    let output = session
        .exec_command(&cmd)
        .await
        .map_err(|e| e.to_string())?;

    if output.to_lowercase().contains("error") || output.to_lowercase().contains("failed") {
        return Err(format!("创建数据库失败: {}", output));
    }

    Ok(())
}

// 需要在 run() 中注册新命令
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_frame::FramePluginBuilder::new()
                .titlebar_height(40)
                .button_width(46)
                .auto_titlebar(false)
                .snap_overlay_delay_ms(15)
                .close_hover_bg("rgba(239,68,68,1)")
                .button_hover_bg("rgba(255,255,255,0.1)")
                .build(),
        )
        .setup(|_app| {
            // 手动为主窗口创建覆盖标题栏
            #[cfg(windows)]
            {
                use tauri::Manager;
                use tauri_plugin_frame::WebviewWindowExt;

                let app_handle = _app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    // 延迟一点确保窗口完全创建
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                    if let Some(window) = app_handle.get_webview_window("main") {
                        let result = window.create_overlay_titlebar();
                        match result {
                            Ok(_) => {
                                log::info!("[Frame Plugin] Overlay titlebar created successfully")
                            }
                            Err(e) => log::error!(
                                "[Frame Plugin] Failed to create overlay titlebar: {}",
                                e
                            ),
                        }
                    } else {
                        log::warn!("[Frame Plugin] Main window not found");
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect_ssh,
            write_to_pty,
            resize_pty,
            disconnect_ssh,
            browser_open,
            browser_get_proxy_port,
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
            update_database_config,
            // 数据库实例管理命令
            get_databases,
            create_database,
            change_database_password,
            update_database,
            delete_database,
            get_database_schemas,
            create_database_schema
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::Exit | tauri::RunEvent::ExitRequested { .. } => {
            close_all_browser_processes();
            close_all_proxy_sidecars();
        }
        _ => {}
    });
}
