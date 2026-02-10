use serde::{Deserialize, Serialize};

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
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: FileEntryType,
    pub size: u64,
    pub modified_time: Option<String>,
    pub created_time: Option<String>,
    pub permissions: String,
    pub owner: String,
    pub group: String,
    pub is_hidden: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileListResult {
    pub path: String,
    pub entries: Vec<FileEntry>,
    pub parent_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct FileTransferProgress {
    pub file_name: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub percentage: f64,
}
