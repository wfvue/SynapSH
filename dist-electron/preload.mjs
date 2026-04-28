import { contextBridge, ipcRenderer } from "electron";
//#region electron/preload.ts
contextBridge.exposeInMainWorld("electronAPI", {
	connectSSH: (sessionId, params) => ipcRenderer.invoke("ssh:connect", sessionId, params),
	disconnectSSH: (sessionId) => ipcRenderer.invoke("ssh:disconnect", sessionId),
	writeToPty: (sessionId, data) => ipcRenderer.invoke("ssh:write", sessionId, data),
	resizePty: (sessionId, cols, rows) => ipcRenderer.invoke("ssh:resize", sessionId, cols, rows),
	testConnection: (params) => ipcRenderer.invoke("ssh:test", params),
	listFiles: (sessionId, path) => ipcRenderer.invoke("fs:list", sessionId, path),
	createFolder: (sessionId, path) => ipcRenderer.invoke("fs:mkdir", sessionId, path),
	deleteFile: (sessionId, path, isDirectory) => ipcRenderer.invoke("fs:delete", sessionId, path, isDirectory),
	renameFile: (sessionId, oldPath, newPath) => ipcRenderer.invoke("fs:rename", sessionId, oldPath, newPath),
	downloadFile: (sessionId, remotePath) => ipcRenderer.invoke("fs:download", sessionId, remotePath),
	uploadFile: (sessionId, remotePath, content) => ipcRenderer.invoke("fs:upload", sessionId, remotePath, content),
	createFile: (sessionId, path, content) => ipcRenderer.invoke("fs:create", sessionId, path, content),
	chmodFile: (sessionId, path, mode) => ipcRenderer.invoke("fs:chmod", sessionId, path, mode),
	getSystemStats: (sessionId) => ipcRenderer.invoke("monitor:stats", sessionId),
	killProcess: (sessionId, pid, signal) => ipcRenderer.invoke("monitor:kill", sessionId, pid, signal),
	listMachines: () => ipcRenderer.invoke("db:list-machines"),
	addMachine: (input) => ipcRenderer.invoke("db:add-machine", input),
	updateMachine: (id, input) => ipcRenderer.invoke("db:update-machine", id, input),
	deleteMachine: (id) => ipcRenderer.invoke("db:delete-machine", id),
	getSetting: (key, defaultValue) => ipcRenderer.invoke("db:get-setting", key, defaultValue),
	setSetting: (key, value) => ipcRenderer.invoke("db:set-setting", key, value),
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
	browserOpen: (sessionId, url, options) => ipcRenderer.invoke("browser:open", sessionId, url, options),
	browserGetProxyPort: (sessionId) => ipcRenderer.invoke("browser:get-proxy-port", sessionId),
	onSSHData: (callback) => {
		const handler = (_event, sessionId, data) => callback(sessionId, data);
		ipcRenderer.on("ssh:data", handler);
		return () => ipcRenderer.removeListener("ssh:data", handler);
	},
	onBrowserProxyError: (callback) => {
		const handler = (_event, error) => callback(error);
		ipcRenderer.on("browser:proxy-error", handler);
		return () => ipcRenderer.removeListener("browser:proxy-error", handler);
	},
	minimizeWindow: () => ipcRenderer.send("window:minimize"),
	maximizeWindow: () => ipcRenderer.send("window:maximize"),
	closeWindow: () => ipcRenderer.send("window:close")
});
//#endregion
