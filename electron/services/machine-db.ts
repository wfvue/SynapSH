// 本地机器数据库管理 - 使用 better-sqlite3

import Database from 'better-sqlite3';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import { v4 as uuidv4 } from 'uuid';

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

export class MachineDatabase {
  private db: Database.Database | null = null;
  private dbPath: string;
  private inMemoryMachines: Machine[] = [];
  private useMemory = false;

  constructor() {
    const homeDir = os.homedir();
    const dataDir = path.join(homeDir, '.synapsh');
    
    // 确保目录存在
    if (!fs.existsSync(dataDir)) {
      fs.mkdirSync(dataDir, { recursive: true });
    }
    
    this.dbPath = path.join(dataDir, 'synapsh.db');
  }

  // 初始化数据库
  async initialize(): Promise<void> {
    try {
      this.db = new Database(this.dbPath);
      this.db.pragma('journal_mode = WAL');
      
      // 创建表
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
        )
      `);
      
      console.log('Machine database initialized');
    } catch (error) {
      console.error('Failed to initialize database, using in-memory fallback:', error);
      // 使用内存存储作为后备
      this.db = null;
      this.inMemoryMachines = [];
      this.useMemory = true;
    }
  }

  // 获取所有机器
  async listMachines(): Promise<Machine[]> {
    if (this.useMemory) {
      return this.inMemoryMachines;
    }
    if (!this.db) throw new Error('Database not initialized');
    
    const rows = this.db.prepare('SELECT * FROM machines ORDER BY created_at DESC').all() as any[];
    
    return rows.map(row => ({
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
      updatedAt: row.updated_at,
    }));
  }

  // 添加机器
  async addMachine(input: MachineInput): Promise<Machine> {
    const id = uuidv4();
    const now = new Date().toISOString();
    const name = input.name || input.host;
    const tags = JSON.stringify(input.tags || []);
    const os = input.os || 'linux';
    const port = input.port || 22;
    
    const machine: Machine = {
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
      updatedAt: now,
    };
    
    if (this.useMemory) {
      this.inMemoryMachines.push(machine);
      return machine;
    }
    
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare(`
      INSERT INTO machines (id, name, host, port, username, password, private_key_path, auth_type, tags, os, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `);
    
    stmt.run(
      id,
      name,
      input.host,
      port,
      input.username,
      input.password || null,
      input.privateKeyPath || null,
      input.authType,
      tags,
      os,
      now,
      now
    );
    
    return machine;
  }

  // 更新机器
  async updateMachine(id: string, input: MachineInput): Promise<Machine> {
    const now = new Date().toISOString();
    const name = input.name || input.host;
    const tags = JSON.stringify(input.tags || []);
    const os = input.os || 'linux';
    const port = input.port || 22;
    
    if (this.useMemory) {
      const index = this.inMemoryMachines.findIndex(m => m.id === id);
      if (index === -1) throw new Error('Machine not found');
      
      const updated: Machine = {
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
        updatedAt: now,
      };
      this.inMemoryMachines[index] = updated;
      return updated;
    }
    
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare(`
      UPDATE machines 
      SET name = ?, host = ?, port = ?, username = ?, password = ?, 
          private_key_path = ?, auth_type = ?, tags = ?, os = ?, updated_at = ?
      WHERE id = ?
    `);
    
    stmt.run(
      name,
      input.host,
      port,
      input.username,
      input.password || null,
      input.privateKeyPath || null,
      input.authType,
      tags,
      os,
      now,
      id
    );
    
    // 返回更新后的记录
    const row = this.db.prepare('SELECT * FROM machines WHERE id = ?').get(id) as any;
    
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
      updatedAt: row.updated_at,
    };
  }

  // 删除机器
  async deleteMachine(id: string): Promise<void> {
    if (this.useMemory) {
      this.inMemoryMachines = this.inMemoryMachines.filter(m => m.id !== id);
      return;
    }
    
    if (!this.db) throw new Error('Database not initialized');
    
    const stmt = this.db.prepare('DELETE FROM machines WHERE id = ?');
    stmt.run(id);
  }

  // 获取单个机器
  async getMachine(id: string): Promise<Machine | null> {
    if (this.useMemory) {
      return this.inMemoryMachines.find(m => m.id === id) || null;
    }
    
    if (!this.db) throw new Error('Database not initialized');
    
    const row = this.db.prepare('SELECT * FROM machines WHERE id = ?').get(id) as any;
    
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
      updatedAt: row.updated_at,
    };
  }

  // 关闭数据库
  close(): void {
    if (this.db) {
      this.db.close();
      this.db = null;
    }
  }
}
