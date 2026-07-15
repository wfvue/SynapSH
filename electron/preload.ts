// Preload script - 暴露安全的 IPC 桥接到渲染进程

import { contextBridge, ipcRenderer } from "electron";

// 定义 API 类型
export interface ElectronAPI {
  // SSH 相关
  connectSSH: (sessionId: string, params: SSHConnectionParams) => Promise<void>;
  disconnectSSH: (sessionId: string) => Promise<void>;
  writeToPty: (sessionId: string, data: string) => Promise<void>;
  resizePty: (sessionId: string, cols: number, rows: number) => Promise<void>;
  testConnection: (params: TestConnectionParams) => Promise<boolean>;

  // 文件操作
  listFiles: (sessionId: string, path: string) => Promise<FileListResult>;
  createFolder: (sessionId: string, path: string) => Promise<void>;
  deleteFile: (sessionId: string, path: string, isDirectory: boolean) => Promise<void>;
  renameFile: (sessionId: string, oldPath: string, newPath: string) => Promise<void>;
  downloadFile: (sessionId: string, remotePath: string) => Promise<string>;
  uploadFile: (sessionId: string, remotePath: string, content: string) => Promise<void>;
  createFile: (sessionId: string, path: string, content?: string) => Promise<void>;
  chmodFile: (sessionId: string, path: string, mode: number) => Promise<void>;

  // 系统监控
  getSystemStats: (sessionId: string) => Promise<SystemStats>;
  killProcess: (sessionId: string, pid: number, signal?: number) => Promise<void>;

  // 机器管理
  listMachines: () => Promise<Machine[]>;
  addMachine: (input: MachineInput) => Promise<Machine>;
  updateMachine: (id: string, input: MachineInput) => Promise<Machine>;
  deleteMachine: (id: string) => Promise<void>;
  getSetting: (key: string, defaultValue?: any) => Promise<any>;
  setSetting: (key: string, value: any) => Promise<void>;

  // AI assistant
  chatWithAI: (
    messages: AIMessage[],
    serverContext?: string,
    conversationId?: string,
  ) => Promise<AIChatResponse>;

  // 数据库管理
  detectDatabases: (sessionId: string) => Promise<DatabaseDetectionResult[]>;
  installDatabase: (params: InstallDatabaseParams) => Promise<string>;
  manageDatabaseService: (params: ManageServiceParams) => Promise<string>;
  getDatabaseConfig: (params: GetDatabaseConfigParams) => Promise<string>;
  updateDatabaseConfig: (params: UpdateDatabaseConfigParams) => Promise<string>;
  getDatabases: (params: GetDatabasesParams) => Promise<DatabaseInfo[]>;
  createDatabase: (params: CreateDatabaseParams) => Promise<void>;
  changeDatabasePassword: (params: ChangePasswordParams) => Promise<void>;
  updateDatabase: (params: UpdateDatabaseParams) => Promise<void>;
  deleteDatabase: (params: DeleteDatabaseParams) => Promise<void>;

  // 浏览器代理
  browserOpen: (sessionId: string, url: string, options?: BrowserLaunchOptions) => Promise<void>;
  browserGetProxyPort: (sessionId: string) => Promise<number | null>;

  // 事件监听
  onSSHData: (callback: (sessionId: string, data: string) => void) => void;
  onBrowserProxyError: (callback: (error: BrowserProxyError) => void) => void;

  // 窗口控制
  minimizeWindow: () => void;
  maximizeWindow: () => void;
  closeWindow: () => void;
}

interface SSHConnectionParams {
  host: string;
  port: number;
  username: string;
  password?: string;
  privateKey?: string;
}

interface TestConnectionParams {
  host: string;
  port: number;
  username: string;
  password?: string;
  privateKey?: string;
}

interface AIMessage {
  role: "user" | "assistant";
  content: string;
}

interface AIChatResponse {
  text: string;
  model: string;
}

interface FileListResult {
  path: string;
  entries: FileEntry[];
  parentPath?: string;
}

