#!/usr/bin/env node
import { Command } from "commander";
import WebSocket from "ws";
import readline from "node:readline";
import {
  CreateSessionRequest,
  SessionDetailResponse,
  SessionListResponse,
  ServerToClientWsMessage
} from "@agent-hub/protocol";
import { AgentHubConfig, resolveConfig } from "@agent-hub/config";
import { fetch } from "undici";

const program = new Command();
program.name("agent-hub").description("Interact with the local Agent Hub server");

interface JsonRequestInit {
  method?: string;
  headers?: Record<string, string>;
  body?: string;
}

function baseHttpUrl(config: AgentHubConfig) {
  return `${config.protocol}://${config.host}:${config.httpPort}`;
}

function baseWsUrl(config: AgentHubConfig) {
  const protocol = config.protocol === "https" ? "wss" : "ws";
  return `${protocol}://${config.host}:${config.wsPort}`;
}

async function httpRequest<T>(
  config: AgentHubConfig,
  path: string,
  init?: JsonRequestInit
): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(init?.headers || {})
  };
  if (config.sharedSecret) {
    headers["X-Agent-Hub-Auth"] = config.sharedSecret;
  }

  const response = await fetch(`${baseHttpUrl(config)}${path}`, {
    ...init,
    headers
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`HTTP ${response.status}: ${text}`);
  }

  return (await response.json()) as T;
}

const sessionsCommand = program.command("sessions").description("Manage sessions");

sessionsCommand
  .command("list")
  .description("List sessions on the server")
  .action(async () => {
    const config = resolveConfig();
    const data = await httpRequest<SessionListResponse>(config, "/sessions");
    if (data.sessions.length === 0) {
      console.log("No sessions found.");
      return;
    }
    for (const session of data.sessions) {
      console.log(
        `${session.id} | ${session.name} | profile=${session.profile} | provider=${session.llmConfig.provider}`
      );
    }
  });

sessionsCommand
  .command("new")
  .description("Create a new session")
  .requiredOption("-n, --name <name>", "Human friendly name")
  .requiredOption("-p, --profile <profile>", "Profile id to load")
  .option("--provider <provider>", "LLM provider override")
  .option("--model <model>", "LLM model override")
  .action(async (options) => {
    const config = resolveConfig();
    const payload: CreateSessionRequest = {
      name: options.name,
      profile: options.profile,
      llmConfig: {},
      meta: {}
    };
    if (options.provider) {
      payload.llmConfig = payload.llmConfig || {};
      payload.llmConfig.provider = options.provider;
    }
    if (options.model) {
      payload.llmConfig = payload.llmConfig || {};
      payload.llmConfig.model = options.model;
    }

    const response = await httpRequest<SessionDetailResponse>(config, "/sessions", {
      method: "POST",
      body: JSON.stringify(payload)
    });

    console.log(`Session created: ${response.session.id} (${response.session.name})`);
  });

sessionsCommand
  .command("attach")
  .description("Attach to a running session")
  .argument("<sessionId>", "Session identifier")
  .action(async (sessionId) => {
    const config = resolveConfig();
    const wsUrl = `${baseWsUrl(config)}/sessions`;
    const headers: Record<string, string> = {};
    if (config.sharedSecret) {
      headers["X-Agent-Hub-Auth"] = config.sharedSecret;
    }

    console.log(`Connecting to ${wsUrl}...`);
    const ws = new WebSocket(wsUrl, { headers });

    ws.on("open", () => {
      const joinMessage = {
        type: "JoinSession",
        sessionId
      };
      ws.send(JSON.stringify(joinMessage));
      console.log(`Joined session ${sessionId}. Type /exit to leave.`);
      promptLoop(ws, sessionId);
    });

    ws.on("message", (raw) => {
      try {
        const payload = JSON.parse(raw.toString()) as ServerToClientWsMessage;
        handleWsMessage(payload);
      } catch (error) {
        console.error("Failed to parse message:", error);
      }
    });

    ws.on("error", (err) => {
      console.error("WebSocket error:", err);
    });

    ws.on("close", () => {
      console.log("Connection closed.");
      process.exit(0);
    });
  });

function promptLoop(ws: WebSocket, sessionId: string) {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
  });

  rl.on("line", (line) => {
    if (line.trim() === "/exit") {
      rl.close();
      ws.close();
      return;
    }
    const userMessage = {
      type: "UserMessage",
      sessionId,
      content: line.trim()
    };
    ws.send(JSON.stringify(userMessage));
  });

  rl.on("SIGINT", () => {
    rl.close();
    ws.close();
  });
}

function handleWsMessage(message: ServerToClientWsMessage) {
  switch (message.type) {
    case "SessionJoined":
      console.log(`Session acknowledged: ${message.sessionId}`);
      break;
    case "AssistantMessageStart":
      console.log(`\n[assistant:${message.messageId}]`);
      break;
    case "AssistantMessageChunk":
      process.stdout.write(message.textChunk);
      break;
    case "AssistantMessageComplete":
      process.stdout.write("\n");
      break;
    case "SessionUpdated":
      console.log(
        `\nSession updated: ${message.session.id} (${message.session.name})`
      );
      break;
    case "Error":
      console.error(`Error (${message.code}): ${message.message}`);
      break;
    default:
      console.log("Unknown message", message);
  }
}

program.parseAsync();
