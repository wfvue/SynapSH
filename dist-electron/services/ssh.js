"use strict";
// SSH 会话管理器 - 使用 ssh2 库
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
exports.SSHSessionManager = void 0;
const ssh2 = __importStar(require("ssh2"));
// SFTP 文件类型常量 (来自 ssh2)
const S_IFMT = 0o170000;
const S_IFREG = 0o100000;
const S_IFDIR = 0o040000;
const S_IFLNK = 0o120000;
class SSHSessionManager {
    sessions = new Map();
    dataCallbacks = new Map();
    sftpSessions = new Map();
    shellChannels = new Map();
    // 连接 SSH
    async connect(sessionId, params) {
        return new Promise((resolve, reject) => {
            const client = new ssh2.Client();
            const config = {
                host: params.host,
                port: params.port,
                username: params.username,
                readyTimeout: 30000,
                keepaliveInterval: 20000,
                keepaliveCountMax: 6,
            };
            if (params.privateKey) {
                config.privateKey = params.privateKey;
            }
            else if (params.password) {
                config.password = params.password;
            }
            client.on('ready', () => {
                console.log(`SSH connected: ${sessionId}`);
                this.sessions.set(sessionId, client);
                // 打开交互式 shell
                client.shell((err, stream) => {
                    if (err) {
                        reject(err);
                        return;
                    }
                    this.shellChannels.set(sessionId, stream);
                    stream.on('data', (data) => {
                        const callbacks = this.dataCallbacks.get(sessionId) || [];
                        callbacks.forEach(cb => cb(data));
                    });
                    stream.on('close', () => {
                        console.log(`Shell closed: ${sessionId}`);
                    });
                    resolve();
                });
            });
            client.on('error', (err) => {
                console.error(`SSH error: ${sessionId}`, err);
                reject(err);
            });
            client.on('close', () => {
                console.log(`SSH disconnected: ${sessionId}`);
                this.cleanup(sessionId);
            });
            client.connect(config);
        });
    }
    // 断开连接
    async disconnect(sessionId) {
        const client = this.sessions.get(sessionId);
        if (client) {
            client.end();
            this.cleanup(sessionId);
        }
    }
    // 断开所有连接
    async disconnectAll() {
        for (const sessionId of this.sessions.keys()) {
            await this.disconnect(sessionId);
        }
    }
    // 清理会话资源
    cleanup(sessionId) {
        this.sessions.delete(sessionId);
        this.dataCallbacks.delete(sessionId);
        this.sftpSessions.delete(sessionId);
        this.shellChannels.delete(sessionId);
    }
    // 注册数据回调
    onData(sessionId, callback) {
        const callbacks = this.dataCallbacks.get(sessionId) || [];
        callbacks.push(callback);
        this.dataCallbacks.set(sessionId, callbacks);
    }
    // 写入数据
    async write(sessionId, data) {
        const stream = this.shellChannels.get(sessionId);
        if (stream) {
            stream.write(data);
        }
    }
    // 调整终端大小
    async resize(sessionId, cols, rows) {
        const stream = this.shellChannels.get(sessionId);
        if (stream) {
            stream.setWindow(rows, cols, 0, 0);
        }
    }
    // 测试连接
    async testConnection(params) {
        return new Promise((resolve) => {
            const client = new ssh2.Client();
            const config = {
                host: params.host,
                port: params.port,
                username: params.username,
                readyTimeout: 10000,
            };
            if (params.privateKey) {
                config.privateKey = params.privateKey;
            }
            else if (params.password) {
                config.password = params.password;
            }
            client.on('ready', () => {
                client.end();
                resolve(true);
            });
            client.on('error', () => {
                resolve(false);
            });
            client.connect(config);
        });
    }
    // 获取 SFTP 会话
    async getSFTP(sessionId) {
        if (this.sftpSessions.has(sessionId)) {
            return this.sftpSessions.get(sessionId);
        }
        const client = this.sessions.get(sessionId);
        if (!client) {
            throw new Error('Session not found');
        }
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
    // 列出目录
    async listDirectory(sessionId, remotePath) {
        const sftp = await this.getSFTP(sessionId);
        return new Promise((resolve, reject) => {
            const entries = [];
            // 使用正确的回调类型 - ssh2.SFTPWrapper.readdir 期望 FileEntry[]
            const listCallback = (err, list) => {
                if (err) {
                    reject(err);
                    return;
                }
                for (const file of list) {
                    const attrs = file.attrs;
                    // 使用 mode 常量判断文件类型
                    const mode = attrs.mode;
                    const isDirectory = (mode & S_IFMT) === S_IFDIR;
                    const isSymlink = (mode & S_IFMT) === S_IFLNK;
                    const isFile = (mode & S_IFMT) === S_IFREG;
                    let fileType;
                    if (isDirectory) {
                        fileType = 'directory';
                    }
                    else if (isSymlink) {
                        fileType = 'symlink';
                    }
                    else if (isFile) {
                        fileType = 'file';
                    }
                    else {
                        fileType = 'unknown';
                    }
                    const permissions = this.formatPermissions(mode);
                    entries.push({
                        name: file.filename,
                        path: `${remotePath.replace(/\/$/, '')}/${file.filename}`,
                        type: fileType,
                        size: attrs.size,
                        modifiedTime: attrs.mtime ? new Date(attrs.mtime * 1000).toISOString() : undefined,
                        createdTime: attrs.atime ? new Date(attrs.atime * 1000).toISOString() : undefined,
                        permissions,
                        owner: String(attrs.uid),
                        group: String(attrs.gid),
                        isHidden: file.filename.startsWith('.'),
                    });
                }
                // 排序：目录优先，然后按名称排序
                entries.sort((a, b) => {
                    if (a.type === 'directory' && b.type !== 'directory')
                        return -1;
                    if (a.type !== 'directory' && b.type === 'directory')
                        return 1;
                    return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
                });
                // 计算父路径
                let parentPath;
                if (remotePath !== '/' && remotePath !== '') {
                    const parts = remotePath.split('/').filter(Boolean);
                    parts.pop();
                    parentPath = parts.length > 0 ? '/' + parts.join('/') : '/';
                }
                resolve({
                    path: remotePath,
                    entries,
                    parentPath,
                });
            };
            if (remotePath === '/' || remotePath === '') {
                sftp.readdir('.', listCallback);
            }
            else {
                sftp.readdir(remotePath, listCallback);
            }
        });
    }
    // 格式化权限
    formatPermissions(mode) {
        const perms = (mode & 0o777).toString(8);
        return perms.padStart(3, '0');
    }
    // 创建目录
    async createDirectory(sessionId, remotePath) {
        const sftp = await this.getSFTP(sessionId);
        return new Promise((resolve, reject) => {
            sftp.mkdir(remotePath, (err) => {
                if (err)
                    reject(err);
                else
                    resolve();
            });
        });
    }
    // 删除文件
    async removeFile(sessionId, remotePath) {
        const sftp = await this.getSFTP(sessionId);
        return new Promise((resolve, reject) => {
            sftp.unlink(remotePath, (err) => {
                if (err)
                    reject(err);
                else
                    resolve();
            });
        });
    }
    // 删除目录
    async removeDirectory(sessionId, remotePath) {
        const sftp = await this.getSFTP(sessionId);
        return new Promise((resolve, reject) => {
            sftp.rmdir(remotePath, (err) => {
                if (err)
                    reject(err);
                else
                    resolve();
            });
        });
    }
    // 重命名/移动
    async rename(sessionId, oldPath, newPath) {
        const sftp = await this.getSFTP(sessionId);
        return new Promise((resolve, reject) => {
            sftp.rename(oldPath, newPath, (err) => {
                if (err)
                    reject(err);
                else
                    resolve();
            });
        });
    }
    // 下载文件
    async readFile(sessionId, remotePath) {
        const sftp = await this.getSFTP(sessionId);
        return new Promise((resolve, reject) => {
            const chunks = [];
            const readStream = sftp.createReadStream(remotePath);
            readStream.on('data', (chunk) => {
                chunks.push(chunk);
            });
            readStream.on('end', () => {
                resolve(Buffer.concat(chunks));
            });
            readStream.on('error', (err) => {
                reject(err);
            });
        });
    }
    // 上传文件
    async writeFile(sessionId, remotePath, content) {
        const sftp = await this.getSFTP(sessionId);
        return new Promise((resolve, reject) => {
            const writeStream = sftp.createWriteStream(remotePath);
            writeStream.on('close', () => {
                resolve();
            });
            writeStream.on('error', (err) => {
                reject(err);
            });
            writeStream.write(content);
            writeStream.end();
        });
    }
    // 修改权限
    async chmod(sessionId, remotePath, mode) {
        const sftp = await this.getSFTP(sessionId);
        return new Promise((resolve, reject) => {
            sftp.chmod(remotePath, mode, (err) => {
                if (err)
                    reject(err);
                else
                    resolve();
            });
        });
    }
    // 执行命令
    async execCommand(sessionId, command) {
        const client = this.sessions.get(sessionId);
        if (!client) {
            throw new Error('Session not found');
        }
        return new Promise((resolve, reject) => {
            client.exec(command, (err, stream) => {
                if (err) {
                    reject(err);
                    return;
                }
                let output = '';
                let errorOutput = '';
                stream.on('data', (data) => {
                    output += data.toString();
                });
                stream.stderr.on('data', (data) => {
                    errorOutput += data.toString();
                });
                stream.on('close', (code) => {
                    if (code !== 0 && errorOutput) {
                        reject(new Error(errorOutput));
                    }
                    else {
                        resolve(output);
                    }
                });
            });
        });
    }
    // 获取系统统计
    async getSystemStats(sessionId) {
        const cmd = `
      echo "@@@SECTION:CPU@@@"; top -bn1 | head -n 5;
      echo "@@@SECTION:MEM@@@"; free -b;
      echo "@@@SECTION:DISK@@@"; df -B1 -x tmpfs -x devtmpfs;
      echo "@@@SECTION:NET@@@"; cat /proc/net/dev;
      echo "@@@SECTION:PROC@@@"; ps aux --sort=-%cpu | head -n 21 | awk 'BEGIN {print "PID|USER|CPU|MEM|VSZ|RSS|STAT|START|TIME|COMMAND"} NR>1 {printf "%s|%s|%s|%s|%s|%s|%s|%s|%s|%s\n", $2, $1, $3, $4, $5, $6, $8, $9, $10, substr($0, index($0,$11))}' ;
      echo "@@@SECTION:SYS@@@"; hostname; uptime -p; uname -r; nproc; awk '{print $1" "$2" "$3}' /proc/loadavg
    `;
        const output = await this.execCommand(sessionId, cmd);
        return this.parseSystemStats(output);
    }
    // 解析系统统计
    parseSystemStats(output) {
        const sections = output.split('@@@SECTION:');
        let cpuPercent = 0;
        let memory = { total: 0, used: 0, free: 0, cached: 0 };
        let disks = [];
        let network = { rxBytes: 0, txBytes: 0 };
        let processes = [];
        let system = {
            hostname: 'Unknown',
            uptime: '',
            loadAverage: [0, 0, 0],
            cpuCores: 1,
            kernelVersion: '',
            totalMemory: 0,
        };
        for (const section of sections) {
            const trimmed = section.trim();
            if (!trimmed)
                continue;
            if (trimmed.startsWith('CPU@@@') || trimmed.startsWith('CPU')) {
                cpuPercent = this.parseCPU(trimmed);
            }
            else if (trimmed.startsWith('MEM@@@') || trimmed.startsWith('MEM')) {
                memory = this.parseMemory(trimmed);
                system.totalMemory = memory.total;
            }
            else if (trimmed.startsWith('DISK@@@') || trimmed.startsWith('DISK')) {
                disks = this.parseDisks(trimmed);
            }
            else if (trimmed.startsWith('NET@@@') || trimmed.startsWith('NET')) {
                network = this.parseNetwork(trimmed);
            }
            else if (trimmed.startsWith('PROC@@@') || trimmed.startsWith('PROC')) {
                processes = this.parseProcesses(trimmed);
            }
            else if (trimmed.startsWith('SYS@@@') || trimmed.startsWith('SYS')) {
                system = this.parseSystem(trimmed);
            }
        }
        return { cpuPercent, memory, disks, network, processes, system };
    }
    parseCPU(output) {
        const lines = output.split('\n');
        for (const line of lines) {
            if (line.includes('Cpu(s)') || line.includes('CPU')) {
                const parts = line.split(',');
                for (const part of parts) {
                    if (part.includes('id')) {
                        const idleStr = part.trim().split(/\s+/)[0];
                        const idle = parseFloat(idleStr);
                        return Math.max(0, 100 - idle);
                    }
                }
            }
        }
        return 0;
    }
    parseMemory(output) {
        const lines = output.split('\n');
        for (const line of lines) {
            if (line.startsWith('Mem:')) {
                const parts = line.split(/\s+/);
                return {
                    total: parseInt(parts[1]) || 0,
                    used: parseInt(parts[2]) || 0,
                    free: parseInt(parts[3]) || 0,
                    cached: parseInt(parts[5]) || 0,
                };
            }
        }
        return { total: 0, used: 0, free: 0, cached: 0 };
    }
    parseDisks(output) {
        const disks = [];
        const lines = output.split('\n');
        for (let i = 1; i < lines.length; i++) {
            const line = lines[i].trim();
            if (!line)
                continue;
            const parts = line.split(/\s+/);
            if (parts.length >= 6) {
                disks.push({
                    name: parts[0],
                    total: parseInt(parts[1]) || 0,
                    used: parseInt(parts[2]) || 0,
                    mountPoint: parts[5],
                });
            }
        }
        return disks;
    }
    parseNetwork(output) {
        let rxBytes = 0;
        let txBytes = 0;
        const lines = output.split('\n');
        for (const line of lines) {
            if (line.includes(':')) {
                const parts = line.split(':')[1]?.trim().split(/\s+/) || [];
                if (parts.length >= 9) {
                    rxBytes += parseInt(parts[0]) || 0;
                    txBytes += parseInt(parts[8]) || 0;
                }
            }
        }
        return { rxBytes, txBytes };
    }
    parseProcesses(output) {
        const processes = [];
        const lines = output.split('\n');
        for (const line of lines) {
            if (line.startsWith('PID|') || !line.includes('|'))
                continue;
            const parts = line.split('|');
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
                    command: parts[9].trim(),
                });
            }
        }
        return processes;
    }
    parseSystem(output) {
        const lines = output.split('\n').filter(l => l.trim());
        return {
            hostname: lines[0]?.trim() || 'Unknown',
            uptime: lines[1]?.trim() || '',
            kernelVersion: lines[2]?.trim() || '',
            cpuCores: parseInt(lines[3]) || 1,
            loadAverage: [
                parseFloat(lines[4]?.split(' ')[0]) || 0,
                parseFloat(lines[4]?.split(' ')[1]) || 0,
                parseFloat(lines[4]?.split(' ')[2]) || 0,
            ],
            totalMemory: 0, // 会在 parseSystemStats 中被 memory.total 更新
        };
    }
    extractProcessName(command) {
        const first = command.split(/\s+/)[0];
        return first?.split('/').pop() || 'unknown';
    }
    getStatusDescription(status) {
        const first = status.charAt(0);
        const descriptions = {
            'R': '运行中',
            'S': '睡眠中',
            'D': '不可中断睡眠',
            'Z': '僵尸进程',
            'T': '已停止',
            't': '追踪停止',
            'W': '内存分页',
            'X': '死亡',
            'K': '内核线程',
            'P': '暂停',
        };
        return descriptions[first] || '未知';
    }
    // 终止进程
    async killProcess(sessionId, pid, signal = 15) {
        await this.execCommand(sessionId, `kill -${signal} ${pid}`);
    }
    // ==================== 数据库管理 ====================
    // 检测数据库
    async detectDatabases(sessionId) {
        const script = `
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
    `;
        const output = await this.execCommand(sessionId, script);
        return this.parseDatabaseDetection(output);
    }
    parseDatabaseDetection(output) {
        const results = [];
        const lines = output.split('\n');
        for (const line of lines) {
            const parts = line.split('|');
            if (parts.length < 5)
                continue;
            const [type, installed, version, status] = parts;
            if (installed === 'true') {
                results.push({
                    dbType: type,
                    installed: true,
                    version: version?.replace(/[\(\)]/g, '').trim(),
                    status: status?.includes('active') ? 'running' : 'stopped',
                    port: this.getDefaultPort(type),
                });
            }
            else {
                results.push({
                    dbType: type,
                    installed: false,
                });
            }
        }
        return results;
    }
    getDefaultPort(type) {
        const ports = {
            mysql: 3306,
            postgresql: 5432,
            redis: 6379,
            mongodb: 27017,
            mariadb: 3306,
        };
        return ports[type] || 0;
    }
    // 安装数据库
    async installDatabase(params) {
        const { sessionId, dbType, options } = params;
        // 根据数据库类型生成安装脚本
        // 这里简化处理，实际需要更复杂的逻辑
        return await this.execCommand(sessionId, `echo "Installing ${dbType}..."`);
    }
    // 管理数据库服务
    async manageDatabaseService(params) {
        const { sessionId, serviceName, action } = params;
        const cmd = `sudo systemctl ${action} ${serviceName}`;
        return await this.execCommand(sessionId, cmd);
    }
    // 获取数据库配置
    async getDatabaseConfig(params) {
        const { sessionId, dbType } = params;
        const configPaths = {
            mysql: '/etc/mysql/my.cnf',
            postgresql: '/etc/postgresql/*/main/postgresql.conf',
            redis: '/etc/redis/redis.conf',
            mongodb: '/etc/mongod.conf',
        };
        const path = configPaths[dbType] || '/etc/my.cnf';
        return await this.execCommand(sessionId, `cat ${path} 2>/dev/null | head -500`);
    }
    // 更新数据库配置
    async updateDatabaseConfig(params) {
        const { sessionId, dbType, configContent } = params;
        // 实际实现需要处理 base64 编码和 sudo 权限
        return await this.execCommand(sessionId, `echo "Updating ${dbType} config..."`);
    }
    // 获取数据库列表
    async getDatabases(params) {
        const { sessionId, dbType } = params;
        let cmd = '';
        switch (dbType) {
            case 'mysql':
            case 'mariadb':
                cmd = `mysql -u root -e "SELECT schema_name FROM information_schema.schemata WHERE schema_name NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');" 2>/dev/null`;
                break;
            case 'postgresql':
                cmd = `sudo -u postgres psql -c "SELECT datname FROM pg_database WHERE datistemplate = false AND datname NOT IN ('postgres');" 2>/dev/null`;
                break;
            case 'mongodb':
                cmd = `mongosh --quiet --eval "db.adminCommand('listDatabases').databases.map(d => d.name).join('\\n')" 2>/dev/null`;
                break;
            case 'redis':
                cmd = `redis-cli INFO server 2>/dev/null | grep -E "redis_version|tcp_port"`;
                break;
        }
        if (!cmd)
            return [];
        const output = await this.execCommand(sessionId, cmd);
        // 解析输出为 DatabaseInfo[]
        // 简化处理
        return [];
    }
    // 创建数据库
    async createDatabase(params) {
        const { sessionId, dbType, name, username, password, charset } = params;
        switch (dbType) {
            case 'mysql':
            case 'mariadb':
                await this.execCommand(sessionId, `mysql -u root -e "CREATE DATABASE IF NOT EXISTS \\\`${name}\\\` CHARACTER SET ${charset || 'utf8mb4'};"`);
                break;
            case 'postgresql':
                await this.execCommand(sessionId, `sudo -u postgres psql -c "CREATE DATABASE \\"${name}\\";"`);
                break;
        }
    }
    // 修改密码
    async changeDatabasePassword(params) {
        const { sessionId, dbType, username, newPassword } = params;
        switch (dbType) {
            case 'mysql':
            case 'mariadb':
                await this.execCommand(sessionId, `mysql -u root -e "ALTER USER '${username}'@'localhost' IDENTIFIED BY '${newPassword}';"`);
                break;
            case 'postgresql':
                await this.execCommand(sessionId, `sudo -u postgres psql -c "ALTER USER \\"${username}\\" WITH PASSWORD '${newPassword}';"`);
                break;
        }
    }
    // 更新数据库
    async updateDatabase(params) {
        // 实现更新数据库备注等功能
    }
    // 删除数据库
    async deleteDatabase(params) {
        const { sessionId, dbType, dbId, username } = params;
        switch (dbType) {
            case 'mysql':
            case 'mariadb':
                await this.execCommand(sessionId, `mysql -u root -e "DROP DATABASE IF EXISTS \\\`${dbId}\\\`; DROP USER IF EXISTS '${username}'@'localhost';"`);
                break;
            case 'postgresql':
                await this.execCommand(sessionId, `sudo -u postgres psql -c "DROP DATABASE IF EXISTS \\"${dbId}\\";"`);
                break;
        }
    }
}
exports.SSHSessionManager = SSHSessionManager;
//# sourceMappingURL=ssh.js.map