# Tauri 应用 MCP 支持实现说明

## 概述

本次更新为 NextChat 的 Tauri 桌面应用添加了完整的 MCP (Model Context Protocol) 支持,使得打包后的桌面应用可以像 Web 版本一样使用 MCP 功能。

## 主要变更

### 1. 新增文件

#### Rust 后端
- `src-tauri/src/mcp.rs` - MCP 功能的 Rust 实现
  - 进程管理
  - 配置读写
  - 命令执行
  - 状态查询

#### TypeScript 前端
- `app/mcp/actions.tauri.ts` - Tauri 环境下的 MCP actions 实现
- `app/mcp/env.ts` - 环境检测工具
- `docs/tauri-mcp-support.md` - 英文技术文档

### 2. 修改文件

#### 构建配置
- `next.config.mjs` 
  - 添加 Tauri 环境检测
  - 根据构建模式选择不同的 MCP actions 实现

#### Rust 配置
- `src-tauri/src/main.rs`
  - 添加 MCP 模块
  - 注册 MCP 相关的 Tauri 命令

- `src-tauri/Cargo.toml`
  - 更新 tauri-plugin-window-state 版本

#### 脚本
- `package.json`
  - 更新 `app:dev` 和 `app:build` 脚本,添加 `TAURI_PLATFORM` 环境变量

## 技术架构

### 三种运行模式

1. **Next.js 服务器模式** (`BUILD_MODE=standalone`)
   - 使用 `app/mcp/actions.ts`
   - 基于 Node.js 的 `fs` 和 `child_process`
   - 支持完整的服务器端功能

2. **静态导出模式** (`BUILD_MODE=export`)
   - 使用 `app/mcp/actions.client.ts`
   - 空实现,MCP 功能不可用
   - 用于纯静态部署

3. **Tauri 桌面模式** (`TAURI_PLATFORM=1`)
   - 使用 `app/mcp/actions.tauri.ts`
   - 基于 Tauri IPC 和 Rust 后端
   - 支持桌面环境的完整功能

### 工作原理

```
┌─────────────────────────────────────────┐
│         React 前端组件                    │
│    (mcp-market.tsx, chat.ts, etc.)     │
└──────────────┬──────────────────────────┘
               │ import from '../mcp/actions'
               ▼
┌─────────────────────────────────────────┐
│      Webpack 别名 (next.config.mjs)     │
│   根据环境重定向到不同的实现文件          │
└──────────────┬──────────────────────────┘
               │
       ┌───────┴───────┐
       │               │
       ▼               ▼
┌─────────────┐  ┌─────────────────┐
│ actions.ts  │  │ actions.tauri.ts│
│ (Node.js)   │  │  (Tauri IPC)    │
└─────────────┘  └────────┬─────────┘
                          │ invoke('mcp_xxx')
                          ▼
                 ┌─────────────────┐
                 │   Rust 后端      │
                 │   (mcp.rs)      │
                 │                 │
                 │ - 进程管理       │
                 │ - 配置存储       │
                 │ - 命令执行       │
                 └─────────────────┘
```

### MCP 服务器通信

```
┌──────────────┐                ┌──────────────┐
│   Rust      │   spawn()      │ MCP Server   │
│   mcp.rs    │  ──────────>   │   Process    │
└──────┬───────┘                └──────┬───────┘
       │                               │
       │  JSON-RPC Request (stdin)     │
       │  ───────────────────────────> │
       │                               │
       │  JSON-RPC Response (stdout)   │
       │  <─────────────────────────── │
       │                               │
```

## 核心 API

### Rust 命令 (Tauri Commands)

```rust
// 读取配置
mcp_read_config() -> McpConfig

// 写入配置
mcp_write_config(config: McpConfig)

// 启动服务器
mcp_start_server(client_id: String, config: ServerConfig)

// 停止服务器
mcp_stop_server(client_id: String)

// 执行命令
mcp_execute_command(client_id: String, request: JsonValue) -> JsonValue

// 获取状态
mcp_get_server_status() -> HashMap<String, ServerStatusResponse>
```

### TypeScript API (保持一致)

