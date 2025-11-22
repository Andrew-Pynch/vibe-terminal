import { ProviderKind } from "@agent-hub/protocol";
export interface AgentHubConfig {
    host: string;
    httpPort: number;
    wsPort: number;
    protocol: "http" | "https";
    sharedSecret?: string;
    promptProfileDir: string;
    defaultProvider: ProviderKind;
    defaultModel: string;
}
export declare function resolveConfig(overrides?: Partial<AgentHubConfig>): AgentHubConfig;
