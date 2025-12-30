import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import { SSEClientTransport } from "@modelcontextprotocol/sdk/client/sse.js";
import { MCPClientLogger } from "./logger";
import { ListToolsResponse, McpRequestMessage, ServerConfig } from "./types";
import { z } from "zod";

const logger = new MCPClientLogger();

// 统一的客户端接口
export type McpClient = Client;

export async function createClient(
  id: string,
  config: ServerConfig,
): Promise<McpClient> {
  const transportType = config.type || "stdio";
  logger.info(`Creating ${transportType} client for ${id}...`);

  if (transportType === "sse") {
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
  } else {
    // stdio 传输
    if (!config.command) {
      throw new Error(`Stdio transport requires 'command' in config for server ${id}`);
    }

    const transport = new StdioClientTransport({
      command: config.command,
      args: config.args || [],
      env: {
        ...Object.fromEntries(
          Object.entries(process.env)
            .filter(([_, v]) => v !== undefined)
            .map(([k, v]) => [k, v as string]),
        ),
        ...(config.env || {}),
      },
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
}

export async function removeClient(client: McpClient) {
  logger.info(`Removing client...`);
  await client.close();
}

export async function listTools(client: McpClient): Promise<ListToolsResponse> {
  return client.listTools();
}

export async function executeRequest(
  client: McpClient,
  request: McpRequestMessage,
  schema?: z.ZodTypeAny,
): Promise<any> {
  return (client.request as any)(request, schema ?? z.any());
}
