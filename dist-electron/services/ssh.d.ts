interface SSHConnectionParams {
    host: string;
    port: number;
    username: string;
    password?: string;
    privateKey?: string;
}
interface RemoteFileEntry {
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
interface FileListResult {
    path: string;
    entries: RemoteFileEntry[];
    parentPath?: string;
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
type DataCallback = (data: Buffer) => void;
export declare class SSHSessionManager {
    private sessions;
    private dataCallbacks;
    private sftpSessions;
    private shellChannels;
    connect(sessionId: string, params: SSHConnectionParams): Promise<void>;
    disconnect(sessionId: string): Promise<void>;
    disconnectAll(): Promise<void>;
    private cleanup;
    onData(sessionId: string, callback: DataCallback): void;
    write(sessionId: string, data: Buffer): Promise<void>;
    resize(sessionId: string, cols: number, rows: number): Promise<void>;
    testConnection(params: SSHConnectionParams): Promise<boolean>;
    private getSFTP;
    listDirectory(sessionId: string, remotePath: string): Promise<FileListResult>;
    private formatPermissions;
    createDirectory(sessionId: string, remotePath: string): Promise<void>;
    removeFile(sessionId: string, remotePath: string): Promise<void>;
    removeDirectory(sessionId: string, remotePath: string): Promise<void>;
    rename(sessionId: string, oldPath: string, newPath: string): Promise<void>;
    readFile(sessionId: string, remotePath: string): Promise<Buffer>;
    writeFile(sessionId: string, remotePath: string, content: Buffer): Promise<void>;
    chmod(sessionId: string, remotePath: string, mode: number): Promise<void>;
    execCommand(sessionId: string, command: string): Promise<string>;
    getSystemStats(sessionId: string): Promise<SystemStats>;
    private parseSystemStats;
    private parseCPU;
    private parseMemory;
    private parseDisks;
    private parseNetwork;
    private parseProcesses;
    private parseSystem;
    private extractProcessName;
    private getStatusDescription;
    killProcess(sessionId: string, pid: number, signal?: number): Promise<void>;
    detectDatabases(sessionId: string): Promise<any[]>;
    private parseDatabaseDetection;
    private getDefaultPort;
    installDatabase(params: any): Promise<string>;
    manageDatabaseService(params: any): Promise<string>;
    getDatabaseConfig(params: any): Promise<string>;
    updateDatabaseConfig(params: any): Promise<string>;
    getDatabases(params: any): Promise<any[]>;
    createDatabase(params: any): Promise<void>;
    changeDatabasePassword(params: any): Promise<void>;
    updateDatabase(params: any): Promise<void>;
    deleteDatabase(params: any): Promise<void>;
}
export {};
