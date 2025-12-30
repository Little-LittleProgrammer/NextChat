export function isMcpJson(content: string) {
  // 匹配 ```json:mcp:xxx 后跟 JSON 内容，最后以 ``` 结束
  return content.match(/```json:mcp:([^\s`]+)[\s\S]*?```/);
}

export function extractMcpJson(content: string) {
  // 匹配 ```json:mcp:clientId，然后是换行符，然后是 { 开头的 JSON，最后是 ```
  // 使用更精确的匹配：clientId 后必须立即是换行符
  const match = content.match(/```json:mcp:([^\s`]+)\n([\s\S]*?)```/);
  if (match && match.length === 3) {
    try {
      const mcp = JSON.parse(match[2].trim());
      return { clientId: match[1], mcp };
    } catch (e) {
      console.error("[MCP] Failed to parse MCP JSON:", e);
      console.error("[MCP] Raw JSON content:", match[2]);
      return null;
    }
  }
  return null;
}

/**
 * 从消息内容中移除 MCP 响应块（```json:mcp-response:xxx```）
 * 提取 JSON 内容并显示为 `> mcp名称` 格式
 */
export function removeMcpResponseBlocks(content: string): string {
  // 匹配所有 ```json:mcp-response:clientId``` 块
  return content.replace(
    /```json:mcp-response:([^\s`]+)\n([\s\S]*?)```/g,
    (match, clientId, jsonContent) => {
      try {
        const json = JSON.parse(jsonContent.trim());
        // 尝试从 JSON 中提取工具名称
        // 响应块可能包含 result，我们可以使用 clientId 或尝试从上下文推断
        const toolName =
          json.result?.name ||
          json.params?.name ||
          clientId ||
          "mcp-response";
        return `> 【mcp:${clientId}】 ${toolName} \n\n `;
      } catch {
        // 如果 JSON 解析失败，使用 clientId
        return `> 【mcp:${clientId}】 ${clientId} \n\n `;
      }
    },
  );
}

/**
 * 从消息内容中移除 MCP 请求块（```json:mcp:xxx```）
 * 提取 JSON 内容并显示为 `> mcp名称` 格式
 */
export function removeMcpRequestBlocks(content: string): string {
  // 匹配所有 ```json:mcp:clientId``` 块
  return content.replace(
    /```json:mcp:([^\s`]+)\n([\s\S]*?)```/g,
    (match, clientId, jsonContent) => {
      try {
        const json = JSON.parse(jsonContent.trim());
        // 从请求 JSON 中提取工具名称
        // 请求块结构：{ "method": "tools/call", "params": { "name": "tool_name", ... } }
        const toolName =
          json.params?.name ||
          json.method?.split("/").pop() ||
          clientId ||
          "mcp-tool";
        return `> 【mcp:${clientId}】 ${toolName} \n\n `;
      } catch {
        // 如果 JSON 解析失败，使用 clientId
        return `> 【mcp:${clientId}】 ${clientId} \n\n `;
      }
    },
  );
}
