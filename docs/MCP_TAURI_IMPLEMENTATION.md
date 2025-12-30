# MCP Support for Tauri App

本次实现为 NextChat 的 Tauri 桌面应用添加了完整的 MCP (Model Context Protocol) 支持。

## 快速开始

### 开发

```bash
yarn app:dev
```

### 构建

```bash
yarn app:build
```

## 主要变更

### 新增文件

1. **Rust 后端**
   - `src-tauri/src/mcp.rs` - MCP 核心实现

2. **TypeScript 前端**
   - `app/mcp/actions.tauri.ts` - Tauri 版 MCP actions
   - `app/mcp/env.ts` - 环境检测

3. **文档**
   - `docs/tauri-mcp-support.md` - 英文技术文档
   - `docs/tauri-mcp-support-cn.md` - 中文技术文档

### 修改文件

1. `next.config.mjs` - 添加构建时模块别名
2. `src-tauri/src/main.rs` - 注册 MCP 命令
3. `src-tauri/Cargo.toml` - 更新依赖
4. `package.json` - 更新构建脚本

## 功能特性

✅ 启动/停止 MCP 服务器进程  
✅ 配置持久化存储  
✅ 服务器状态管理  
✅ 命令执行 (stdin/stdout 通信)  
✅ 多服务器支持  
✅ 环境变量配置  

## 架构

应用根据构建模式自动选择合适的 MCP 实现:

- **Standalone 模式**: 使用 Node.js 实现 (`actions.ts`)
- **Export 模式**: 使用空实现 (`actions.client.ts`)  
- **Tauri 模式**: 使用 Rust 实现 (`actions.tauri.ts`)

## 使用示例

在 MCP 市场中添加服务器:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
      "env": {},
      "status": "active"
    }
  }
}
```

配置会自动保存到应用数据目录。

## 详细文档

- [中文文档](./tauri-mcp-support-cn.md)
- [English Documentation](./tauri-mcp-support.md)

## 测试

基本测试清单:

- [ ] 添加 MCP 服务器
- [ ] 启动/停止服务器
- [ ] 执行工具调用
- [ ] 配置持久化
- [ ] 跨平台测试 (macOS/Windows/Linux)

## 后续改进

- 实现完整的 MCP 协议握手
- 工具列表缓存
- 进程健康检查
- 性能监控
- 更好的错误处理

## 相关链接

- [MCP 规范](https://spec.modelcontextprotocol.io/)
- [NextChat 项目](https://github.com/ChatGPTNextWeb/ChatGPT-Next-Web)

