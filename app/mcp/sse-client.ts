// SSE 客户端，用于 Tauri 环境（不依赖 Node.js）
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { SSEClientTransport } from "@modelcontextprotocol/sdk/client/sse.js";
import { ListToolsResponse, McpRequestMessage, ServerConfig } from "./types";
import { z } from "zod";

export type SSEClient = Client;

export async function createSSEClient(
  id: string,
  config: ServerConfig,
): Promise<SSEClient> {
  if (!config.url) {
    throw new Error(`SSE transport requires 'url' in config for server ${id}`);
  }

  const url = new URL(config.url);
  
  // 构建 requestInit，包含 headers
  const requestInit: RequestInit = {};
  if (config.headers && Object.keys(config.headers).length > 0) {
    requestInit.headers = new Headers();
    for (const [key, value] of Object.entries(config.headers)) {
      requestInit.headers.set(key, value);
    }
  }
  
  const transport = new SSEClientTransport(url, {
    requestInit,
  });

  const client = new Client(
    {
      name: `nextchat-mcp-client-${id}`,
      version: "1.0.0",
    },
    {
      capabilities: {},
    },
  );
  await client.connect(transport);
  return client;
}

export async function removeSSEClient(client: SSEClient): Promise<void> {
  await client.close();
}

export async function listSSETools(client: SSEClient): Promise<ListToolsResponse> {
  return client.listTools();
}

export async function executeSSERequest(
  client: SSEClient,
  request: McpRequestMessage,
  schema?: z.ZodTypeAny,
): Promise<any> {
  return (client.request as any)(request, schema ?? z.any());
}

