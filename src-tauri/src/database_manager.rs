//! 远程服务器数据库管理模块
//! 提供数据库检测、安装、配置、备份等功能

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

// ==================== 数据类型定义 ====================

/// 支持的数据库类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    MySQL,
    PostgreSQL,
    Redis,
    MongoDB,
    MariaDB,
    Elasticsearch,
    ClickHouse,
}

impl DatabaseType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseType::MySQL => "mysql",
            DatabaseType::PostgreSQL => "postgresql",
            DatabaseType::Redis => "redis",
            DatabaseType::MongoDB => "mongodb",
            DatabaseType::MariaDB => "mariadb",
            DatabaseType::Elasticsearch => "elasticsearch",
            DatabaseType::ClickHouse => "clickhouse",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DatabaseType::MySQL => "MySQL",
            DatabaseType::PostgreSQL => "PostgreSQL",
            DatabaseType::Redis => "Redis",
            DatabaseType::MongoDB => "MongoDB",
            DatabaseType::MariaDB => "MariaDB",
            DatabaseType::Elasticsearch => "Elasticsearch",
            DatabaseType::ClickHouse => "ClickHouse",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            DatabaseType::MySQL => 3306,
            DatabaseType::PostgreSQL => 5432,
            DatabaseType::Redis => 6379,
            DatabaseType::MongoDB => 27017,
            DatabaseType::MariaDB => 3306,
            DatabaseType::Elasticsearch => 9200,
            DatabaseType::ClickHouse => 8123,
        }
    }

    pub fn service_name(&self) -> &'static str {
        match self {
            DatabaseType::MySQL => "mysql",
            DatabaseType::PostgreSQL => "postgresql",
            DatabaseType::Redis => "redis-server",
            DatabaseType::MongoDB => "mongod",
            DatabaseType::MariaDB => "mariadb",
            DatabaseType::Elasticsearch => "elasticsearch",
            DatabaseType::ClickHouse => "clickhouse-server",
        }
    }
}

/// 数据库实例信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInstance {
    pub db_type: DatabaseType,
    pub version: Option<String>,
    pub status: DatabaseStatus,
    pub port: u16,
    pub install_path: Option<String>,
    pub data_path: Option<String>,
    pub config_path: Option<String>,
    pub service_name: String,
}

/// 数据库状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseStatus {
    Running,
    Stopped,
    Error,
    NotInstalled,
    Installing,
    Unknown,
}

/// 安装选项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallOptions {
    pub version: Option<String>,
    pub port: Option<u16>,
    pub root_password: Option<String>,
    pub data_path: Option<String>,
}

/// 数据库检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseDetectionResult {
    pub db_type: DatabaseType,
    pub installed: bool,
    pub version: Option<String>,
    pub status: DatabaseStatus,
    pub port: Option<u16>,
    pub install_path: Option<String>,
}

// ==================== 数据库管理器 ====================

pub struct DatabaseManager;

impl DatabaseManager {
    /// 检测所有支持的数据库
    pub async fn detect_all_databases(exec: &impl CommandExecutor) -> Result<Vec<DatabaseDetectionResult>> {
        let db_types = vec![
            DatabaseType::MySQL,
            DatabaseType::PostgreSQL,
            DatabaseType::Redis,
            DatabaseType::MongoDB,
            DatabaseType::MariaDB,
        ];

        let mut results = Vec::new();
        for db_type in db_types {
            match Self::detect_database(exec, &db_type).await {
                Ok(result) => results.push(result),
                Err(e) => log::warn!("检测 {:?} 失败: {}", db_type, e),
            }
        }

        Ok(results)
    }

