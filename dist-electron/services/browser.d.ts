import { SSHSessionManager } from './ssh';
interface BrowserLaunchOptions {
    profileMode?: string;
}
export declare class BrowserManager {
    private browserProcesses;
    private proxySidecars;
    private proxyPorts;
    ensureProxyPort(sessionId: string, sshManager: SSHSessionManager): Promise<number>;
    private reservePort;
    openBrowser(sessionId: string, url: string, options: BrowserLaunchOptions | undefined, sshManager: SSHSessionManager): Promise<void>;
    private getChromeProfileDir;
    private sanitizeSessionId;
    getProxyPort(sessionId: string): number | null;
    closeBrowser(sessionId: string): void;
    closeAll(): void;
    closeProxy(sessionId: string): void;
}
export {};
