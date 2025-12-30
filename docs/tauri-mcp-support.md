# Tauri MCP 支持实现

## 概述

为 NextChat 的 Tauri 应用添加了完整的 MCP (Model Context Protocol) 支持。MCP 允许应用与外部工具和服务进行通信。

## 架构设计

### 1. 多环境支持

项目现在支持三种运行模式,每种模式有不同的 MCP 实现:

- **Standalone 模式** (Next.js 服务器): 使用 `app/mcp/actions.ts` - 基于 Node.js 的完整实现
- **Export 模式** (静态导出): 使用 `app/mcp/actions.client.ts` - 空实现,MCP 不可用
- **Tauri 模式** (桌面应用): 使用 `app/mcp/actions.tauri.ts` - 基于 Tauri API 的实现

### 2. 构建时别名

在 `next.config.mjs` 中配置了 webpack 别名:

```javascript
if (isTauri) {
  // Tauri 模式:使用 Tauri API 实现
  config.resolve.alias = {
    [path.resolve("./app/mcp/actions")]: path.resolve("./app/mcp/actions.tauri.ts"),
  };
}
```

这样,所有导入 `../mcp/actions` 的代码在 Tauri 构建时会自动使用 Tauri 实现。

### 3. Rust 后端实现

在 `src-tauri/src/mcp.rs` 中实现了以下 Tauri 命令:

- `mcp_read_config` - 读取 MCP 配置文件
- `mcp_write_config` - 写入 MCP 配置文件
- `mcp_start_server` - 启动 MCP 服务器进程
- `mcp_stop_server` - 停止 MCP 服务器进程
- `mcp_execute_command` - 执行 MCP 命令(通过 stdin/stdout 通信)
- `mcp_get_server_status` - 获取所有服务器状态

## 功能特性

### 1. 进程管理

- 支持启动/停止多个 MCP 服务器进程
- 自动处理进程清理
- 支持环境变量配置

### 2. 配置管理

- 配置文件存储在应用数据目录 (`app_data_dir/mcp_config.json`)
- 支持服务器状态管理 (active/paused/error)
- 自动初始化配置

### 3. 通信机制

- 通过 stdin/stdout 与 MCP 服务器通信
- JSON-RPC 2.0 协议
- 异步请求/响应模式

## 使用方法

### 1. 构建 Tauri 应用

```bash
# 设置环境变量以启用 Tauri 模式
export TAURI_PLATFORM=macos  # 或 windows, linux
yarn tauri build
```

### 2. 添加 MCP 服务器

在应用的 MCP 市场页面中:

1. 点击 "添加服务器"
2. 选择预设服务器或自定义配置
3. 配置服务器命令、参数和环境变量
4. 保存配置

配置示例:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed/directory"],
      "env": {},
      "status": "active"
    }
  }
}
```

### 3. 使用 MCP 工具

在聊天中,MCP 工具会自动注入到模型的系统提示中。模型可以调用这些工具来执行操作。

## 文件结构

```
app/mcp/
├── actions.ts           # Node.js 服务器实现
├── actions.client.ts    # 静态导出空实现
├── actions.tauri.ts     # Tauri 实现 (新增)
├── client.ts           # MCP 客户端 (仅 Node.js)
├── types.ts            # 类型定义
├── utils.ts            # 工具函数
└── env.ts              # 环境检测 (新增)

src-tauri/src/
├── main.rs             # 主入口 (已更新)
├── mcp.rs              # MCP 功能实现 (新增)
├── stream.rs
└── fetch.rs
```

## 配置文件位置

配置文件根据平台存储在不同位置:

- **macOS**: `~/Library/Application Support/com.yida.chatgpt.next.web/mcp_config.json`
- **Windows**: `%APPDATA%\com.yida.chatgpt.next.web\mcp_config.json`
- **Linux**: `~/.local/share/com.yida.chatgpt.next.web/mcp_config.json`

## 权限配置

在 `src-tauri/tauri.conf.json` 中已添加必要的权限:

```json
{
  "shell": {
    "execute": true,
    "sidecar": true,
    "scope": [
      {
        "name": "mcp-server",
        "cmd": "",
        "args": true
      }
    ]
  },
  "fs": {
    "all": true
  }
}
```

## 调试

### 查看日志

开发模式下,可以在终端看到详细日志:

```bash
yarn tauri dev
```

### 测试 MCP 服务器

可以手动测试 MCP 服务器是否正常工作:

```bash
npx -y @modelcontextprotocol/server-filesystem /tmp
```

然后在 stdin 输入:

```json
{"jsonrpc":"2.0","method":"tools/list","id":1}
```

应该会在 stdout 收到工具列表响应。

## 已知限制

1. **进程通信**: 当前实现使用简单的 stdin/stdout 行读取,不支持复杂的流式响应
2. **错误处理**: 需要更完善的错误恢复机制
3. **工具列表**: `getClientTools` 和 `getAllTools` 尚未完全实现,需要在连接建立后调用 `tools/list` 方法

## 后续改进

1. 实现完整的 MCP 协议客户端(初始化握手、能力协商等)
2. 添加工具列表缓存和自动刷新
3. 支持 MCP 服务器的自动重启
4. 添加更详细的错误信息和状态报告
5. 支持 SSE (Server-Sent Events) 和 WebSocket 传输
6. 添加 MCP 服务器性能监控

## 测试清单

- [ ] 在 macOS 上构建和运行
- [ ] 在 Windows 上构建和运行
- [ ] 在 Linux 上构建和运行
- [ ] 添加 MCP 服务器配置
- [ ] 启动/停止服务器
- [ ] 暂停/恢复服务器
- [ ] 执行 MCP 工具调用
- [ ] 查看工具列表
- [ ] 配置持久化
- [ ] 应用重启后配置恢复