    /// 检测单个数据库
    pub async fn detect_database(
        exec: &impl CommandExecutor,
        db_type: &DatabaseType,
    ) -> Result<DatabaseDetectionResult> {
        let (installed, version, status, port, install_path) = match db_type {
            DatabaseType::MySQL => Self::detect_mysql(exec).await?,
            DatabaseType::PostgreSQL => Self::detect_postgresql(exec).await?,
            DatabaseType::Redis => Self::detect_redis(exec).await?,
            DatabaseType::MongoDB => Self::detect_mongodb(exec).await?,
            DatabaseType::MariaDB => Self::detect_mariadb(exec).await?,
            _ => (false, None, DatabaseStatus::NotInstalled, None, None),
        };

        Ok(DatabaseDetectionResult {
            db_type: db_type.clone(),
            installed,
            version,
            status,
            port,
            install_path,
        })
    }

    // ==================== 各数据库检测逻辑 ====================

    async fn detect_mysql(exec: &impl CommandExecutor) -> Result<(bool, Option<String>, DatabaseStatus, Option<u16>, Option<String>)> {
        // 检查是否安装
        let version_output = exec.execute("mysql --version 2>/dev/null || echo 'NOT_FOUND'").await?;
        
        if version_output.contains("NOT_FOUND") {
            return Ok((false, None, DatabaseStatus::NotInstalled, None, None));
        }

        // 解析版本
        let version = Self::extract_version(&version_output, &["Ver", "version"]);

        // 检查服务状态
        let status = Self::check_service_status(exec, "mysql").await?;

        // 获取端口
        let port = Self::get_mysql_port(exec).await.ok();

        // 获取安装路径
        let install_path = exec.execute("which mysql 2>/dev/null || echo ''").await.ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string());

