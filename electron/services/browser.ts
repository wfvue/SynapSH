// 浏览器代理管理器

import { spawn, ChildProcess } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import { SSHSessionManager } from './ssh';

interface BrowserProcess {
  pid: number;
  port: number;
  child: ChildProcess;
}

interface ProxySidecar {
  pid: number;
  port: number;
  child: ChildProcess;
}

interface BrowserLaunchOptions {
  profileMode?: string;
}

export class BrowserManager {
  private browserProcesses: Map<string, BrowserProcess> = new Map();
  private proxySidecars: Map<string, ProxySidecar[]> = new Map();
  private proxyPorts: Map<string, number> = new Map();

  // 获取或创建 SOCKS5 代理端口
  async ensureProxyPort(sessionId: string, sshManager: SSHSessionManager): Promise<number> {
    if (this.proxyPorts.has(sessionId)) {
      return this.proxyPorts.get(sessionId)!;
    }

    // 通过 SSH 端口转发创建 SOCKS5 代理
    // 这里简化处理，实际需要更复杂的逻辑
    // 可以使用 ssh -D 来创建动态端口转发

    const port = await this.reservePort();
    this.proxyPorts.set(sessionId, port);

    return port;
  }

  // 预留本地端口
  private async reservePort(): Promise<number> {
    return new Promise((resolve) => {
      const net = require('net');
      const server = net.createServer();

      server.listen(0, () => {
        const address = server.address();
        if (address && typeof address === 'object') {
          resolve(address.port);
        } else {
          resolve(0);
        }
        server.close();
      });

      server.on('error', () => {
        resolve(0);
      });
    });
  }

  // 打开浏览器
  async openBrowser(
    sessionId: string,
    url: string,
    options: BrowserLaunchOptions | undefined,
    sshManager: SSHSessionManager
  ): Promise<void> {
    const profileMode = options?.profileMode || 'session';
    const isNewWindow = profileMode === 'new';

    // 获取代理端口
    const proxyPort = await this.ensureProxyPort(sessionId, sshManager);

    // 获取或创建 Chrome 配置目录
    const profileDir = this.getChromeProfileDir(sessionId, profileMode);

    if (!fs.existsSync(profileDir)) {
      fs.mkdirSync(profileDir, { recursive: true });
    }

    // 确定 Chrome 路径
    let chromePath: string;
    if (process.platform === 'darwin') {
      chromePath = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
    } else if (process.platform === 'win32') {
      chromePath = 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe';
    } else {
      chromePath = '/usr/bin/google-chrome';
    }

    // 构建 Chrome 参数
    const chromeArgs = [
      `--proxy-server=socks5://127.0.0.1:${proxyPort}`,
      `--user-data-dir=${profileDir}`,
      '--disable-quic',
      '--disable-features=VizDisplayCompositor,OptimizationGuideModelDownloading,OptimizationHintsFetching,AutofillServerCommunication,MediaRouter',
      '--disable-background-networking',
      '--disable-default-apps',
      '--disable-component-update',
      '--disable-domain-reliability',
      '--disable-client-side-phishing-detection',
      '--safebrowsing-disable-auto-update',
      '--disable-extensions',
      '--disable-sync',
      '--disable-translate',
      '--no-first-run',
      '--no-default-browser-check',
    ];

    if (isNewWindow) {
      chromeArgs.push('--new-window');
    }

    chromeArgs.push(url);

    // 启动 Chrome
    const child = spawn(chromePath, chromeArgs, {
      stdio: 'ignore',
      detached: true,
    });

    const pid = child.pid;

    if (pid) {
      this.browserProcesses.set(sessionId, {
        pid,
        port: proxyPort,
        child,
      });

      child.on('exit', () => {
        this.browserProcesses.delete(sessionId);
      });
    }

    child.unref();
  }

  // 获取 Chrome 配置目录
  private getChromeProfileDir(sessionId: string, profileMode: string): string {
    const baseDir = path.join(os.tmpdir(), 'synapsh-chrome', this.sanitizeSessionId(sessionId));

    if (profileMode === 'new') {
      return path.join(baseDir, `profile-${Date.now()}`);
    }

    return path.join(baseDir, 'profile');
  }

  // 清理 session ID 用于文件名
  private sanitizeSessionId(sessionId: string): string {
    return sessionId.replace(/[^a-zA-Z0-9-_]/g, '_');
  }

  // 获取代理端口
  getProxyPort(sessionId: string): number | null {
    return this.proxyPorts.get(sessionId) || null;
  }

  // 关闭浏览器
  closeBrowser(sessionId: string): void {
    const process = this.browserProcesses.get(sessionId);
    if (process) {
      try {
        process.child.kill();
      } catch (e) {
        // 忽略错误
      }
      this.browserProcesses.delete(sessionId);
    }
  }

  // 关闭所有浏览器
  closeAll(): void {
    for (const [sessionId] of this.browserProcesses) {
      this.closeBrowser(sessionId);
    }
    this.browserProcesses.clear();
    this.proxyPorts.clear();
  }

  // 关闭代理
  closeProxy(sessionId: string): void {
    const sidecars = this.proxySidecars.get(sessionId);
    if (sidecars) {
      for (const sidecar of sidecars) {
        try {
          sidecar.child.kill();
        } catch (e) {
          // 忽略错误
        }
      }
      this.proxySidecars.delete(sessionId);
    }
    this.proxyPorts.delete(sessionId);
  }
}
