use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Machine {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub auth_type: String, // "password" | "key"
    pub tags: String,      // JSON array string
    pub os: String,        // "linux" | "windows" | "macos"
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineInput {
    pub name: Option<String>,
    pub host: String,
    pub port: Option<i32>,
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub auth_type: String,
    pub tags: Option<Vec<String>>,
    pub os: Option<String>,
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        // 获取数据库路径
        let db_path = Self::get_db_path()?;

        // 确保目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        let db = Database { pool };
        db.init_tables().await?;

        Ok(db)
    }

    fn get_db_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户目录"))?;
        Ok(home.join(".synapsh").join("synapsh.db"))
    }

    async fn init_tables(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS machines (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER DEFAULT 22,
                username TEXT NOT NULL,
                password TEXT,
                private_key_path TEXT,
                auth_type TEXT DEFAULT 'password',
                tags TEXT DEFAULT '[]',
                os TEXT DEFAULT 'linux',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_machines(&self) -> Result<Vec<Machine>> {
        let machines =
            sqlx::query_as::<_, Machine>("SELECT * FROM machines ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await?;

        Ok(machines)
    }

    pub async fn add_machine(&self, input: MachineInput) -> Result<Machine> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let name = input.name.unwrap_or_else(|| input.host.clone());
        let tags = serde_json::to_string(&input.tags.unwrap_or_default())?;
        let os = input.os.unwrap_or_else(|| "linux".to_string());
        let port = input.port.unwrap_or(22);

        sqlx::query(
            r#"
            INSERT INTO machines (id, name, host, port, username, password, private_key_path, auth_type, tags, os, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&name)
        .bind(&input.host)
        .bind(port)
        .bind(&input.username)
        .bind(&input.password)
        .bind(&input.private_key_path)
        .bind(&input.auth_type)
        .bind(&tags)
        .bind(&os)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(Machine {
            id,
            name,
            host: input.host,
            port,
            username: input.username,
            password: input.password,
            private_key_path: input.private_key_path,
            auth_type: input.auth_type,
            tags,
            os,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn update_machine(&self, id: &str, input: MachineInput) -> Result<Machine> {
        let now = chrono::Utc::now().to_rfc3339();
        let name = input.name.unwrap_or_else(|| input.host.clone());
        let tags = serde_json::to_string(&input.tags.unwrap_or_default())?;
        let os = input.os.unwrap_or_else(|| "linux".to_string());
        let port = input.port.unwrap_or(22);

        sqlx::query(
            r#"
            UPDATE machines 
            SET name = ?, host = ?, port = ?, username = ?, password = ?, 
                private_key_path = ?, auth_type = ?, tags = ?, os = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&name)
        .bind(&input.host)
        .bind(port)
        .bind(&input.username)
        .bind(&input.password)
        .bind(&input.private_key_path)
        .bind(&input.auth_type)
        .bind(&tags)
        .bind(&os)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        // 获取更新后的机器
        let machine = sqlx::query_as::<_, Machine>("SELECT * FROM machines WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(machine)
    }

    pub async fn delete_machine(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM machines WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_machine(&self, id: &str) -> Result<Option<Machine>> {
        let machine = sqlx::query_as::<_, Machine>("SELECT * FROM machines WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(machine)
    }
}
