// Electron IPC API 适配层
// 替换 @tauri-apps/api 的调用方式，适配 Electron 环境

declare global {
  interface Window {
    electronAPI: ElectronAPI;
  }
}

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
  getSetting: <T>(key: string, defaultValue?: T) => Promise<T>;
  setSetting: (key: string, value: unknown) => Promise<void>;

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
  onSSHData: (callback: (sessionId: string, data: string) => void) => () => void;
  onBrowserProxyError: (callback: (error: BrowserProxyError) => void) => () => void;

  // 窗口控制
  minimizeWindow: () => void;
  maximizeWindow: () => void;
  closeWindow: () => void;
}

export interface SSHConnectionParams {
  host: string;
  port: number;
  username: string;
  password?: string;
  privateKey?: string;
}

export interface TestConnectionParams {
  host: string;
  port: number;
  username: string;
  password?: string;
  privateKey?: string;
}

export interface FileListResult {
  path: string;
  entries: FileEntry[];
  parentPath?: string;
}

export interface FileEntry {
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

export interface SystemStats {
  cpuPercent: number;
  memory: MemoryInfo;
  disks: DiskInfo[];
  network: NetworkInfo;
  processes: ProcessInfo[];
  system: SystemInfo;
}

export interface MemoryInfo {
  total: number;
  used: number;
  free: number;
  cached: number;
}

export interface DiskInfo {
  name: string;
  total: number;
  used: number;
  mountPoint: string;
}

export interface NetworkInfo {
  rxBytes: number;
  txBytes: number;
}

export interface ProcessInfo {
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

export interface SystemInfo {
  hostname: string;
  uptime: string;
  loadAverage: [number, number, number];
  cpuCores: number;
  kernelVersion: string;
  totalMemory: number;
}

export interface Machine {
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

export interface MachineInput {
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

export interface DatabaseDetectionResult {
  dbType: string;
  installed: boolean;
  version?: string;
  status: string;
  port?: number;
  installPath?: string;
}

export interface InstallDatabaseParams {
  sessionId: string;
  dbType: string;
  options: {
    version?: string;
    port?: number;
    rootPassword?: string;
    dataPath?: string;
  };
}

export interface ManageServiceParams {
  sessionId: string;
  serviceName: string;
  action: string;
}

export interface GetDatabaseConfigParams {
  sessionId: string;
  dbType: string;
}

export interface UpdateDatabaseConfigParams {
  sessionId: string;
  dbType: string;
  configContent: string;
}

export interface GetDatabasesParams {
  sessionId: string;
  dbType: string;
}

export interface DatabaseInfo {
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

export interface CreateDatabaseParams {
  sessionId: string;
  dbType: string;
  name: string;
  username: string;
  password: string;
  comment?: string;
  access?: string;
  charset?: string;
}

export interface ChangePasswordParams {
  sessionId: string;
  dbType: string;
  dbId: string;
  username: string;
  newPassword: string;
}

export interface UpdateDatabaseParams {
  sessionId: string;
  dbType: string;
  dbId: string;
  comment: string;
}

export interface DeleteDatabaseParams {
  sessionId: string;
  dbType: string;
  dbId: string;
  username: string;
}

export interface BrowserLaunchOptions {
  profileMode?: string;
}

export interface BrowserProxyError {
  sessionId: string;
  host: string;
  port: number;
  message: string;
}

export interface AIMessage {
  role: "user" | "assistant";
  content: string;
}

export interface AIChatResponse {
  text: string;
  model: string;
}

// 获取 API 实例
function getAPI(): ElectronAPI {
  if (typeof window !== "undefined" && window.electronAPI) {
    return window.electronAPI;
  }
  throw new Error("Electron API is not available");
}

// 封装调用
export const api = {
  // SSH
  connectSSH: (sessionId: string, params: SSHConnectionParams) =>
    getAPI().connectSSH(sessionId, params),
  disconnectSSH: (sessionId: string) => getAPI().disconnectSSH(sessionId),
  writeToPty: (sessionId: string, data: string) => getAPI().writeToPty(sessionId, data),
  resizePty: (sessionId: string, cols: number, rows: number) =>
    getAPI().resizePty(sessionId, cols, rows),
  testConnection: (params: TestConnectionParams) => getAPI().testConnection(params),

  // 文件操作
  listFiles: (sessionId: string, path: string) => getAPI().listFiles(sessionId, path),
  createFolder: (sessionId: string, path: string) => getAPI().createFolder(sessionId, path),
  deleteFile: (sessionId: string, path: string, isDirectory: boolean) =>
    getAPI().deleteFile(sessionId, path, isDirectory),
  renameFile: (sessionId: string, oldPath: string, newPath: string) =>
    getAPI().renameFile(sessionId, oldPath, newPath),
  downloadFile: (sessionId: string, remotePath: string) =>
    getAPI().downloadFile(sessionId, remotePath),
  uploadFile: (sessionId: string, remotePath: string, content: string) =>
    getAPI().uploadFile(sessionId, remotePath, content),
  createFile: (sessionId: string, path: string, content?: string) =>
    getAPI().createFile(sessionId, path, content),
  chmodFile: (sessionId: string, path: string, mode: number) =>
    getAPI().chmodFile(sessionId, path, mode),

  // 系统监控
  getSystemStats: (sessionId: string) => getAPI().getSystemStats(sessionId),
  killProcess: (sessionId: string, pid: number, signal?: number) =>
    getAPI().killProcess(sessionId, pid, signal),

  // 机器管理
  listMachines: () => getAPI().listMachines(),
  addMachine: (input: MachineInput) => getAPI().addMachine(input),
  updateMachine: (id: string, input: MachineInput) => getAPI().updateMachine(id, input),
  deleteMachine: (id: string) => getAPI().deleteMachine(id),

  // AI assistant
  chatWithAI: (messages: AIMessage[], serverContext?: string, conversationId?: string) =>
    getAPI().chatWithAI(messages, serverContext, conversationId),

  // 数据库管理
  detectDatabases: (sessionId: string) => getAPI().detectDatabases(sessionId),
  installDatabase: (params: InstallDatabaseParams) => getAPI().installDatabase(params),
  manageDatabaseService: (params: ManageServiceParams) => getAPI().manageDatabaseService(params),
  getDatabaseConfig: (params: GetDatabaseConfigParams) => getAPI().getDatabaseConfig(params),
  updateDatabaseConfig: (params: UpdateDatabaseConfigParams) =>
    getAPI().updateDatabaseConfig(params),
  getDatabases: (params: GetDatabasesParams) => getAPI().getDatabases(params),
  createDatabase: (params: CreateDatabaseParams) => getAPI().createDatabase(params),
  changeDatabasePassword: (params: ChangePasswordParams) => getAPI().changeDatabasePassword(params),
  updateDatabase: (params: UpdateDatabaseParams) => getAPI().updateDatabase(params),
  deleteDatabase: (params: DeleteDatabaseParams) => getAPI().deleteDatabase(params),

  // 浏览器代理
  browserOpen: (sessionId: string, url: string, options?: BrowserLaunchOptions) =>
    getAPI().browserOpen(sessionId, url, options),
  browserGetProxyPort: (sessionId: string) => getAPI().browserGetProxyPort(sessionId),

  // 事件监听
  onSSHData: (callback: (sessionId: string, data: string) => void) => getAPI().onSSHData(callback),
  onBrowserProxyError: (callback: (error: BrowserProxyError) => void) =>
    getAPI().onBrowserProxyError(callback),

  // 窗口控制
  minimizeWindow: () => getAPI().minimizeWindow(),
  maximizeWindow: () => getAPI().maximizeWindow(),
  closeWindow: () => getAPI().closeWindow(),
};

export default api;