interface FileEntry {
  name: string;
  path: string;
  type: "directory" | "file" | "symlink" | "unknown";
  size: number;
  modifiedTime?: string;
  createdTime?: string;
  permissions: string;
  owner: string;
  group: string;
  isHidden: boolean;
}

interface SystemStats {
  cpuPercent: number;
  memory: MemoryInfo;
  disks: DiskInfo[];
  network: NetworkInfo;
  processes: ProcessInfo[];
  system: SystemInfo;
}

interface MemoryInfo {
  total: number;
  used: number;
  free: number;
  cached: number;
}

interface DiskInfo {
  name: string;
  total: number;
  used: number;
  mountPoint: string;
}

interface NetworkInfo {
  rxBytes: number;
  txBytes: number;
}

interface ProcessInfo {
  pid: number;
  name: string;
  cpu: number;
  memory: number;
  user: string;
  status: string;
  statusDesc: string;
  startTime: string;
  elapsedTime: string;
  vsz: number;
  rss: number;
  command: string;
}

interface SystemInfo {
  hostname: string;
  uptime: string;
  loadAverage: [number, number, number];
  cpuCores: number;
  kernelVersion: string;
  totalMemory: number;
}

interface Machine {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  password?: string;
  privateKeyPath?: string;
  authType: string;
  tags: string;
  os: string;
  createdAt: string;
  updatedAt: string;
}

interface MachineInput {
  name?: string;
  host: string;
  port?: number;
  username: string;
  password?: string;
  privateKeyPath?: string;
  authType: string;
  tags?: string[];
  os?: string;
}

interface DatabaseDetectionResult {
  dbType: string;
  installed: boolean;
  version?: string;
  status: string;
  port?: number;
  installPath?: string;
}

interface InstallDatabaseParams {
  sessionId: string;
  dbType: string;
  options: {
    version?: string;
    port?: number;
    rootPassword?: string;
    dataPath?: string;
  };
}

interface ManageServiceParams {
  sessionId: string;
  serviceName: string;
  action: string;
}

interface GetDatabaseConfigParams {
  sessionId: string;
  dbType: string;
}

interface UpdateDatabaseConfigParams {
  sessionId: string;
  dbType: string;
  configContent: string;
}

interface GetDatabasesParams {
  sessionId: string;
  dbType: string;
}

interface DatabaseInfo {
  id: string;
  name: string;
  username: string;
  password: string;
  backupCount: number;
  location: string;
  comment: string;
  status: string;
  createdAt: string;
}

interface CreateDatabaseParams {
  sessionId: string;
  dbType: string;
  name: string;
  username: string;
  password: string;
  comment?: string;
  access?: string;
  charset?: string;
}

interface ChangePasswordParams {
  sessionId: string;
  dbType: string;
  dbId: string;
  username: string;
  newPassword: string;
}

interface UpdateDatabaseParams {
  sessionId: string;
  dbType: string;
  dbId: string;
  comment: string;
}

interface DeleteDatabaseParams {
  sessionId: string;
  dbType: string;
  dbId: string;
  username: string;
}

interface BrowserLaunchOptions {
  profileMode?: string;
}

interface BrowserProxyError {
  sessionId: string;
  host: string;
  port: number;
  message: string;
}

