use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu: f64,
    pub memory: f64,
    pub user: String,
    // 进程状态
    pub status: String,       // 进程状态 (R/S/Z/D/T等)
    pub status_desc: String,  // 状态描述
    pub start_time: String,   // 启动时间
    pub elapsed_time: String, // 运行时长 (TIME字段)
    pub vsz: u64,             // 虚拟内存 (KB)
    pub rss: u64,             // 物理内存 (KB)
    pub command: String,      // 完整命令行
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub name: String,
    pub total: u64,
    pub used: u64,
    pub mount_point: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub cached: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInfo {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub hostname: String,
    pub uptime: String,
    pub load_average: [f64; 3],
    pub cpu_cores: u32,
    pub kernel_version: String,
    pub total_memory: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub cpu_percent: f64,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub network: NetworkInfo,
    pub processes: Vec<ProcessInfo>,
    pub system: SystemInfo,
}

// 辅助函数：解析内存信息
pub fn parse_memory(output: &str) -> MemoryInfo {
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
pub fn parse_disks(output: &str) -> Vec<DiskInfo> {
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
pub fn parse_cpu(output: &str) -> f64 {
    // Output from top -bn1 | grep "Cpu(s)"
    // %Cpu(s):  0.0 us,  0.0 sy,  0.0 ni, 100.0 id,  0.0 wa,  0.0 hi,  0.0 si,  0.0 st
    // We want 100 - id
    if let Some(line) = output.lines().find(|l| l.contains("Cpu(s)")) {
        if let Some(id_part) = line.split(",").find(|p| p.contains("id")) {
            let id_str = id_part.split_whitespace().next().unwrap_or("100");
            let id_val: f64 = id_str.parse().unwrap_or(100.0);
            return 100.0 - id_val;
        }
    }
    0.0
}

// 辅助函数：获取进程状态描述
pub fn get_status_description(status: &str) -> String {
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
pub fn extract_process_name(command: &str) -> String {
    // 提取命令名（去除路径和参数）
    command
        .split_whitespace()
        .next()
        .map(|s| s.split('/').next_back().unwrap_or(s).to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