所有 actions 文件都实现相同的接口:

```typescript
// 系统管理
initializeMcpSystem()
isMcpEnabled()
getClientsStatus()
getAvailableClientsCount()

// 服务器管理
addMcpServer(clientId, config)
pauseMcpServer(clientId)
resumeMcpServer(clientId)
removeMcpServer(clientId)
restartAllClients()

// 工具相关
getClientTools(clientId)
getAllTools()
executeMcpAction(clientId, request)

// 配置管理
getMcpConfigFromFile()
```

## 使用方法

### 开发模式

```bash
# 启动 Tauri 开发模式
yarn app:dev
```

这会:
1. 设置 `TAURI_PLATFORM=1` 环境变量
2. Next.js 使用 Tauri 版本的 MCP actions
3. 启动 Tauri 开发服务器

### 构建生产版本

```bash
# 构建 Tauri 应用
yarn app:build
```

### 添加 MCP 服务器

1. 启动应用
2. 进入 "MCP 市场" 页面
3. 点击添加服务器
4. 配置服务器信息:
   - 命令: 例如 `npx`
   - 参数: 例如 `["-y", "@modelcontextprotocol/server-filesystem", "/path"]`
   - 环境变量: 可选
5. 保存并启动

配置会保存在:
- macOS: `~/Library/Application Support/com.yida.chatgpt.next.web/mcp_config.json`
- Windows: `%APPDATA%\com.yida.chatgpt.next.web\mcp_config.json`
- Linux: `~/.local/share/com.yida.chatgpt.next.web/mcp_config.json`

## 已知限制和后续改进

### 当前限制

1. ��具列表功能尚未完全实现 (`getClientTools`, `getAllTools`)
2. 进程通信使用简单的行读取,不支持复杂的流式响应
3. 错误处理可以更完善
4. 没有实现 MCP 完整的初始化握手流程

### 计划改进

1. 实现完整的 MCP 协议客户端
   - 初始化握手
   - 能力协商
   - 工具列表获取和缓存

2. 改进进程管理
   - 自动重启失败的进程
   - 进程健康检查
   - 资源使用监控

3. 增强错误处理
   - 更详细的错误信息
   - 错误恢复机制
   - 日志记录

4. 性能优化
   - 工具列表缓存
   - 批量命令执行
   - 异步处理改进

## 测试建议

### 基本功能测试

- [ ] 启动应用
- [ ] 打开 MCP 市场
- [ ] 添加一个 MCP 服务器(如 filesystem)
- [ ] 查看服务器状态为 "active"
- [ ] 在聊天中使用 MCP 工具
- [ ] 暂停服务器
- [ ] 恢复服务器
- [ ] 删除服务器
- [ ] 重启应用,确认配置持久化

### 跨平台测试

在以下平台上执行基本功能测试:
- [ ] macOS (x64)
- [ ] macOS (arm64)
- [ ] Windows
- [ ] Linux

### 常见 MCP 服务器测试

- [ ] @modelcontextprotocol/server-filesystem
- [ ] @modelcontextprotocol/server-github
- [ ] 自定义 MCP 服务器

## 故障排查

### 服务器无法启动

1. 检查命令路径是否正确
2. 检查参数是否正确
3. 查看应用日志 (开发模式下在终端)
4. 确认 MCP 服务器可以独立运行

### 无法执行工具

1. 确认服务器状态为 "active"
2. 检查工具调用的参数格式
3. 查看错误信息

### 配置丢失

1. 检查应用数据目录权限
2. 确认 `mcp_config.json` 文件存在
3. 检查 JSON 格式是否正确

## 贡献指南

如果要继续完善 MCP 功能:

1. Rust 后端: 编辑 `src-tauri/src/mcp.rs`
2. TypeScript 接口: 编辑 `app/mcp/actions.tauri.ts`
3. 类型定义: 编辑 `app/mcp/types.ts`
4. 测试后提交 PR

## 相关资源

- [MCP 规范](https://spec.modelcontextprotocol.io/)
- [Tauri 文档](https://tauri.app/)
- [MCP SDK](https://github.com/modelcontextprotocol/sdk)

