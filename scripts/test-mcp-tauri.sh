#!/bin/bash

# Tauri MCP 功能测试脚本

set -e

echo "================================"
echo "Tauri MCP 功能测试"
echo "================================"
echo ""

# 检查当前目录
if [ ! -f "package.json" ]; then
    echo "❌ 错误: 请在项目根目录运行此脚本"
    exit 1
fi

echo "✓ 检测到项目根目录"
echo ""

# 1. Rust 编译测试
echo "1. 测试 Rust 编译..."
cd src-tauri
if cargo check &> /dev/null; then
    echo "   ✓ Rust 编译通过"
else
    echo "   ❌ Rust 编译失败"
    cargo check
    exit 1
fi
cd ..
echo ""

# 2. TypeScript 编译测试
echo "2. 测试 TypeScript 编译..."
if yarn tsc --noEmit &> /dev/null; then
    echo "   ✓ TypeScript 编译通过"
else
    echo "   ❌ TypeScript 编译失败"
    yarn tsc --noEmit
    exit 1
fi
echo ""

# 3. 检查关键文件
echo "3. 检查关键文件..."
files=(
    "src-tauri/src/mcp.rs"
    "app/mcp/actions.tauri.ts"
    "docs/tauri-mcp-support.md"
    "docs/tauri-mcp-support-cn.md"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✓ $file"
    else
        echo "   ❌ 缺少文件: $file"
        exit 1
    fi
done
echo ""

# 4. 检查配置
echo "4. 检查构建配置..."

# 检查 next.config.mjs
if grep -q "TAURI_PLATFORM" next.config.mjs; then
    echo "   ✓ next.config.mjs 包含 TAURI_PLATFORM 检测"
else
    echo "   ❌ next.config.mjs 缺少 TAURI_PLATFORM 检测"
    exit 1
fi

# 检查 package.json
if grep -q "TAURI_PLATFORM=1" package.json; then
    echo "   ✓ package.json 包含 TAURI_PLATFORM 环境变量"
else
    echo "   ❌ package.json 缺少 TAURI_PLATFORM 环境变量"
    exit 1
fi

# 检查 main.rs
if grep -q "mod mcp" src-tauri/src/main.rs; then
    echo "   ✓ main.rs 包含 MCP 模块"
else
    echo "   ❌ main.rs 缺少 MCP 模块"
    exit 1
fi
echo ""

# 5. 代码质量检查
echo "5. 代码质量检查..."

# 检查 Rust 格式
cd src-tauri
if cargo fmt -- --check &> /dev/null; then
    echo "   ✓ Rust 代码格式正确"
else
    echo "   ⚠ Rust 代码格式需要调整 (运行 cargo fmt 修复)"
fi
cd ..
echo ""

echo "================================"
echo "✅ 所有测试通过!"
echo "================================"
echo ""
echo "下一步:"
echo "1. 运行 'yarn app:dev' 启动开发模式"
echo "2. 在应用中测试 MCP 功能"
echo "3. 运行 'yarn app:build' 构建生产版本"
echo ""

