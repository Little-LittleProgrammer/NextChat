// Tauri 版本的 MCP Actions
import { invoke } from "@tauri-apps/api/tauri";
import {
  McpConfigData,
  McpRequestMessage,
  ServerConfig,
  ServerStatusResponse,
  DEFAULT_MCP_CONFIG,
} from "./types";
import {
  createSSEClient,
  executeSSERequest,
  listSSETools,
  removeSSEClient,
  SSEClient,
} from "./sse-client";

// 获取客户端状态
export async function getClientsStatus(): Promise<
  Record<string, ServerStatusResponse>
> {
  try {
    return await invoke<Record<string, ServerStatusResponse>>(
      "mcp_get_server_status",
    );
  } catch (error) {
    console.error("Failed to get clients status:", error);
    return {};
  }
}

// 获取客户端工具
export async function getClientTools(clientId: string) {
  try {
    // 检查服务器是否活跃
    const status = await getClientsStatus();
    const serverStatus = status[clientId];

    if (!serverStatus || serverStatus.status !== "active") {
      return null;
    }

    // 检查服务器类型
    const config = await getMcpConfigFromFile();
    const serverConfig = config.mcpServers[clientId];
    
    if (!serverConfig) {
      return null;
    }
    
    const serverType = serverConfig.type || "stdio";
    
    // SSE 类型的服务器使用 SSE 客户端
    if (serverType === "sse") {
      let client = sseClientsMap.get(clientId);
      if (!client) {
        client = await createSSEClient(clientId, serverConfig);
        sseClientsMap.set(clientId, client);
      }
      
      const tools = await listSSETools(client);
      return tools;
    } else {
      // stdio 类型的服务器发送 tools/list 请求
      const request: McpRequestMessage = {
        jsonrpc: "2.0",
        id: Date.now().toString(),
        method: "tools/list",
        params: {},
      };

      const response = await executeMcpAction(clientId, request);

      // 检查响应格式
      if (response && typeof response === "object" && "result" in response) {
        return response.result;
      }

      return null;
    }
  } catch (error) {
    console.error(`Failed to get tools for ${clientId}:`, error);
    return null;
  }
}

// 获取可用客户端数量
export async function getAvailableClientsCount() {
  const status = await getClientsStatus();
  return Object.values(status).filter((s) => s.status === "active").length;
}

// 获取所有客户端工具
export async function getAllTools() {
  try {
    const status = await getClientsStatus();
    const result = [];

    for (const [clientId, serverStatus] of Object.entries(status)) {
      if (serverStatus.status === "active") {
        try {
          const tools = await getClientTools(clientId);
          if (tools) {
            result.push({
              clientId,
              tools,
            });
          }
        } catch (error) {
          console.error(`Failed to get tools for ${clientId}:`, error);
        }
      }
    }

    return result;
  } catch (error) {
    console.error("Failed to get all tools:", error);
    return [];
  }
}

// 初始化 MCP 系统
export async function initializeMcpSystem() {
  try {
    const config = await getMcpConfigFromFile();
    
    // 启动所有活跃的服务器
    for (const [clientId, serverConfig] of Object.entries(config.mcpServers)) {
      if (serverConfig.status !== "paused") {
        try {
          await invoke("mcp_start_server", {
            clientId,
            config: serverConfig,
          });
        } catch (error) {
          console.error(`Failed to start server ${clientId}:`, error);
        }
      }
    }
    
    return config;
  } catch (error) {
    console.error("Failed to initialize MCP system:", error);
    throw error;
  }
}

// 添加服务器
export async function addMcpServer(clientId: string, config: ServerConfig) {
  try {
    const currentConfig = await getMcpConfigFromFile();
    const isNewServer = !(clientId in currentConfig.mcpServers);

    // 如果是新服务器，设置默认状态为 active
    if (isNewServer && !config.status) {
      config.status = "active";
    }

    const newConfig = {
      ...currentConfig,
      mcpServers: {
        ...currentConfig.mcpServers,
        [clientId]: config,
      },
    };
    
    await updateMcpConfig(newConfig);

    // 只有新服务器或状态为 active 的服务器才启动
    if (isNewServer || config.status === "active") {
      await invoke("mcp_start_server", {
        clientId,
        config,
      });
    }

    return newConfig;
  } catch (error) {
    console.error(`Failed to add server ${clientId}:`, error);
    throw error;
  }
}

// 暂停服务器
export async function pauseMcpServer(clientId: string) {
  try {
    const currentConfig = await getMcpConfigFromFile();
    const serverConfig = currentConfig.mcpServers[clientId];
    if (!serverConfig) {
      throw new Error(`Server ${clientId} not found`);
    }

    // 先停止服务器
    await invoke("mcp_stop_server", { clientId });

    // 然后更新配置
    const newConfig: McpConfigData = {
      ...currentConfig,
      mcpServers: {
        ...currentConfig.mcpServers,
        [clientId]: {
          ...serverConfig,
          status: "paused",
        },
      },
    };
    await updateMcpConfig(newConfig);

    return newConfig;
  } catch (error) {
    console.error(`Failed to pause server ${clientId}:`, error);
    throw error;
  }
}