// 暴露 API 到渲染进程
const electronAPI: ElectronAPI = {
  // SSH 相关
  connectSSH: (sessionId, params) => ipcRenderer.invoke("ssh:connect", sessionId, params),
  disconnectSSH: (sessionId) => ipcRenderer.invoke("ssh:disconnect", sessionId),
  writeToPty: (sessionId, data) => ipcRenderer.invoke("ssh:write", sessionId, data),
  resizePty: (sessionId, cols, rows) => ipcRenderer.invoke("ssh:resize", sessionId, cols, rows),
  testConnection: (params) => ipcRenderer.invoke("ssh:test", params),

  // 文件操作
  listFiles: (sessionId, path) => ipcRenderer.invoke("fs:list", sessionId, path),
  createFolder: (sessionId, path) => ipcRenderer.invoke("fs:mkdir", sessionId, path),
  deleteFile: (sessionId, path, isDirectory) =>
    ipcRenderer.invoke("fs:delete", sessionId, path, isDirectory),
  renameFile: (sessionId, oldPath, newPath) =>
    ipcRenderer.invoke("fs:rename", sessionId, oldPath, newPath),
  downloadFile: (sessionId, remotePath) => ipcRenderer.invoke("fs:download", sessionId, remotePath),
  uploadFile: (sessionId, remotePath, content) =>
    ipcRenderer.invoke("fs:upload", sessionId, remotePath, content),
  createFile: (sessionId, path, content) =>
    ipcRenderer.invoke("fs:create", sessionId, path, content),
  chmodFile: (sessionId, path, mode) => ipcRenderer.invoke("fs:chmod", sessionId, path, mode),

  // 系统监控
  getSystemStats: (sessionId) => ipcRenderer.invoke("monitor:stats", sessionId),
  killProcess: (sessionId, pid, signal) =>
    ipcRenderer.invoke("monitor:kill", sessionId, pid, signal),

  // 机器管理
  listMachines: () => ipcRenderer.invoke("db:list-machines"),
  addMachine: (input) => ipcRenderer.invoke("db:add-machine", input),
  updateMachine: (id, input) => ipcRenderer.invoke("db:update-machine", id, input),
  deleteMachine: (id) => ipcRenderer.invoke("db:delete-machine", id),
  getSetting: (key, defaultValue) => ipcRenderer.invoke("db:get-setting", key, defaultValue),
  setSetting: (key, value) => ipcRenderer.invoke("db:set-setting", key, value),

  // AI assistant
  chatWithAI: (messages, serverContext, conversationId) =>
    ipcRenderer.invoke("ai:chat", messages, serverContext, conversationId),

  // 数据库管理
  detectDatabases: (sessionId) => ipcRenderer.invoke("db:detect-databases", sessionId),
  installDatabase: (params) => ipcRenderer.invoke("db:install-database", params),
  manageDatabaseService: (params) => ipcRenderer.invoke("db:manage-service", params),
  getDatabaseConfig: (params) => ipcRenderer.invoke("db:get-config", params),
  updateDatabaseConfig: (params) => ipcRenderer.invoke("db:update-config", params),
  getDatabases: (params) => ipcRenderer.invoke("db:get-databases", params),
  createDatabase: (params) => ipcRenderer.invoke("db:create-database", params),
  changeDatabasePassword: (params) => ipcRenderer.invoke("db:change-password", params),
  updateDatabase: (params) => ipcRenderer.invoke("db:update-database", params),
  deleteDatabase: (params) => ipcRenderer.invoke("db:delete-database", params),

  // 浏览器代理
  browserOpen: (sessionId, url, options) =>
    ipcRenderer.invoke("browser:open", sessionId, url, options),
  browserGetProxyPort: (sessionId) => ipcRenderer.invoke("browser:get-proxy-port", sessionId),

  // 事件监听
  onSSHData: (callback) => {
    const handler = (_event: any, sessionId: string, data: string) => callback(sessionId, data);
    ipcRenderer.on("ssh:data", handler);
    return () => ipcRenderer.removeListener("ssh:data", handler);
  },
  onBrowserProxyError: (callback) => {
    const handler = (_event: any, error: BrowserProxyError) => callback(error);
    ipcRenderer.on("browser:proxy-error", handler);
    return () => ipcRenderer.removeListener("browser:proxy-error", handler);
  },

  // 窗口控制
  minimizeWindow: () => ipcRenderer.send("window:minimize"),
  maximizeWindow: () => ipcRenderer.send("window:maximize"),
  closeWindow: () => ipcRenderer.send("window:close"),
};

contextBridge.exposeInMainWorld("electronAPI", electronAPI);

// 添加类型声明到全局
declare global {
  interface Window {
    electronAPI: ElectronAPI;
  }
}
