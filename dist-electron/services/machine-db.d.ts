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
export declare class MachineDatabase {
    private db;
    private dbPath;
    private inMemoryMachines;
    private useMemory;
    constructor();
    initialize(): Promise<void>;
    listMachines(): Promise<Machine[]>;
    addMachine(input: MachineInput): Promise<Machine>;
    updateMachine(id: string, input: MachineInput): Promise<Machine>;
    deleteMachine(id: string): Promise<void>;
    getMachine(id: string): Promise<Machine | null>;
    close(): void;
}
export {};