        Ok((true, version, status, port, install_path))
    }

    async fn detect_postgresql(exec: &impl CommandExecutor) -> Result<(bool, Option<String>, DatabaseStatus, Option<u16>, Option<String>)> {
        let version_output = exec.execute("psql --version 2>/dev/null || echo 'NOT_FOUND'").await?;
        
        if version_output.contains("NOT_FOUND") {
            return Ok((false, None, DatabaseStatus::NotInstalled, None, None));
        }

        let version = Self::extract_version(&version_output, &["psql", "PostgreSQL"]);
        let status = Self::check_service_status(exec, "postgresql").await?;
        let port = Self::get_postgresql_port(exec).await.ok();
        let install_path = exec.execute("which psql 2>/dev/null || echo ''").await.ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string());

        Ok((true, version, status, port, install_path))
    }

    async fn detect_redis(exec: &impl CommandExecutor) -> Result<(bool, Option<String>, DatabaseStatus, Option<u16>, Option<String>)> {
        let version_output = exec.execute("redis-server --version 2>/dev/null || echo 'NOT_FOUND'").await?;
        
        if version_output.contains("NOT_FOUND") {
            return Ok((false, None, DatabaseStatus::NotInstalled, None, None));
        }

        let version = Self::extract_version(&version_output, &["v=", "Redis server"]);
        let status = Self::check_service_status(exec, "redis-server").await?;
        let port = Self::get_redis_port(exec).await.ok();
        let install_path = exec.execute("which redis-server 2>/dev/null || echo ''").await.ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string());

        Ok((true, version, status, port, install_path))
    }

    async fn detect_mongodb(exec: &impl CommandExecutor) -> Result<(bool, Option<String>, DatabaseStatus, Option<u16>, Option<String>)> {
        let version_output = exec.execute("mongod --version 2>/dev/null || echo 'NOT_FOUND'").await?;
        
        if version_output.contains("NOT_FOUND") {
            return Ok((false, None, DatabaseStatus::NotInstalled, None, None));
        }

        let version = Self::extract_version(&version_output, &["db version", "v"]);
        let status = Self::check_service_status(exec, "mongod").await?;
        let port = Some(27017); // MongoDB 默认端口
        let install_path = exec.execute("which mongod 2>/dev/null || echo ''").await.ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string());

        Ok((true, version, status, port, install_path))
    }

    async fn detect_mariadb(exec: &impl CommandExecutor) -> Result<(bool, Option<String>, DatabaseStatus, Option<u16>, Option<String>)> {
        let version_output = exec.execute("mariadb --version 2>/dev/null || mysql --version 2>/dev/null || echo 'NOT_FOUND'").await?;
        
        if version_output.contains("NOT_FOUND") || !version_output.to_lowercase().contains("mariadb") {
            return Ok((false, None, DatabaseStatus::NotInstalled, None, None));
        }

        let version = Self::extract_version(&version_output, &["Ver", "version"]);
        let status = Self::check_service_status(exec, "mariadb").await?;
        let port = Self::get_mysql_port(exec).await.ok();
        let install_path = exec.execute("which mariadb 2>/dev/null || which mysql 2>/dev/null || echo ''").await.ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string());

        Ok((true, version, status, port, install_path))
    }

    // ==================== 安装数据库 ====================

    pub async fn install_database(
        exec: &impl CommandExecutor,
        db_type: &DatabaseType,
        options: &InstallOptions,
    ) -> Result<String> {
        // 检测操作系统类型
        let os_info = Self::detect_os(exec).await?;
        
        let install_script = match db_type {
            DatabaseType::MySQL => Self::generate_mysql_install_script(&os_info, options),
            DatabaseType::PostgreSQL => Self::generate_postgresql_install_script(&os_info, options),
            DatabaseType::Redis => Self::generate_redis_install_script(&os_info, options),
            DatabaseType::MongoDB => Self::generate_mongodb_install_script(&os_info, options),
            DatabaseType::MariaDB => Self::generate_mariadb_install_script(&os_info, options),
            _ => return Err(anyhow!("暂不支持安装 {:?}", db_type)),
        };

        // 执行安装脚本
        let output = exec.execute(&install_script).await?;
        
        Ok(output)
    }

    // ==================== 服务管理 ====================

    pub async fn start_service(exec: &impl CommandExecutor, service_name: &str) -> Result<String> {
        let cmd = format!("sudo systemctl start {} 2>&1 || sudo service {} start 2>&1", service_name, service_name);
        exec.execute(&cmd).await
    }

    pub async fn stop_service(exec: &impl CommandExecutor, service_name: &str) -> Result<String> {
        let cmd = format!("sudo systemctl stop {} 2>&1 || sudo service {} stop 2>&1", service_name, service_name);
        exec.execute(&cmd).await
    }

    pub async fn restart_service(exec: &impl CommandExecutor, service_name: &str) -> Result<String> {
        let cmd = format!("sudo systemctl restart {} 2>&1 || sudo service {} restart 2>&1", service_name, service_name);
        exec.execute(&cmd).await
    }

    pub async fn enable_service(exec: &impl CommandExecutor, service_name: &str) -> Result<String> {
        let cmd = format!("sudo systemctl enable {} 2>&1", service_name);
        exec.execute(&cmd).await
    }

    // ==================== 配置管理 ====================

    pub async fn get_config(exec: &impl CommandExecutor, db_type: &DatabaseType) -> Result<String> {
        let config_path = match db_type {
            DatabaseType::MySQL | DatabaseType::MariaDB => "/etc/mysql/my.cnf /etc/my.cnf ~/.my.cnf",
            DatabaseType::PostgreSQL => "/etc/postgresql/*/main/postgresql.conf",
            DatabaseType::Redis => "/etc/redis/redis.conf /etc/redis.conf",
            DatabaseType::MongoDB => "/etc/mongod.conf",
            _ => return Err(anyhow!("暂不支持获取 {:?} 配置", db_type)),
        };

        let cmd = format!("cat {} 2>/dev/null | head -500", config_path);
        exec.execute(&cmd).await
    }

    pub async fn update_config(
        exec: &impl CommandExecutor,
        db_type: &DatabaseType,
        config_content: &str,
    ) -> Result<String> {
        let config_path: String = match db_type {
            DatabaseType::MySQL | DatabaseType::MariaDB => "/etc/mysql/my.cnf".to_string(),
            DatabaseType::PostgreSQL => {
                // 需要动态获取版本号
                let version_dir = exec.execute("ls /etc/postgresql/ 2>/dev/null | head -1").await?;
                format!("/etc/postgresql/{}/main/postgresql.conf", version_dir.trim())
            }
            DatabaseType::Redis => "/etc/redis/redis.conf".to_string(),
            DatabaseType::MongoDB => "/etc/mongod.conf".to_string(),
            _ => return Err(anyhow!("暂不支持修改 {:?} 配置", db_type)),
        };

        // 备份原配置
        let backup_cmd = format!("sudo cp {} {}.backup.$(date +%Y%m%d_%H%M%S) 2>&1", config_path, config_path);
        let _ = exec.execute(&backup_cmd).await;

        // 写入新配置
        let escaped_content = config_content.replace("'", "'\"'\"'");
        let cmd = format!("echo '{}' | sudo tee {} > /dev/null 2>&1", escaped_content, config_path);
        exec.execute(&cmd).await
    }

    // ==================== 备份恢复 ====================

    pub async fn backup_database(
        exec: &impl CommandExecutor,
        db_type: &DatabaseType,
        database_name: &str,
        output_path: &str,
    ) -> Result<String> {
        let cmd = match db_type {
            DatabaseType::MySQL | DatabaseType::MariaDB => {
                format!("mysqldump -u root {} > {} 2>&1", database_name, output_path)
            }
            DatabaseType::PostgreSQL => {
                format!("pg_dump -U postgres {} > {} 2>&1", database_name, output_path)
            }
            DatabaseType::MongoDB => {
                format!("mongodump --db {} --out {} 2>&1", database_name, output_path)
            }
            DatabaseType::Redis => {
                format!("redis-cli SAVE && cp /var/lib/redis/dump.rdb {} 2>&1", output_path)
            }
            _ => return Err(anyhow!("暂不支持备份 {:?}", db_type)),
        };

        exec.execute(&cmd).await
    }

    pub async fn restore_database(
        exec: &impl CommandExecutor,
        db_type: &DatabaseType,
        database_name: &str,
        backup_path: &str,
    ) -> Result<String> {
        let cmd = match db_type {
            DatabaseType::MySQL | DatabaseType::MariaDB => {
                format!("mysql -u root {} < {} 2>&1", database_name, backup_path)
            }
            DatabaseType::PostgreSQL => {
                format!("psql -U postgres {} < {} 2>&1", database_name, backup_path)
            }
            DatabaseType::MongoDB => {
                format!("mongorestore --db {} {} 2>&1", database_name, backup_path)
            }
            _ => return Err(anyhow!("暂不支持恢复 {:?}", db_type)),
        };

        exec.execute(&cmd).await
    }

    // ==================== 辅助方法 ====================

    async fn check_service_status(exec: &impl CommandExecutor, service_name: &str) -> Result<DatabaseStatus> {
        let cmd = format!(
            "sudo systemctl is-active {} 2>/dev/null || sudo service {} status 2>&1 | grep -q running && echo 'active' || echo 'inactive'",
            service_name, service_name
        );
        
        match exec.execute(&cmd).await {
            Ok(output) => {
                if output.trim() == "active" {
                    Ok(DatabaseStatus::Running)
                } else {
                    Ok(DatabaseStatus::Stopped)
                }
            }
            Err(_) => Ok(DatabaseStatus::Unknown),
        }
    }

    async fn get_mysql_port(exec: &impl CommandExecutor) -> Result<u16> {
        let output = exec.execute("mysql -u root -e \"SHOW VARIABLES LIKE 'port';\" 2>/dev/null | tail -1 | awk '{print $2}'").await?;
        Ok(output.trim().parse().unwrap_or(3306))
    }

    async fn get_postgresql_port(exec: &impl CommandExecutor) -> Result<u16> {
        let output = exec.execute("sudo -u postgres psql -c \"SHOW port;\" 2>/dev/null | tail -3 | head -1 | tr -d ' '").await?;
        Ok(output.trim().parse().unwrap_or(5432))
    }

    async fn get_redis_port(exec: &impl CommandExecutor) -> Result<u16> {
        let output = exec.execute("redis-cli CONFIG GET port 2>/dev/null | tail -1").await?;
        Ok(output.trim().parse().unwrap_or(6379))
    }

    fn extract_version(output: &str, keywords: &[&str]) -> Option<String> {
        for line in output.lines() {
            for keyword in keywords {
                if let Some(pos) = line.find(keyword) {
                    let after = &line[pos + keyword.len()..];
                    // 提取版本号 (数字.数字.数字 格式)
                    let version_regex = regex::Regex::new(r"(\d+\.\d+(?:\.\d+)?)").ok()?;
                    if let Some(cap) = version_regex.captures(after) {
                        return Some(cap.get(1)?.as_str().to_string());
                    }
                }
            }
        }
        None
    }

    async fn detect_os(exec: &impl CommandExecutor) -> Result<OsInfo> {
        let os_release = exec.execute("cat /etc/os-release 2>/dev/null || echo 'ID=unknown'").await?;
        
        let mut id = "unknown".to_string();
        let mut version_id = None;
        let mut like = None;

        for line in os_release.lines() {
            if line.starts_with("ID=") {
                id = line.trim_start_matches("ID=").trim_matches('"').to_string();
            } else if line.starts_with("VERSION_ID=") {
                version_id = Some(line.trim_start_matches("VERSION_ID=").trim_matches('"').to_string());
            } else if line.starts_with("ID_LIKE=") {
                like = Some(line.trim_start_matches("ID_LIKE=").trim_matches('"').to_string());
            }
        }

        let package_manager = if id.contains("debian") || id.contains("ubuntu") || like.as_ref().map(|l| l.contains("debian")).unwrap_or(false) {
            PackageManager::Apt
        } else if id.contains("centos") || id.contains("rhel") || id.contains("fedora") || like.as_ref().map(|l| l.contains("rhel")).unwrap_or(false) {
            PackageManager::Yum
        } else if id.contains("alpine") {
            PackageManager::Apk
        } else {
            PackageManager::Unknown
        };

        Ok(OsInfo {
            id,
            version_id,
            package_manager,
        })
    }

    // ==================== 安装脚本生成 ====================

    fn generate_mysql_install_script(os: &OsInfo, options: &InstallOptions) -> String {
        let port = options.port.unwrap_or(3306);
        let password = options.root_password.as_deref().unwrap_or("");
        
        match os.package_manager {
            PackageManager::Apt => format!(
                r#"export DEBIAN_FRONTEND=noninteractive && \
sudo apt-get update && \
sudo apt-get install -y mysql-server && \
sudo systemctl start mysql && \
sudo systemctl enable mysql && \
sudo mysql -e "ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY '{}'; FLUSH PRIVILEGES;" && \
echo "MySQL installed successfully on port {}""#,
                password, port
            ),
            PackageManager::Yum => format!(
                r#"sudo yum install -y mysql-server && \
sudo systemctl start mysqld && \
sudo systemctl enable mysqld && \
echo "MySQL installed successfully on port {}""#,
                port
            ),
            _ => "echo 'Unsupported package manager'".to_string(),
        }
    }

    fn generate_postgresql_install_script(os: &OsInfo, options: &InstallOptions) -> String {
        let port = options.port.unwrap_or(5432);
        
        match os.package_manager {
            PackageManager::Apt => format!(
                r#"sudo apt-get update && \
sudo apt-get install -y postgresql postgresql-contrib && \
sudo systemctl start postgresql && \
sudo systemctl enable postgresql && \
sudo -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'postgres';" && \
echo "PostgreSQL installed successfully on port {}""#,
                port
            ),
            PackageManager::Yum => format!(
                r#"sudo yum install -y postgresql-server postgresql-contrib && \
sudo postgresql-setup initdb || true && \
sudo systemctl start postgresql && \
sudo systemctl enable postgresql && \
echo "PostgreSQL installed successfully on port {}""#,
                port
            ),
            _ => "echo 'Unsupported package manager'".to_string(),
        }
    }

    fn generate_redis_install_script(os: &OsInfo, options: &InstallOptions) -> String {
        let port = options.port.unwrap_or(6379);
        
        match os.package_manager {
            PackageManager::Apt => format!(
                r#"sudo apt-get update && \
sudo apt-get install -y redis-server && \
sudo sed -i 's/^#*port .*/port {}/' /etc/redis/redis.conf && \
sudo systemctl restart redis-server && \
sudo systemctl enable redis-server && \
echo "Redis installed successfully on port {}""#,
                port, port
            ),
            PackageManager::Yum => format!(
                r#"sudo yum install -y redis && \
sudo systemctl start redis && \
sudo systemctl enable redis && \
echo "Redis installed successfully on port {}""#,
                port
            ),
            _ => "echo 'Unsupported package manager'".to_string(),
        }
    }

    fn generate_mongodb_install_script(os: &OsInfo, options: &InstallOptions) -> String {
        let port = options.port.unwrap_or(27017);
        
        match os.package_manager {
            PackageManager::Apt => format!(
                r#"curl -fsSL https://pgp.mongodb.com/server-7.0.asc | sudo gpg -o /usr/share/keyrings/mongodb-server-7.0.gpg --dearmor && \
echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-7.0.gpg ] https://repo.mongodb.org/apt/ubuntu $(lsb_release -cs)/mongodb-org/7.0 multiverse" | sudo tee /etc/apt/sources.list.d/mongodb-org-7.0.list && \
sudo apt-get update && \
sudo apt-get install -y mongodb-org && \
sudo systemctl start mongod && \
sudo systemctl enable mongod && \
echo "MongoDB installed successfully on port {}""#,
                port
            ),
            PackageManager::Yum => format!(
                r#"echo '[mongodb-org-7.0]
name=MongoDB Repository
baseurl=https://repo.mongodb.org/yum/redhat/$releasever/mongodb-org/7.0/x86_64/
gpgcheck=1
enabled=1
gpgkey=https://pgp.mongodb.com/server-7.0.asc' | sudo tee /etc/yum.repos.d/mongodb-org-7.0.repo && \
sudo yum install -y mongodb-org && \
sudo systemctl start mongod && \
sudo systemctl enable mongod && \
echo "MongoDB installed successfully on port {}""#,
                port
            ),
            _ => "echo 'Unsupported package manager'".to_string(),
        }
    }

    fn generate_mariadb_install_script(os: &OsInfo, options: &InstallOptions) -> String {
        let port = options.port.unwrap_or(3306);
        
        match os.package_manager {
            PackageManager::Apt => format!(
                r#"sudo apt-get update && \
sudo apt-get install -y mariadb-server && \
sudo systemctl start mariadb && \
sudo systemctl enable mariadb && \
sudo mysql -e "ALTER USER 'root'@'localhost' IDENTIFIED BY 'root'; FLUSH PRIVILEGES;" && \
echo "MariaDB installed successfully on port {}""#,
                port
            ),
            PackageManager::Yum => format!(
                r#"sudo yum install -y mariadb-server && \
sudo systemctl start mariadb && \
sudo systemctl enable mariadb && \
echo "MariaDB installed successfully on port {}""#,
                port
            ),
            _ => "echo 'Unsupported package manager'".to_string(),
        }
    }
}

// ==================== 操作系统信息 ====================

#[derive(Debug, Clone)]
struct OsInfo {
    id: String,
    version_id: Option<String>,
    package_manager: PackageManager,
}

#[derive(Debug, Clone)]
enum PackageManager {
    Apt,    // Debian/Ubuntu
    Yum,    // CentOS/RHEL/Fedora
    Apk,    // Alpine
    Unknown,
}

// ==================== 命令执行trait ====================

#[async_trait::async_trait]
pub trait CommandExecutor: Send + Sync {
    async fn execute(&self, command: &str) -> Result<String>;
}
