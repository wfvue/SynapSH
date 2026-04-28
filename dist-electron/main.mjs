import { BrowserWindow, Menu, app, ipcMain, shell } from "electron";
import * as path from "path";
import { fileURLToPath } from "node:url";
import { dirname } from "node:path";
import * as ssh2 from "ssh2";
import { createRequire } from "node:module";
import * as fs from "fs";
import * as os from "os";
import { randomFillSync, randomUUID } from "crypto";
import { spawn } from "child_process";
//#region \0rolldown/runtime.js
var __require = /* @__PURE__ */ ((x) => typeof require !== "undefined" ? require : typeof Proxy !== "undefined" ? new Proxy(x, { get: (a, b) => (typeof require !== "undefined" ? require : a)[b] }) : x)(function(x) {
	if (typeof require !== "undefined") return require.apply(this, arguments);
	throw Error("Calling `require` for \"" + x + "\" in an environment that doesn't expose the `require` function. See https://rolldown.rs/in-depth/bundling-cjs#require-external-modules for more details.");
});
//#endregion
//#region electron/services/ssh.ts
var S_IFMT = 61440;
var S_IFREG = 32768;
var S_IFDIR = 16384;
var S_IFLNK = 40960;
var SSHSessionManager = class {
	sessions = /* @__PURE__ */ new Map();
	dataCallbacks = /* @__PURE__ */ new Map();
	sftpSessions = /* @__PURE__ */ new Map();
	shellChannels = /* @__PURE__ */ new Map();
	async connect(sessionId, params) {
		return new Promise((resolve, reject) => {
			const client = new ssh2.Client();
			const config = {
				host: params.host,
				port: params.port,
				username: params.username,
				readyTimeout: 3e4,
				keepaliveInterval: 2e4,
				keepaliveCountMax: 6
			};
			if (params.privateKey) config.privateKey = params.privateKey;
			else if (params.password) config.password = params.password;
			client.on("ready", () => {
				console.log(`SSH connected: ${sessionId}`);
				this.sessions.set(sessionId, client);
				client.shell((err, stream) => {
					if (err) {
						reject(err);
						return;
					}
					this.shellChannels.set(sessionId, stream);
					stream.on("data", (data) => {
						(this.dataCallbacks.get(sessionId) || []).forEach((cb) => cb(data));
					});
					stream.on("close", () => {
						console.log(`Shell closed: ${sessionId}`);
					});
					resolve();
				});
			});
			client.on("error", (err) => {
				console.error(`SSH error: ${sessionId}`, err);
				reject(err);
			});
			client.on("close", () => {
				console.log(`SSH disconnected: ${sessionId}`);
				this.cleanup(sessionId);
			});
			client.connect(config);
		});
	}
	async disconnect(sessionId) {
		const client = this.sessions.get(sessionId);
		if (client) {
			client.end();
			this.cleanup(sessionId);
		}
	}
	async disconnectAll() {
		for (const sessionId of this.sessions.keys()) await this.disconnect(sessionId);
	}
	cleanup(sessionId) {
		this.sessions.delete(sessionId);
		this.dataCallbacks.delete(sessionId);
		this.sftpSessions.delete(sessionId);
		this.shellChannels.delete(sessionId);
	}
	onData(sessionId, callback) {
		const callbacks = this.dataCallbacks.get(sessionId) || [];
		callbacks.push(callback);
		this.dataCallbacks.set(sessionId, callbacks);
	}
	async write(sessionId, data) {
		const stream = this.shellChannels.get(sessionId);
		if (stream) stream.write(data);
	}
	async resize(sessionId, cols, rows) {
		const stream = this.shellChannels.get(sessionId);
		if (stream) stream.setWindow(rows, cols, 0, 0);
	}
	async testConnection(params) {
		return new Promise((resolve) => {
			const client = new ssh2.Client();
			const config = {
				host: params.host,
				port: params.port,
				username: params.username,
				readyTimeout: 1e4
			};
			if (params.privateKey) config.privateKey = params.privateKey;
			else if (params.password) config.password = params.password;
			client.on("ready", () => {
				client.end();
				resolve(true);
			});
			client.on("error", () => {
				resolve(false);
			});
			client.connect(config);
		});
	}
	async getSFTP(sessionId) {
		if (this.sftpSessions.has(sessionId)) return this.sftpSessions.get(sessionId);
		const client = this.sessions.get(sessionId);
		if (!client) throw new Error("Session not found");
		return new Promise((resolve, reject) => {
			client.sftp((err, sftp) => {
				if (err) {
					reject(err);
					return;
				}
				this.sftpSessions.set(sessionId, sftp);
				resolve(sftp);
			});
		});
	}
	async listDirectory(sessionId, remotePath) {
		const sftp = await this.getSFTP(sessionId);
		return new Promise((resolve, reject) => {
			const entries = [];
			const listCallback = (err, list) => {
				if (err) {
					reject(err);
					return;
				}
				for (const file of list) {
					const attrs = file.attrs;
					const mode = attrs.mode;
					const isDirectory = (mode & S_IFMT) === S_IFDIR;
					const isSymlink = (mode & S_IFMT) === S_IFLNK;
					const isFile = (mode & S_IFMT) === S_IFREG;
					let fileType;
					if (isDirectory) fileType = "directory";
					else if (isSymlink) fileType = "symlink";
					else if (isFile) fileType = "file";
					else fileType = "unknown";
					const permissions = this.formatPermissions(mode);
					entries.push({
						name: file.filename,
						path: `${remotePath.replace(/\/$/, "")}/${file.filename}`,
						type: fileType,
						size: attrs.size,
						modifiedTime: attrs.mtime ? (/* @__PURE__ */ new Date(attrs.mtime * 1e3)).toISOString() : void 0,
						createdTime: attrs.atime ? (/* @__PURE__ */ new Date(attrs.atime * 1e3)).toISOString() : void 0,
						permissions,
						owner: String(attrs.uid),
						group: String(attrs.gid),
						isHidden: file.filename.startsWith(".")
					});
				}
				entries.sort((a, b) => {
					if (a.type === "directory" && b.type !== "directory") return -1;
					if (a.type !== "directory" && b.type === "directory") return 1;
					return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
				});
				let parentPath;
				if (remotePath !== "/" && remotePath !== "") {
					const parts = remotePath.split("/").filter(Boolean);
					parts.pop();
					parentPath = parts.length > 0 ? "/" + parts.join("/") : "/";
				}
				resolve({
					path: remotePath,
					entries,
					parentPath
				});
			};
			if (remotePath === "/" || remotePath === "") sftp.readdir(".", listCallback);
			else sftp.readdir(remotePath, listCallback);
		});
	}
	formatPermissions(mode) {
		return (mode & 511).toString(8).padStart(3, "0");
	}
	async createDirectory(sessionId, remotePath) {
		const sftp = await this.getSFTP(sessionId);
		return new Promise((resolve, reject) => {
			sftp.mkdir(remotePath, (err) => {
				if (err) reject(err);
				else resolve();
			});
		});
	}
	async removeFile(sessionId, remotePath) {
		const sftp = await this.getSFTP(sessionId);
		return new Promise((resolve, reject) => {
			sftp.unlink(remotePath, (err) => {
				if (err) reject(err);
				else resolve();
			});
		});
	}
	async removeDirectory(sessionId, remotePath) {
		const sftp = await this.getSFTP(sessionId);
		return new Promise((resolve, reject) => {
			sftp.rmdir(remotePath, (err) => {
				if (err) reject(err);
				else resolve();
			});
		});
	}
	async rename(sessionId, oldPath, newPath) {
		const sftp = await this.getSFTP(sessionId);
		return new Promise((resolve, reject) => {
			sftp.rename(oldPath, newPath, (err) => {
				if (err) reject(err);
				else resolve();
			});
		});
	}
	async readFile(sessionId, remotePath) {
		const sftp = await this.getSFTP(sessionId);
		return new Promise((resolve, reject) => {
			const chunks = [];
			const readStream = sftp.createReadStream(remotePath);
			readStream.on("data", (chunk) => {
				chunks.push(chunk);
			});
			readStream.on("end", () => {
				resolve(Buffer.concat(chunks));
			});
			readStream.on("error", (err) => {
				reject(err);
			});
		});
	}
	async writeFile(sessionId, remotePath, content) {
		const sftp = await this.getSFTP(sessionId);
		return new Promise((resolve, reject) => {
			const writeStream = sftp.createWriteStream(remotePath);
			writeStream.on("close", () => {
				resolve();
			});
			writeStream.on("error", (err) => {
				reject(err);
			});
			writeStream.write(content);
			writeStream.end();
		});
	}
	async chmod(sessionId, remotePath, mode) {
		const sftp = await this.getSFTP(sessionId);
		return new Promise((resolve, reject) => {
			sftp.chmod(remotePath, mode, (err) => {
				if (err) reject(err);
				else resolve();
			});
		});
	}
	async execCommand(sessionId, command) {
		const client = this.sessions.get(sessionId);
		if (!client) throw new Error("Session not found");
		return new Promise((resolve, reject) => {
			client.exec(command, (err, stream) => {
				if (err) {
					reject(err);
					return;
				}
				let output = "";
				let errorOutput = "";
				stream.on("data", (data) => {
					output += data.toString();
				});
				stream.stderr.on("data", (data) => {
					errorOutput += data.toString();
				});
				stream.on("close", (code) => {
					if (code !== 0 && errorOutput) reject(new Error(errorOutput));
					else resolve(output);
				});
			});
		});
	}
	async getSystemStats(sessionId) {
		const output = await this.execCommand(sessionId, `
      echo "@@@SECTION:CPU@@@"; top -bn1 | head -n 5;
      echo "@@@SECTION:MEM@@@"; free -b;
      echo "@@@SECTION:DISK@@@"; df -B1 -x tmpfs -x devtmpfs;
      echo "@@@SECTION:NET@@@"; cat /proc/net/dev;
      echo "@@@SECTION:PROC@@@"; ps aux --sort=-%cpu | head -n 21 | awk 'BEGIN {print "PID|USER|CPU|MEM|VSZ|RSS|STAT|START|TIME|COMMAND"} NR>1 {printf "%s|%s|%s|%s|%s|%s|%s|%s|%s|%s\n", $2, $1, $3, $4, $5, $6, $8, $9, $10, substr($0, index($0,$11))}' ;
      echo "@@@SECTION:SYS@@@"; hostname; uptime -p; uname -r; nproc; awk '{print $1" "$2" "$3}' /proc/loadavg
    `);
		return this.parseSystemStats(output);
	}
	parseSystemStats(output) {
		const sections = output.split("@@@SECTION:");
		let cpuPercent = 0;
		let memory = {
			total: 0,
			used: 0,
			free: 0,
			cached: 0
		};
		let disks = [];
		let network = {
			rxBytes: 0,
			txBytes: 0
		};
		let processes = [];
		let system = {
			hostname: "Unknown",
			uptime: "",
			loadAverage: [
				0,
				0,
				0
			],
			cpuCores: 1,
			kernelVersion: "",
			totalMemory: 0
		};
		for (const section of sections) {
			const trimmed = section.trim();
			if (!trimmed) continue;
			if (trimmed.startsWith("CPU@@@") || trimmed.startsWith("CPU")) cpuPercent = this.parseCPU(trimmed);
			else if (trimmed.startsWith("MEM@@@") || trimmed.startsWith("MEM")) {
				memory = this.parseMemory(trimmed);
				system.totalMemory = memory.total;
			} else if (trimmed.startsWith("DISK@@@") || trimmed.startsWith("DISK")) disks = this.parseDisks(trimmed);
			else if (trimmed.startsWith("NET@@@") || trimmed.startsWith("NET")) network = this.parseNetwork(trimmed);
			else if (trimmed.startsWith("PROC@@@") || trimmed.startsWith("PROC")) processes = this.parseProcesses(trimmed);
			else if (trimmed.startsWith("SYS@@@") || trimmed.startsWith("SYS")) system = this.parseSystem(trimmed);
		}
		return {
			cpuPercent,
			memory,
			disks,
			network,
			processes,
			system
		};
	}
	parseCPU(output) {
		const lines = output.split("\n");
		for (const line of lines) if (line.includes("Cpu(s)") || line.includes("CPU")) {
			const parts = line.split(",");
			for (const part of parts) if (part.includes("id")) {
				const idleStr = part.trim().split(/\s+/)[0];
				return Math.max(0, 100 - parseFloat(idleStr));
			}
		}
		return 0;
	}
	parseMemory(output) {
		const lines = output.split("\n");
		for (const line of lines) if (line.startsWith("Mem:")) {
			const parts = line.split(/\s+/);
			return {
				total: parseInt(parts[1]) || 0,
				used: parseInt(parts[2]) || 0,
				free: parseInt(parts[3]) || 0,
				cached: parseInt(parts[5]) || 0
			};
		}
		return {
			total: 0,
			used: 0,
			free: 0,
			cached: 0
		};
	}
	parseDisks(output) {
		const disks = [];
		const lines = output.split("\n");
		for (let i = 1; i < lines.length; i++) {
			const line = lines[i].trim();
			if (!line) continue;
			const parts = line.split(/\s+/);
			if (parts.length >= 6) disks.push({
				name: parts[0],
				total: parseInt(parts[1]) || 0,
				used: parseInt(parts[2]) || 0,
				mountPoint: parts[5]
			});
		}
		return disks;
	}
	parseNetwork(output) {
		let rxBytes = 0;
		let txBytes = 0;
		const lines = output.split("\n");
		for (const line of lines) if (line.includes(":")) {
			const parts = line.split(":")[1]?.trim().split(/\s+/) || [];
			if (parts.length >= 9) {
				rxBytes += parseInt(parts[0]) || 0;
				txBytes += parseInt(parts[8]) || 0;
			}
		}
		return {
			rxBytes,
			txBytes
		};
	}
	parseProcesses(output) {
		const processes = [];
		const lines = output.split("\n");
		for (const line of lines) {
			if (line.startsWith("PID|") || !line.includes("|")) continue;
			const parts = line.split("|");
			if (parts.length >= 10) {
				const status = parts[6].trim();
				processes.push({
					pid: parseInt(parts[0]) || 0,
					name: this.extractProcessName(parts[9]),
					cpu: parseFloat(parts[2]) || 0,
					memory: parseFloat(parts[3]) || 0,
					user: parts[1].trim(),
					status,
					statusDesc: this.getStatusDescription(status),
					startTime: parts[7].trim(),
					elapsedTime: parts[8].trim(),
					vsz: parseInt(parts[4]) || 0,
					rss: parseInt(parts[5]) || 0,
					command: parts[9].trim()
				});
			}
		}
		return processes;
	}
	parseSystem(output) {
		const lines = output.split("\n").filter((l) => l.trim());
		return {
			hostname: lines[0]?.trim() || "Unknown",
			uptime: lines[1]?.trim() || "",
			kernelVersion: lines[2]?.trim() || "",
			cpuCores: parseInt(lines[3]) || 1,
			loadAverage: [
				parseFloat(lines[4]?.split(" ")[0]) || 0,
				parseFloat(lines[4]?.split(" ")[1]) || 0,
				parseFloat(lines[4]?.split(" ")[2]) || 0
			],
			totalMemory: 0
		};
	}
	extractProcessName(command) {
		return command.split(/\s+/)[0]?.split("/").pop() || "unknown";
	}
	getStatusDescription(status) {
		return {
			R: "运行中",
			S: "睡眠中",
			D: "不可中断睡眠",
			Z: "僵尸进程",
			T: "已停止",
			t: "追踪停止",
			W: "内存分页",
			X: "死亡",
			K: "内核线程",
			P: "暂停"
		}[status.charAt(0)] || "未知";
	}
	async killProcess(sessionId, pid, signal = 15) {
		await this.execCommand(sessionId, `kill -${signal} ${pid}`);
	}
	async detectDatabases(sessionId) {
		const output = await this.execCommand(sessionId, `
      check_service() {
        if command -v systemctl >/dev/null 2>&1; then
          systemctl is-active $1 2>/dev/null
        else
          service $1 status 2>/dev/null | grep -q running && echo "active" || echo "inactive"
        fi
      }

      # MySQL
      (command -v mysql >/dev/null 2>&1 && echo "mysql|true|$(mysql --version 2>/dev/null)|$(check_service mysql)|$(which mysql 2>/dev/null)") || echo "mysql|false|||"
      # PostgreSQL
      (command -v psql >/dev/null 2>&1 && echo "postgresql|true|$(psql --version 2>/dev/null)|$(check_service postgresql)|$(which psql 2>/dev/null)") || echo "postgresql|false|||"
      # Redis
      (command -v redis-server >/dev/null 2>&1 && echo "redis|true|$(redis-server --version 2>/dev/null)|$(check_service redis-server)|$(which redis-server 2>/dev/null)") || echo "redis|false|||"
      # MongoDB
      (command -v mongod >/dev/null 2>&1 && echo "mongodb|true|$(mongod --version 2>/dev/null)|$(check_service mongod)|$(which mongod 2>/dev/null)") || echo "mongodb|false|||"
      # MariaDB
      (command -v mariadb >/dev/null 2>&1 && echo "mariadb|true|$(mariadb --version 2>/dev/null)|$(check_service mariadb)|$(which mariadb 2>/dev/null)") || echo "mariadb|false|||"
    `);
		return this.parseDatabaseDetection(output);
	}
	parseDatabaseDetection(output) {
		const results = [];
		const lines = output.split("\n");
		for (const line of lines) {
			const parts = line.split("|");
			if (parts.length < 5) continue;
			const [type, installed, version, status] = parts;
			if (installed === "true") results.push({
				dbType: type,
				installed: true,
				version: version?.replace(/[()]/g, "").trim(),
				status: status?.includes("active") ? "running" : "stopped",
				port: this.getDefaultPort(type)
			});
			else results.push({
				dbType: type,
				installed: false
			});
		}
		return results;
	}
	getDefaultPort(type) {
		return {
			mysql: 3306,
			postgresql: 5432,
			redis: 6379,
			mongodb: 27017,
			mariadb: 3306
		}[type] || 0;
	}
	async installDatabase(params) {
		const { sessionId, dbType, options } = params;
		return await this.execCommand(sessionId, `echo "Installing ${dbType}..."`);
	}
	async manageDatabaseService(params) {
		const { sessionId, serviceName, action } = params;
		const cmd = `sudo systemctl ${action} ${serviceName}`;
		return await this.execCommand(sessionId, cmd);
	}
	async getDatabaseConfig(params) {
		const { sessionId, dbType } = params;
		const path = {
			mysql: "/etc/mysql/my.cnf",
			postgresql: "/etc/postgresql/*/main/postgresql.conf",
			redis: "/etc/redis/redis.conf",
			mongodb: "/etc/mongod.conf"
		}[dbType] || "/etc/my.cnf";
		return await this.execCommand(sessionId, `cat ${path} 2>/dev/null | head -500`);
	}
	async updateDatabaseConfig(params) {
		const { sessionId, dbType, configContent } = params;
		return await this.execCommand(sessionId, `echo "Updating ${dbType} config..."`);
	}
	async getDatabases(params) {
		const { sessionId, dbType } = params;
		let cmd = "";
		switch (dbType) {
			case "mysql":
			case "mariadb":
				cmd = `mysql -u root -e "SELECT schema_name FROM information_schema.schemata WHERE schema_name NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');" 2>/dev/null`;
				break;
			case "postgresql":
				cmd = `sudo -u postgres psql -c "SELECT datname FROM pg_database WHERE datistemplate = false AND datname NOT IN ('postgres');" 2>/dev/null`;
				break;
			case "mongodb":
				cmd = `mongosh --quiet --eval "db.adminCommand('listDatabases').databases.map(d => d.name).join('\\n')" 2>/dev/null`;
				break;
			case "redis":
				cmd = `redis-cli INFO server 2>/dev/null | grep -E "redis_version|tcp_port"`;
				break;
		}
		if (!cmd) return [];
		await this.execCommand(sessionId, cmd);
		return [];
	}
	async createDatabase(params) {
		const { sessionId, dbType, name, username, password, charset } = params;
		switch (dbType) {
			case "mysql":
			case "mariadb":
				await this.execCommand(sessionId, `mysql -u root -e "CREATE DATABASE IF NOT EXISTS \\\`${name}\\\` CHARACTER SET ${charset || "utf8mb4"};"`);
				break;
			case "postgresql":
				await this.execCommand(sessionId, `sudo -u postgres psql -c "CREATE DATABASE \\"${name}\\";"`);
				break;
		}
	}
	async changeDatabasePassword(params) {
		const { sessionId, dbType, username, newPassword } = params;
		switch (dbType) {
			case "mysql":
			case "mariadb":
				await this.execCommand(sessionId, `mysql -u root -e "ALTER USER '${username}'@'localhost' IDENTIFIED BY '${newPassword}';"`);
				break;
			case "postgresql":
				await this.execCommand(sessionId, `sudo -u postgres psql -c "ALTER USER \\"${username}\\" WITH PASSWORD '${newPassword}';"`);
				break;
		}
	}
	async updateDatabase(params) {}
	async deleteDatabase(params) {
		const { sessionId, dbType, dbId, username } = params;
		switch (dbType) {
			case "mysql":
			case "mariadb":
				await this.execCommand(sessionId, `mysql -u root -e "DROP DATABASE IF EXISTS \\\`${dbId}\\\`; DROP USER IF EXISTS '${username}'@'localhost';"`);
				break;
			case "postgresql":
				await this.execCommand(sessionId, `sudo -u postgres psql -c "DROP DATABASE IF EXISTS \\"${dbId}\\";"`);
				break;
		}
	}
};
//#endregion
//#region node_modules/.pnpm/uuid@11.1.0/node_modules/uuid/dist/esm/stringify.js
var byteToHex = [];
for (let i = 0; i < 256; ++i) byteToHex.push((i + 256).toString(16).slice(1));
function unsafeStringify(arr, offset = 0) {
	return (byteToHex[arr[offset + 0]] + byteToHex[arr[offset + 1]] + byteToHex[arr[offset + 2]] + byteToHex[arr[offset + 3]] + "-" + byteToHex[arr[offset + 4]] + byteToHex[arr[offset + 5]] + "-" + byteToHex[arr[offset + 6]] + byteToHex[arr[offset + 7]] + "-" + byteToHex[arr[offset + 8]] + byteToHex[arr[offset + 9]] + "-" + byteToHex[arr[offset + 10]] + byteToHex[arr[offset + 11]] + byteToHex[arr[offset + 12]] + byteToHex[arr[offset + 13]] + byteToHex[arr[offset + 14]] + byteToHex[arr[offset + 15]]).toLowerCase();
}
//#endregion
//#region node_modules/.pnpm/uuid@11.1.0/node_modules/uuid/dist/esm/rng.js
var rnds8Pool = new Uint8Array(256);
var poolPtr = rnds8Pool.length;
function rng() {
	if (poolPtr > rnds8Pool.length - 16) {
		randomFillSync(rnds8Pool);
		poolPtr = 0;
	}
	return rnds8Pool.slice(poolPtr, poolPtr += 16);
}
//#endregion
//#region node_modules/.pnpm/uuid@11.1.0/node_modules/uuid/dist/esm/native.js
var native_default = { randomUUID };
//#endregion
//#region node_modules/.pnpm/uuid@11.1.0/node_modules/uuid/dist/esm/v4.js
function v4(options, buf, offset) {
	if (native_default.randomUUID && !buf && !options) return native_default.randomUUID();
	options = options || {};
	const rnds = options.random ?? options.rng?.() ?? rng();
	if (rnds.length < 16) throw new Error("Random bytes length must be >= 16");
	rnds[6] = rnds[6] & 15 | 64;
	rnds[8] = rnds[8] & 63 | 128;
	if (buf) {
		offset = offset || 0;
		if (offset < 0 || offset + 16 > buf.length) throw new RangeError(`UUID byte range ${offset}:${offset + 15} is out of buffer bounds`);
		for (let i = 0; i < 16; ++i) buf[offset + i] = rnds[i];
		return buf;
	}
	return unsafeStringify(rnds);
}
//#endregion
//#region electron/services/machine-db.ts
var Database = createRequire(import.meta.url)("better-sqlite3");
var MachineDatabase = class {
	db = null;
	dbPath;
	inMemoryMachines = [];
	useMemory = false;
	constructor() {
		const homeDir = os.homedir();
		const dataDir = path.join(homeDir, ".synapsh");
		if (!fs.existsSync(dataDir)) fs.mkdirSync(dataDir, { recursive: true });
		this.dbPath = path.join(dataDir, "synapsh.db");
	}
	async initialize() {
		try {
			this.db = new Database(this.dbPath);
			this.db.pragma("journal_mode = WAL");
			this.db.exec(`
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
        );

        CREATE TABLE IF NOT EXISTS settings (
          key TEXT PRIMARY KEY,
          value TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
      `);
			console.log("Machine database initialized");
		} catch (error) {
			console.error("Failed to initialize database, using in-memory fallback:", error);
			this.db = null;
			this.inMemoryMachines = [];
			this.useMemory = true;
		}
	}
	async listMachines() {
		if (this.useMemory) return this.inMemoryMachines;
		if (!this.db) throw new Error("Database not initialized");
		return this.db.prepare("SELECT * FROM machines ORDER BY created_at DESC").all().map((row) => ({
			id: row.id,
			name: row.name,
			host: row.host,
			port: row.port,
			username: row.username,
			password: row.password,
			privateKeyPath: row.private_key_path,
			authType: row.auth_type,
			tags: row.tags,
			os: row.os,
			createdAt: row.created_at,
			updatedAt: row.updated_at
		}));
	}
	async addMachine(input) {
		const id = v4();
		const now = (/* @__PURE__ */ new Date()).toISOString();
		const name = input.name || input.host;
		const tags = JSON.stringify(input.tags || []);
		const os = input.os || "linux";
		const port = input.port || 22;
		const machine = {
			id,
			name,
			host: input.host,
			port,
			username: input.username,
			password: input.password,
			privateKeyPath: input.privateKeyPath,
			authType: input.authType,
			tags,
			os,
			createdAt: now,
			updatedAt: now
		};
		if (this.useMemory) {
			this.inMemoryMachines.push(machine);
			return machine;
		}
		if (!this.db) throw new Error("Database not initialized");
		this.db.prepare(`
      INSERT INTO machines (id, name, host, port, username, password, private_key_path, auth_type, tags, os, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(id, name, input.host, port, input.username, input.password || null, input.privateKeyPath || null, input.authType, tags, os, now, now);
		return machine;
	}
	async updateMachine(id, input) {
		const now = (/* @__PURE__ */ new Date()).toISOString();
		const name = input.name || input.host;
		const tags = JSON.stringify(input.tags || []);
		const os = input.os || "linux";
		const port = input.port || 22;
		if (this.useMemory) {
			const index = this.inMemoryMachines.findIndex((m) => m.id === id);
			if (index === -1) throw new Error("Machine not found");
			const updated = {
				id,
				name,
				host: input.host,
				port,
				username: input.username,
				password: input.password,
				privateKeyPath: input.privateKeyPath,
				authType: input.authType,
				tags,
				os,
				createdAt: this.inMemoryMachines[index].createdAt,
				updatedAt: now
			};
			this.inMemoryMachines[index] = updated;
			return updated;
		}
		if (!this.db) throw new Error("Database not initialized");
		this.db.prepare(`
      UPDATE machines 
      SET name = ?, host = ?, port = ?, username = ?, password = ?, 
          private_key_path = ?, auth_type = ?, tags = ?, os = ?, updated_at = ?
      WHERE id = ?
    `).run(name, input.host, port, input.username, input.password || null, input.privateKeyPath || null, input.authType, tags, os, now, id);
		const row = this.db.prepare("SELECT * FROM machines WHERE id = ?").get(id);
		return {
			id: row.id,
			name: row.name,
			host: row.host,
			port: row.port,
			username: row.username,
			password: row.password,
			privateKeyPath: row.private_key_path,
			authType: row.auth_type,
			tags: row.tags,
			os: row.os,
			createdAt: row.created_at,
			updatedAt: row.updated_at
		};
	}
	async deleteMachine(id) {
		if (this.useMemory) {
			this.inMemoryMachines = this.inMemoryMachines.filter((m) => m.id !== id);
			return;
		}
		if (!this.db) throw new Error("Database not initialized");
		this.db.prepare("DELETE FROM machines WHERE id = ?").run(id);
	}
	async getMachine(id) {
		if (this.useMemory) return this.inMemoryMachines.find((m) => m.id === id) || null;
		if (!this.db) throw new Error("Database not initialized");
		const row = this.db.prepare("SELECT * FROM machines WHERE id = ?").get(id);
		if (!row) return null;
		return {
			id: row.id,
			name: row.name,
			host: row.host,
			port: row.port,
			username: row.username,
			password: row.password,
			privateKeyPath: row.private_key_path,
			authType: row.auth_type,
			tags: row.tags,
			os: row.os,
			createdAt: row.created_at,
			updatedAt: row.updated_at
		};
	}
	async getSetting(key, defaultValue = null) {
		if (this.useMemory) return defaultValue;
		if (!this.db) return defaultValue;
		try {
			const row = this.db.prepare("SELECT value FROM settings WHERE key = ?").get(key);
			if (!row) return defaultValue;
			return JSON.parse(row.value);
		} catch (e) {
			console.error(`Failed to get setting ${key}:`, e);
			return defaultValue;
		}
	}
	async setSetting(key, value) {
		if (this.useMemory) return;
		if (!this.db) throw new Error("Database not initialized");
		const now = (/* @__PURE__ */ new Date()).toISOString();
		const valueStr = JSON.stringify(value);
		try {
			this.db.prepare(`
        INSERT INTO settings (key, value, updated_at)
        VALUES (?, ?, ?)
        ON CONFLICT(key) DO UPDATE SET
          value = excluded.value,
          updated_at = excluded.updated_at
      `).run(key, valueStr, now);
		} catch (e) {
			console.error(`Failed to set setting ${key}:`, e);
			throw e;
		}
	}
	close() {
		if (this.db) {
			this.db.close();
			this.db = null;
		}
	}
};
//#endregion
//#region electron/services/browser.ts
var BrowserManager = class {
	browserProcesses = /* @__PURE__ */ new Map();
	proxySidecars = /* @__PURE__ */ new Map();
	proxyPorts = /* @__PURE__ */ new Map();
	async ensureProxyPort(sessionId, sshManager) {
		if (this.proxyPorts.has(sessionId)) return this.proxyPorts.get(sessionId);
		const port = await this.reservePort();
		this.proxyPorts.set(sessionId, port);
		return port;
	}
	async reservePort() {
		return new Promise((resolve) => {
			const server = __require("net").createServer();
			server.listen(0, () => {
				const address = server.address();
				if (address && typeof address === "object") resolve(address.port);
				else resolve(0);
				server.close();
			});
			server.on("error", () => {
				resolve(0);
			});
		});
	}
	async openBrowser(sessionId, url, options, sshManager) {
		const profileMode = options?.profileMode || "session";
		const isNewWindow = profileMode === "new";
		const proxyPort = await this.ensureProxyPort(sessionId, sshManager);
		const profileDir = this.getChromeProfileDir(sessionId, profileMode);
		if (!fs.existsSync(profileDir)) fs.mkdirSync(profileDir, { recursive: true });
		let chromePath;
		if (process.platform === "darwin") chromePath = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
		else if (process.platform === "win32") chromePath = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
		else chromePath = "/usr/bin/google-chrome";
		const chromeArgs = [
			`--proxy-server=socks5://127.0.0.1:${proxyPort}`,
			`--user-data-dir=${profileDir}`,
			"--disable-quic",
			"--disable-features=VizDisplayCompositor,OptimizationGuideModelDownloading,OptimizationHintsFetching,AutofillServerCommunication,MediaRouter",
			"--disable-background-networking",
			"--disable-default-apps",
			"--disable-component-update",
			"--disable-domain-reliability",
			"--disable-client-side-phishing-detection",
			"--safebrowsing-disable-auto-update",
			"--disable-extensions",
			"--disable-sync",
			"--disable-translate",
			"--no-first-run",
			"--no-default-browser-check"
		];
		if (isNewWindow) chromeArgs.push("--new-window");
		chromeArgs.push(url);
		const child = spawn(chromePath, chromeArgs, {
			stdio: "ignore",
			detached: true
		});
		const pid = child.pid;
		if (pid) {
			this.browserProcesses.set(sessionId, {
				pid,
				port: proxyPort,
				child
			});
			child.on("exit", () => {
				this.browserProcesses.delete(sessionId);
			});
		}
		child.unref();
	}
	getChromeProfileDir(sessionId, profileMode) {
		const baseDir = path.join(os.tmpdir(), "synapsh-chrome", this.sanitizeSessionId(sessionId));
		if (profileMode === "new") return path.join(baseDir, `profile-${Date.now()}`);
		return path.join(baseDir, "profile");
	}
	sanitizeSessionId(sessionId) {
		return sessionId.replace(/[^a-zA-Z0-9-_]/g, "_");
	}
	getProxyPort(sessionId) {
		return this.proxyPorts.get(sessionId) || null;
	}
	closeBrowser(sessionId) {
		const process = this.browserProcesses.get(sessionId);
		if (process) {
			try {
				process.child.kill();
			} catch (e) {}
			this.browserProcesses.delete(sessionId);
		}
	}
	closeAll() {
		for (const [sessionId] of this.browserProcesses) this.closeBrowser(sessionId);
		this.browserProcesses.clear();
		this.proxyPorts.clear();
	}
	closeProxy(sessionId) {
		const sidecars = this.proxySidecars.get(sessionId);
		if (sidecars) {
			for (const sidecar of sidecars) try {
				sidecar.child.kill();
			} catch (e) {}
			this.proxySidecars.delete(sessionId);
		}
		this.proxyPorts.delete(sessionId);
	}
};
//#endregion
//#region electron/main.ts
var __dirname = dirname(fileURLToPath(import.meta.url));
var mainWindow = null;
var sshManager = new SSHSessionManager();
var machineDb = new MachineDatabase();
var browserManager = new BrowserManager();
function createWindow() {
	mainWindow = new BrowserWindow({
		width: 1200,
		height: 800,
		minWidth: 800,
		minHeight: 600,
		frame: true,
		titleBarStyle: "hiddenInset",
		backgroundColor: "#0b0d10",
		webPreferences: {
			preload: path.join(__dirname, "preload.mjs"),
			contextIsolation: true,
			nodeIntegration: false,
			sandbox: false
		}
	});
	if (process.env.VITE_DEV_SERVER_URL) {
		mainWindow.loadURL(process.env.VITE_DEV_SERVER_URL);
		mainWindow.webContents.openDevTools();
	} else mainWindow.loadFile(path.join(__dirname, "../dist/index.html"));
	mainWindow.on("closed", () => {
		mainWindow = null;
	});
	createMenu();
}
function createMenu() {
	const menu = Menu.buildFromTemplate([
		{
			label: "SynapSH",
			submenu: [
				{
					label: "关于 SynapSH",
					role: "about"
				},
				{ type: "separator" },
				{
					label: "偏好设置",
					accelerator: "CmdOrCtrl+,",
					click: () => {}
				},
				{ type: "separator" },
				{
					label: "隐藏 SynapSH",
					accelerator: "CmdOrCtrl+H",
					role: "hide"
				},
				{
					label: "隐藏其他",
					accelerator: "CmdOrCtrl+Alt+H",
					role: "hideOthers"
				},
				{
					label: "显示全部",
					role: "unhide"
				},
				{ type: "separator" },
				{
					label: "退出 SynapSH",
					accelerator: "CmdOrCtrl+Q",
					role: "quit"
				}
			]
		},
		{
			label: "编辑",
			submenu: [
				{
					label: "撤销",
					accelerator: "CmdOrCtrl+Z",
					role: "undo"
				},
				{
					label: "重做",
					accelerator: "Shift+CmdOrCtrl+Z",
					role: "redo"
				},
				{ type: "separator" },
				{
					label: "剪切",
					accelerator: "CmdOrCtrl+X",
					role: "cut"
				},
				{
					label: "复制",
					accelerator: "CmdOrCtrl+C",
					role: "copy"
				},
				{
					label: "粘贴",
					accelerator: "CmdOrCtrl+V",
					role: "paste"
				},
				{
					label: "全选",
					accelerator: "CmdOrCtrl+A",
					role: "selectAll"
				}
			]
		},
		{
			label: "窗口",
			submenu: [{
				label: "最小化",
				accelerator: "CmdOrCtrl+M",
				role: "minimize"
			}, {
				label: "关闭",
				accelerator: "CmdOrCtrl+W",
				role: "close"
			}]
		},
		{
			label: "帮助",
			submenu: [{
				label: "文档",
				click: () => shell.openExternal("https://github.com/synapsh")
			}]
		}
	]);
	Menu.setApplicationMenu(menu);
}
function registerIPCHandlers() {
	ipcMain.handle("ssh:connect", async (_event, sessionId, params) => {
		try {
			await sshManager.connect(sessionId, params);
			sshManager.onData(sessionId, (data) => {
				if (mainWindow) mainWindow.webContents.send("ssh:data", sessionId, data.toString("base64"));
			});
			return { success: true };
		} catch (error) {
			return {
				success: false,
				error: String(error)
			};
		}
	});
	ipcMain.handle("ssh:disconnect", async (_event, sessionId) => {
		await sshManager.disconnect(sessionId);
		return { success: true };
	});
	ipcMain.handle("ssh:write", async (_event, sessionId, data) => {
		const buffer = Buffer.from(data, "utf-8");
		await sshManager.write(sessionId, buffer);
		return { success: true };
	});
	ipcMain.handle("ssh:resize", async (_event, sessionId, cols, rows) => {
		await sshManager.resize(sessionId, cols, rows);
		return { success: true };
	});
	ipcMain.handle("ssh:test", async (_event, params) => {
		try {
			return await sshManager.testConnection(params);
		} catch (error) {
			return false;
		}
	});
	ipcMain.handle("fs:list", async (_event, sessionId, remotePath) => {
		return await sshManager.listDirectory(sessionId, remotePath);
	});
	ipcMain.handle("fs:mkdir", async (_event, sessionId, remotePath) => {
		await sshManager.createDirectory(sessionId, remotePath);
		return { success: true };
	});
	ipcMain.handle("fs:delete", async (_event, sessionId, remotePath, isDirectory) => {
		if (isDirectory) await sshManager.removeDirectory(sessionId, remotePath);
		else await sshManager.removeFile(sessionId, remotePath);
		return { success: true };
	});
	ipcMain.handle("fs:rename", async (_event, sessionId, oldPath, newPath) => {
		await sshManager.rename(sessionId, oldPath, newPath);
		return { success: true };
	});
	ipcMain.handle("fs:download", async (_event, sessionId, remotePath) => {
		return (await sshManager.readFile(sessionId, remotePath)).toString("base64");
	});
	ipcMain.handle("fs:upload", async (_event, sessionId, remotePath, base64Content) => {
		const content = Buffer.from(base64Content, "base64");
		await sshManager.writeFile(sessionId, remotePath, content);
		return { success: true };
	});
	ipcMain.handle("fs:create", async (_event, sessionId, remotePath, content) => {
		const data = content ? Buffer.from(content, "utf-8") : Buffer.alloc(0);
		await sshManager.writeFile(sessionId, remotePath, data);
		return { success: true };
	});
	ipcMain.handle("fs:chmod", async (_event, sessionId, remotePath, mode) => {
		await sshManager.chmod(sessionId, remotePath, mode);
		return { success: true };
	});
	ipcMain.handle("monitor:stats", async (_event, sessionId) => {
		return await sshManager.getSystemStats(sessionId);
	});
	ipcMain.handle("monitor:kill", async (_event, sessionId, pid, signal) => {
		await sshManager.killProcess(sessionId, pid, signal);
		return { success: true };
	});
	ipcMain.handle("db:list-machines", async () => {
		return await machineDb.listMachines();
	});
	ipcMain.handle("db:add-machine", async (_event, input) => {
		return await machineDb.addMachine(input);
	});
	ipcMain.handle("db:update-machine", async (_event, id, input) => {
		return await machineDb.updateMachine(id, input);
	});
	ipcMain.handle("db:delete-machine", async (_event, id) => {
		return { success: true };
	});
	ipcMain.handle("db:get-setting", async (_event, key, defaultValue) => {
		return await machineDb.getSetting(key, defaultValue);
	});
	ipcMain.handle("db:set-setting", async (_event, key, value) => {
		await machineDb.setSetting(key, value);
		return { success: true };
	});
	ipcMain.handle("db:detect-databases", async (_event, sessionId) => {
		return await sshManager.detectDatabases(sessionId);
	});
	ipcMain.handle("db:install-database", async (_event, params) => {
		return await sshManager.installDatabase(params);
	});
	ipcMain.handle("db:manage-service", async (_event, params) => {
		return await sshManager.manageDatabaseService(params);
	});
	ipcMain.handle("db:get-config", async (_event, params) => {
		return await sshManager.getDatabaseConfig(params);
	});
	ipcMain.handle("db:update-config", async (_event, params) => {
		return await sshManager.updateDatabaseConfig(params);
	});
	ipcMain.handle("db:get-databases", async (_event, params) => {
		return await sshManager.getDatabases(params);
	});
	ipcMain.handle("db:create-database", async (_event, params) => {
		return await sshManager.createDatabase(params);
	});
	ipcMain.handle("db:change-password", async (_event, params) => {
		return await sshManager.changeDatabasePassword(params);
	});
	ipcMain.handle("db:update-database", async (_event, params) => {
		return await sshManager.updateDatabase(params);
	});
	ipcMain.handle("db:delete-database", async (_event, params) => {
		return await sshManager.deleteDatabase(params);
	});
	ipcMain.handle("browser:open", async (_event, sessionId, url, options) => {
		try {
			await browserManager.openBrowser(sessionId, url, options, sshManager);
			return { success: true };
		} catch (error) {
			if (mainWindow) mainWindow.webContents.send("browser:proxy-error", {
				sessionId,
				host: new URL(url).hostname,
				port: 443,
				message: String(error)
			});
			return {
				success: false,
				error: String(error)
			};
		}
	});
	ipcMain.handle("browser:get-proxy-port", async (_event, sessionId) => {
		return browserManager.getProxyPort(sessionId);
	});
	ipcMain.on("window:minimize", () => {
		mainWindow?.minimize();
	});
	ipcMain.on("window:maximize", () => {
		if (mainWindow?.isMaximized()) mainWindow.unmaximize();
		else mainWindow?.maximize();
	});
	ipcMain.on("window:close", () => {
		mainWindow?.close();
	});
}
app.whenReady().then(async () => {
	await machineDb.initialize();
	registerIPCHandlers();
	createWindow();
	app.on("activate", () => {
		if (BrowserWindow.getAllWindows().length === 0) createWindow();
	});
});
app.on("window-all-closed", () => {
	if (process.platform !== "darwin") app.quit();
});
app.on("before-quit", async () => {
	await sshManager.disconnectAll();
	browserManager.closeAll();
});
//#endregion
