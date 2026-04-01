export interface ElectronAPI {
    connectSSH: (sessionId: string, params: SSHConnectionParams) => Promise<void>;
    disconnectSSH: (sessionId: string) => Promise<void>;
    writeToPty: (sessionId: string, data: string) => Promise<void>;
    resizePty: (sessionId: string, cols: number, rows: number) => Promise<void>;
    testConnection: (params: TestConnectionParams) => Promise<boolean>;
    listFiles: (sessionId: string, path: string) => Promise<FileListResult>;
    createFolder: (sessionId: string, path: string) => Promise<void>;
    deleteFile: (sessionId: string, path: string, isDirectory: boolean) => Promise<void>;
    renameFile: (sessionId: string, oldPath: string, newPath: string) => Promise<void>;
    downloadFile: (sessionId: string, remotePath: string) => Promise<string>;
    uploadFile: (sessionId: string, remotePath: string, content: string) => Promise<void>;
    createFile: (sessionId: string, path: string, content?: string) => Promise<void>;
    chmodFile: (sessionId: string, path: string, mode: number) => Promise<void>;
    getSystemStats: (sessionId: string) => Promise<SystemStats>;
    killProcess: (sessionId: string, pid: number, signal?: number) => Promise<void>;
    listMachines: () => Promise<Machine[]>;
    addMachine: (input: MachineInput) => Promise<Machine>;
    updateMachine: (id: string, input: MachineInput) => Promise<Machine>;
    deleteMachine: (id: string) => Promise<void>;
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
    browserOpen: (sessionId: string, url: string, options?: BrowserLaunchOptions) => Promise<void>;
    browserGetProxyPort: (sessionId: string) => Promise<number | null>;
    onSSHData: (callback: (sessionId: string, data: string) => void) => void;
    onBrowserProxyError: (callback: (error: BrowserProxyError) => void) => void;
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
interface FileListResult {
    path: string;
    entries: FileEntry[];
    parentPath?: string;
}
interface FileEntry {
    name: string;
    path: string;
    type: 'directory' | 'file' | 'symlink' | 'unknown';
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
declare global {
    interface Window {
        electronAPI: ElectronAPI;
    }
}
export {};
