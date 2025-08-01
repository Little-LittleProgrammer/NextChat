// 客户端兼容版本 - 用于静态导出模式
import {
  McpRequestMessage,
  ServerStatusResponse,
  McpConfigData,
  ServerConfig,
} from "./types";

// 所有函数返回空实现，用于静态导出模式
export async function getClientsStatus(): Promise<
  Record<string, ServerStatusResponse>
> {
  return {};
}

export async function startServer(clientId: string): Promise<void> {
  // 空实现
}

export async function stopServer(clientId: string): Promise<void> {
  // 空实现
}

export async function pauseServer(clientId: string): Promise<void> {
  // 空实现
}

export async function resumeServer(clientId: string): Promise<void> {
  // 空实现
}

export async function updateServerConfig(config: McpConfigData): Promise<void> {
  // 空实现
}

export async function executeMcpAction(
  request: McpRequestMessage,
): Promise<any> {
  throw new Error("MCP actions are not available in static export mode");
}

export async function getAllTools(): Promise<any[]> {
  return [];
}

export async function isMcpEnabled(): Promise<boolean> {
  return false;
}

export async function getAvailableClientsCount(): Promise<number> {
  return 0;
}

export async function initializeMcpSystem(): Promise<void> {
  // 空实现
}

export async function restartServer(clientId: string): Promise<void> {
  // 空实现
}

export async function getMcpConfig(): Promise<McpConfigData> {
  const { DEFAULT_MCP_CONFIG } = await import("./types");
  return DEFAULT_MCP_CONFIG;
}

export async function removeClientFromConfig(clientId: string): Promise<void> {
  // 空实现
}

export async function getServerList(): Promise<Record<string, ServerConfig>> {
  return {};
}
