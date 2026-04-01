"use strict";
// Preload script - 暴露安全的 IPC 桥接到渲染进程
Object.defineProperty(exports, "__esModule", { value: true });
const electron_1 = require("electron");
// 暴露 API 到渲染进程
const electronAPI = {
    // SSH 相关
    connectSSH: (sessionId, params) => electron_1.ipcRenderer.invoke('ssh:connect', sessionId, params),
    disconnectSSH: (sessionId) => electron_1.ipcRenderer.invoke('ssh:disconnect', sessionId),
    writeToPty: (sessionId, data) => electron_1.ipcRenderer.invoke('ssh:write', sessionId, data),
    resizePty: (sessionId, cols, rows) => electron_1.ipcRenderer.invoke('ssh:resize', sessionId, cols, rows),
    testConnection: (params) => electron_1.ipcRenderer.invoke('ssh:test', params),
    // 文件操作
    listFiles: (sessionId, path) => electron_1.ipcRenderer.invoke('fs:list', sessionId, path),
    createFolder: (sessionId, path) => electron_1.ipcRenderer.invoke('fs:mkdir', sessionId, path),
    deleteFile: (sessionId, path, isDirectory) => electron_1.ipcRenderer.invoke('fs:delete', sessionId, path, isDirectory),
    renameFile: (sessionId, oldPath, newPath) => electron_1.ipcRenderer.invoke('fs:rename', sessionId, oldPath, newPath),
    downloadFile: (sessionId, remotePath) => electron_1.ipcRenderer.invoke('fs:download', sessionId, remotePath),
    uploadFile: (sessionId, remotePath, content) => electron_1.ipcRenderer.invoke('fs:upload', sessionId, remotePath, content),
    createFile: (sessionId, path, content) => electron_1.ipcRenderer.invoke('fs:create', sessionId, path, content),
    chmodFile: (sessionId, path, mode) => electron_1.ipcRenderer.invoke('fs:chmod', sessionId, path, mode),
    // 系统监控
    getSystemStats: (sessionId) => electron_1.ipcRenderer.invoke('monitor:stats', sessionId),
    killProcess: (sessionId, pid, signal) => electron_1.ipcRenderer.invoke('monitor:kill', sessionId, pid, signal),
    // 机器管理
    listMachines: () => electron_1.ipcRenderer.invoke('db:list-machines'),
    addMachine: (input) => electron_1.ipcRenderer.invoke('db:add-machine', input),
    updateMachine: (id, input) => electron_1.ipcRenderer.invoke('db:update-machine', id, input),
    deleteMachine: (id) => electron_1.ipcRenderer.invoke('db:delete-machine', id),
    // 数据库管理
    detectDatabases: (sessionId) => electron_1.ipcRenderer.invoke('db:detect-databases', sessionId),
    installDatabase: (params) => electron_1.ipcRenderer.invoke('db:install-database', params),
    manageDatabaseService: (params) => electron_1.ipcRenderer.invoke('db:manage-service', params),
    getDatabaseConfig: (params) => electron_1.ipcRenderer.invoke('db:get-config', params),
    updateDatabaseConfig: (params) => electron_1.ipcRenderer.invoke('db:update-config', params),
    getDatabases: (params) => electron_1.ipcRenderer.invoke('db:get-databases', params),
    createDatabase: (params) => electron_1.ipcRenderer.invoke('db:create-database', params),
    changeDatabasePassword: (params) => electron_1.ipcRenderer.invoke('db:change-password', params),
    updateDatabase: (params) => electron_1.ipcRenderer.invoke('db:update-database', params),
    deleteDatabase: (params) => electron_1.ipcRenderer.invoke('db:delete-database', params),
    // 浏览器代理
    browserOpen: (sessionId, url, options) => electron_1.ipcRenderer.invoke('browser:open', sessionId, url, options),
    browserGetProxyPort: (sessionId) => electron_1.ipcRenderer.invoke('browser:get-proxy-port', sessionId),
    // 事件监听
    onSSHData: (callback) => {
        const handler = (_event, sessionId, data) => callback(sessionId, data);
        electron_1.ipcRenderer.on('ssh:data', handler);
        return () => electron_1.ipcRenderer.removeListener('ssh:data', handler);
    },
    onBrowserProxyError: (callback) => {
        const handler = (_event, error) => callback(error);
        electron_1.ipcRenderer.on('browser:proxy-error', handler);
        return () => electron_1.ipcRenderer.removeListener('browser:proxy-error', handler);
    },
    // 窗口控制
    minimizeWindow: () => electron_1.ipcRenderer.send('window:minimize'),
    maximizeWindow: () => electron_1.ipcRenderer.send('window:maximize'),
    closeWindow: () => electron_1.ipcRenderer.send('window:close'),
};
electron_1.contextBridge.exposeInMainWorld('electronAPI', electronAPI);
//# sourceMappingURL=preload.js.map