// 恢复服务器
export async function resumeMcpServer(clientId: string): Promise<void> {
  try {
    const currentConfig = await getMcpConfigFromFile();
    const serverConfig = currentConfig.mcpServers[clientId];
    if (!serverConfig) {
      throw new Error(`Server ${clientId} not found`);
    }

    // 先尝试启动服务器
    try {
      await invoke("mcp_start_server", {
        clientId,
        config: serverConfig,
      });

      // 启动成功后更新配置
      const newConfig: McpConfigData = {
        ...currentConfig,
        mcpServers: {
          ...currentConfig.mcpServers,
          [clientId]: {
            ...serverConfig,
            status: "active" as const,
          },
        },
      };
      await updateMcpConfig(newConfig);
    } catch (error) {
      // 启动失败，更新状态为 error
      const currentConfig = await getMcpConfigFromFile();
      const serverConfig = currentConfig.mcpServers[clientId];

      if (serverConfig) {
        serverConfig.status = "error";
        await updateMcpConfig(currentConfig);
      }

      throw error;
    }
  } catch (error) {
    console.error(`Failed to resume server ${clientId}:`, error);
    throw error;
  }
}

// 移除服务器
export async function removeMcpServer(clientId: string) {
  try {
    // 检查服务器类型
    const currentConfig = await getMcpConfigFromFile();
    const serverConfig = currentConfig.mcpServers[clientId];
    
    // 如果是 SSE 类型，关闭 SSE 客户端
    if (serverConfig?.type === "sse") {
      const client = sseClientsMap.get(clientId);
      if (client) {
        await removeSSEClient(client);
        sseClientsMap.delete(clientId);
      }
    } else {
      // stdio 类型，停止 Rust 后端进程
      await invoke("mcp_stop_server", { clientId });
    }

    // 然后更新配置
    const { [clientId]: _, ...rest } = currentConfig.mcpServers;
    const newConfig = {
      ...currentConfig,
      mcpServers: rest,
    };
    await updateMcpConfig(newConfig);

    return newConfig;
  } catch (error) {
    console.error(`Failed to remove server ${clientId}:`, error);
    throw error;
  }
}

// 重启所有客户端
export async function restartAllClients() {
  try {
    const config = await getMcpConfigFromFile();

    // 停止所有服务器
    for (const clientId of Object.keys(config.mcpServers)) {
      try {
        await invoke("mcp_stop_server", { clientId });
      } catch (error) {
        console.error(`Failed to stop server ${clientId}:`, error);
      }
    }

    // 重新启动活跃的服务器
    for (const [clientId, serverConfig] of Object.entries(config.mcpServers)) {
      if (serverConfig.status !== "paused") {
        try {
          await invoke("mcp_start_server", {
            clientId,
            config: serverConfig,
          });
        } catch (error) {
          console.error(`Failed to start server ${clientId}:`, error);
        }
      }
    }

    return config;
  } catch (error) {
    console.error("Failed to restart clients:", error);
    throw error;
  }
}

// SSE 客户端缓存（仅用于 SSE 类型的服务器）
const sseClientsMap = new Map<string, SSEClient>();

// 执行 MCP 请求
export async function executeMcpAction(
  clientId: string,
  request: McpRequestMessage,
) {
  try {
    // 检查服务器类型
    const config = await getMcpConfigFromFile();
    const serverConfig = config.mcpServers[clientId];
    
    if (!serverConfig) {
      throw new Error(`Server ${clientId} not found`);
    }
    
    const serverType = serverConfig.type || "stdio";
    
    // SSE 类型的服务器使用 SSE 客户端处理
    if (serverType === "sse") {
      // 获取或创建 SSE 客户端
      let client = sseClientsMap.get(clientId);
      if (!client) {
        client = await createSSEClient(clientId, serverConfig);
        sseClientsMap.set(clientId, client);
      }
      
      return await executeSSERequest(client, request);
    } else {
      // stdio 类型的服务器使用 Rust 后端处理
      return await invoke("mcp_execute_command", {
        clientId,
        request,
      });
    }
  } catch (error) {
    console.error(`Failed to execute request for ${clientId}:`, error);
    throw error;
  }
}

// 获取 MCP 配置文件
export async function getMcpConfigFromFile(): Promise<McpConfigData> {
  try {
    return await invoke<McpConfigData>("mcp_read_config");
  } catch (error) {
    console.error("Failed to load MCP config, using default config:", error);
    return DEFAULT_MCP_CONFIG;
  }
}

// 更新 MCP 配置文件
async function updateMcpConfig(config: McpConfigData): Promise<void> {
  try {
    await invoke("mcp_write_config", { config });
  } catch (error) {
    console.error("Failed to write MCP config:", error);
    throw error;
  }
}

// 检查 MCP 是否启用
export async function isMcpEnabled() {
  // Tauri 环境下默认启用 MCP
  return true;
}

// 导入用户配置（从旧版 NextChat 路径）
export async function importUserMcpConfig(): Promise<McpConfigData> {
  try {
    return await invoke<McpConfigData>("mcp_import_user_config");
  } catch (error) {
    console.error("Failed to import user config:", error);
    return getMcpConfigFromFile();
  }
}
