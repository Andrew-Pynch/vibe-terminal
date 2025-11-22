function readEnv(key) {
    if (typeof process === "undefined" || !process.env) {
        return undefined;
    }
    return process.env[key] ?? process.env[`EXPO_PUBLIC_${key}`];
}
const DEFAULTS = {
    host: readEnv("AGENT_HUB_HOST") || "127.0.0.1",
    httpPort: Number(readEnv("AGENT_HUB_HTTP_PORT") || 42001),
    wsPort: Number(readEnv("AGENT_HUB_WS_PORT") || 42002),
    protocol: readEnv("AGENT_HUB_PROTOCOL") || "http",
    sharedSecret: readEnv("AGENT_HUB_SECRET"),
    promptProfileDir: readEnv("AGENT_HUB_PROMPT_PROFILE_DIR") || "prompts/profiles",
    defaultProvider: readEnv("AGENT_HUB_PROVIDER") || "dummy",
    defaultModel: readEnv("AGENT_HUB_MODEL") || "dummy-orchestrator"
};
export function resolveConfig(overrides = {}) {
    return {
        ...DEFAULTS,
        ...overrides
    };
}
