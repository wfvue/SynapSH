"use strict";
// Electron 主进程入口
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const electron_1 = require("electron");
const path = __importStar(require("path"));
const ssh_1 = require("./services/ssh");
const machine_db_1 = require("./services/machine-db");
const browser_1 = require("./services/browser");
// 全局变量
let mainWindow = null;
const sshManager = new ssh_1.SSHSessionManager();
const machineDb = new machine_db_1.MachineDatabase();
const browserManager = new browser_1.BrowserManager();
// 创建主窗口
function createWindow() {
    mainWindow = new electron_1.BrowserWindow({
        width: 1200,
        height: 800,
        minWidth: 800,
        minHeight: 600,
        frame: true,
        titleBarStyle: 'hiddenInset',
        backgroundColor: '#0b0d10',
        webPreferences: {
            preload: path.join(__dirname, 'preload.js'),
            contextIsolation: true,
            nodeIntegration: false,
            sandbox: false,
        },
    });
    // 加载前端页面
    if (process.env.NODE_ENV === 'development') {
        mainWindow.loadURL('http://localhost:19000');
        mainWindow.webContents.openDevTools();
    }
    else {
        mainWindow.loadFile(path.join(__dirname, '../dist/index.html'));
    }
    mainWindow.on('closed', () => {
        mainWindow = null;
    });
    // 创建菜单
    createMenu();
}
// 创建应用菜单
function createMenu() {
    const template = [
        {
            label: 'SynapSH',
            submenu: [
                { label: '关于 SynapSH', role: 'about' },
                { type: 'separator' },
                { label: '偏好设置', accelerator: 'CmdOrCtrl+,', click: () => { } },
                { type: 'separator' },
                { label: '隐藏 SynapSH', accelerator: 'CmdOrCtrl+H', role: 'hide' },
                { label: '隐藏其他', accelerator: 'CmdOrCtrl+Alt+H', role: 'hideOthers' },
                { label: '显示全部', role: 'unhide' },
                { type: 'separator' },
                { label: '退出 SynapSH', accelerator: 'CmdOrCtrl+Q', role: 'quit' },
            ],
        },
        {
            label: '编辑',
            submenu: [
                { label: '撤销', accelerator: 'CmdOrCtrl+Z', role: 'undo' },
                { label: '重做', accelerator: 'Shift+CmdOrCtrl+Z', role: 'redo' },
                { type: 'separator' },
                { label: '剪切', accelerator: 'CmdOrCtrl+X', role: 'cut' },
                { label: '复制', accelerator: 'CmdOrCtrl+C', role: 'copy' },
                { label: '粘贴', accelerator: 'CmdOrCtrl+V', role: 'paste' },
                { label: '全选', accelerator: 'CmdOrCtrl+A', role: 'selectAll' },
            ],
        },
        {
            label: '窗口',
            submenu: [
                { label: '最小化', accelerator: 'CmdOrCtrl+M', role: 'minimize' },
                { label: '关闭', accelerator: 'CmdOrCtrl+W', role: 'close' },
            ],
        },
        {
            label: '帮助',
            submenu: [
                { label: '文档', click: () => electron_1.shell.openExternal('https://github.com/synapsh') },
            ],
        },
    ];
    const menu = electron_1.Menu.buildFromTemplate(template);
    electron_1.Menu.setApplicationMenu(menu);
}
// 注册 IPC 处理程序
function registerIPCHandlers() {
    // SSH 相关
    electron_1.ipcMain.handle('ssh:connect', async (_event, sessionId, params) => {
        try {
            await sshManager.connect(sessionId, params);
            // 监听 SSH 数据并转发到渲染进程
            sshManager.onData(sessionId, (data) => {
                if (mainWindow) {
                    mainWindow.webContents.send('ssh:data', sessionId, data.toString('base64'));
                }
            });
            return { success: true };
        }
        catch (error) {
            return { success: false, error: String(error) };
        }
    });
    electron_1.ipcMain.handle('ssh:disconnect', async (_event, sessionId) => {
        await sshManager.disconnect(sessionId);
        return { success: true };
    });
    electron_1.ipcMain.handle('ssh:write', async (_event, sessionId, data) => {
        const buffer = Buffer.from(data, 'utf-8');
        await sshManager.write(sessionId, buffer);
        return { success: true };
    });
    electron_1.ipcMain.handle('ssh:resize', async (_event, sessionId, cols, rows) => {
        await sshManager.resize(sessionId, cols, rows);
        return { success: true };
    });
    electron_1.ipcMain.handle('ssh:test', async (_event, params) => {
        try {
            const result = await sshManager.testConnection(params);
            return result;
        }
        catch (error) {
            return false;
        }
    });
    // 文件操作
    electron_1.ipcMain.handle('fs:list', async (_event, sessionId, remotePath) => {
        return await sshManager.listDirectory(sessionId, remotePath);
    });
    electron_1.ipcMain.handle('fs:mkdir', async (_event, sessionId, remotePath) => {
        await sshManager.createDirectory(sessionId, remotePath);
        return { success: true };
    });
    electron_1.ipcMain.handle('fs:delete', async (_event, sessionId, remotePath, isDirectory) => {
        if (isDirectory) {
            await sshManager.removeDirectory(sessionId, remotePath);
        }
        else {
            await sshManager.removeFile(sessionId, remotePath);
        }
        return { success: true };
    });
    electron_1.ipcMain.handle('fs:rename', async (_event, sessionId, oldPath, newPath) => {
        await sshManager.rename(sessionId, oldPath, newPath);
        return { success: true };
    });
    electron_1.ipcMain.handle('fs:download', async (_event, sessionId, remotePath) => {
        const content = await sshManager.readFile(sessionId, remotePath);
        return content.toString('base64');
    });
    electron_1.ipcMain.handle('fs:upload', async (_event, sessionId, remotePath, base64Content) => {
        const content = Buffer.from(base64Content, 'base64');
        await sshManager.writeFile(sessionId, remotePath, content);
        return { success: true };
    });
    electron_1.ipcMain.handle('fs:create', async (_event, sessionId, remotePath, content) => {
        const data = content ? Buffer.from(content, 'utf-8') : Buffer.alloc(0);
        await sshManager.writeFile(sessionId, remotePath, data);
        return { success: true };
    });
    electron_1.ipcMain.handle('fs:chmod', async (_event, sessionId, remotePath, mode) => {
        await sshManager.chmod(sessionId, remotePath, mode);
        return { success: true };
    });
    // 系统监控
    electron_1.ipcMain.handle('monitor:stats', async (_event, sessionId) => {
        return await sshManager.getSystemStats(sessionId);
    });
    electron_1.ipcMain.handle('monitor:kill', async (_event, sessionId, pid, signal) => {
        await sshManager.killProcess(sessionId, pid, signal);
        return { success: true };
    });
    // 机器管理 (本地数据库)
    electron_1.ipcMain.handle('db:list-machines', async () => {
        return await machineDb.listMachines();
    });
    electron_1.ipcMain.handle('db:add-machine', async (_event, input) => {
        return await machineDb.addMachine(input);
    });
    electron_1.ipcMain.handle('db:update-machine', async (_event, id, input) => {
        return await machineDb.updateMachine(id, input);
    });
    electron_1.ipcMain.handle('db:delete-machine', async (_event, id) => {
        await machineDb.deleteMachine(id);
        return { success: true };
    });
    // 数据库管理 (远程服务器)
    electron_1.ipcMain.handle('db:detect-databases', async (_event, sessionId) => {
        return await sshManager.detectDatabases(sessionId);
    });
    electron_1.ipcMain.handle('db:install-database', async (_event, params) => {
        return await sshManager.installDatabase(params);
    });
    electron_1.ipcMain.handle('db:manage-service', async (_event, params) => {
        return await sshManager.manageDatabaseService(params);
    });
    electron_1.ipcMain.handle('db:get-config', async (_event, params) => {
        return await sshManager.getDatabaseConfig(params);
    });
    electron_1.ipcMain.handle('db:update-config', async (_event, params) => {
        return await sshManager.updateDatabaseConfig(params);
    });
    electron_1.ipcMain.handle('db:get-databases', async (_event, params) => {
        return await sshManager.getDatabases(params);
    });
    electron_1.ipcMain.handle('db:create-database', async (_event, params) => {
        return await sshManager.createDatabase(params);
    });
    electron_1.ipcMain.handle('db:change-password', async (_event, params) => {
        return await sshManager.changeDatabasePassword(params);
    });
    electron_1.ipcMain.handle('db:update-database', async (_event, params) => {
        return await sshManager.updateDatabase(params);
    });
    electron_1.ipcMain.handle('db:delete-database', async (_event, params) => {
        return await sshManager.deleteDatabase(params);
    });
    // 浏览器代理
    electron_1.ipcMain.handle('browser:open', async (_event, sessionId, url, options) => {
        try {
            await browserManager.openBrowser(sessionId, url, options, sshManager);
            return { success: true };
        }
        catch (error) {
            if (mainWindow) {
                mainWindow.webContents.send('browser:proxy-error', {
                    sessionId,
                    host: new URL(url).hostname,
                    port: 443,
                    message: String(error),
                });
            }
            return { success: false, error: String(error) };
        }
    });
    electron_1.ipcMain.handle('browser:get-proxy-port', async (_event, sessionId) => {
        return browserManager.getProxyPort(sessionId);
    });
    // 窗口控制
    electron_1.ipcMain.on('window:minimize', () => {
        mainWindow?.minimize();
    });
    electron_1.ipcMain.on('window:maximize', () => {
        if (mainWindow?.isMaximized()) {
            mainWindow.unmaximize();
        }
        else {
            mainWindow?.maximize();
        }
    });
    electron_1.ipcMain.on('window:close', () => {
        mainWindow?.close();
    });
}
// 应用准备就绪
electron_1.app.whenReady().then(async () => {
    // 初始化数据库
    await machineDb.initialize();
    // 注册 IPC 处理程序
    registerIPCHandlers();
    // 创建窗口
    createWindow();
    electron_1.app.on('activate', () => {
        if (electron_1.BrowserWindow.getAllWindows().length === 0) {
            createWindow();
        }
    });
});
// 所有窗口关闭
electron_1.app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') {
        electron_1.app.quit();
    }
});
// 应用退出前清理
electron_1.app.on('before-quit', async () => {
    // 关闭所有 SSH 连接
    await sshManager.disconnectAll();
    // 关闭所有浏览器进程
    browserManager.closeAll();
});
//# sourceMappingURL=main.js.map