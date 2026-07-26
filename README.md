# Brave Search MCP Server

一个用 Rust 实现的 Brave Search MCP 服务器，支持 HTTP Streamable 传输和多 API Key 负载均衡。

## 特性

- 🚀 **高性能**: Rust 原生编译，低内存占用
- 🔑 **多 Key 负载均衡**: 自动轮询多个 Brave API Key
- 🌐 **HTTP Streamable**: 符合 MCP 2024-11-05 协议
- 🛡️ **安全防护**: DNS 重绑定攻击防护、CORS 支持
- 📦 **容器化**: 提供 Docker 镜像和 docker-compose
- 🔧 **8 个搜索工具**: 网页、图片、视频、新闻、本地商家、AI 摘要、LLM 上下文、地点搜索

## 快速开始

### 使用 Cargo 安装

```bash
cargo install --path .
brave-search-mcp-server --brave-api-keys YOUR_API_KEY
```

### 使用 Docker

```bash
docker run -p 3000:3000 -e BRAVE_API_KEYS=YOUR_API_KEY ghcr.io/ifpj/brave-search-mcp-server:main
```

### 使用 Docker Compose

```bash
export BRAVE_API_KEYS=YOUR_API_KEY
docker-compose up -d
```

## 配置

### 命令行参数

```bash
brave-search-mcp-server \
  --brave-api-keys KEY1,KEY2,KEY3 \
  --host 0.0.0.0 \
  --port 3000 \
  --allowed-origins "http://localhost:3000" \
  --allowed-hosts "example.com" \
  --enabled-tools brave_web_search,brave_news_search
```

### 环境变量

- `BRAVE_API_KEYS`: Brave API Keys（逗号分隔）
- `MCP_HOST`: 监听地址（默认 127.0.0.1）
- `MCP_PORT`: 监听端口（默认 3000）
- `MCP_ALLOWED_ORIGINS`: 允许的 CORS 源
- `MCP_ALLOWED_HOSTS`: 允许的主机名
- `MCP_ENABLED_TOOLS`: 启用的工具
- `MCP_DISABLED_TOOLS`: 禁用的工具

## API 端点

- **MCP 协议**: `POST /mcp`
- **健康检查**: `GET /health`

## 可用工具

1. `brave_web_search` - 网页搜索
2. `brave_image_search` - 图片搜索
3. `brave_video_search` - 视频搜索
4. `brave_news_search` - 新闻搜索
5. `brave_local_search` - 本地商家搜索
6. `brave_summarizer` - AI 摘要生成
7. `brave_llm_context` - LLM 上下文检索
8. `brave_place_search` - 地点搜索

## 开发

```bash
# 克隆仓库
git clone https://github.com/ifpj/brave-search-mcp-server.git
cd brave-search-mcp-server

# 编译
cargo build

# 运行
cargo run -- --brave-api-keys YOUR_API_KEY

# 测试
cargo test
```

## 许可证

MIT
