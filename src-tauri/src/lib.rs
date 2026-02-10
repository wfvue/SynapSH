use anyhow::Result;
use log::info;
use russh::Disconnect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, OnceLock};
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

// Implementations moved to ssh.rs

// Helper functions removed (moved to monitor.rs / ssh.rs / utils.rs)

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
        .ensure_socks5_proxy(app, session_id.clone())
        .await
        .map_err(|e| format!("启动 SOCKS5 代理失败：{e}"))?;
    drop(session);

    let proxy_arg = format!("--proxy-server=socks5://127.0.0.1:{proxy_port}");
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

    tauri::Builder::default()
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

                let app_handle = app.handle().clone();
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
