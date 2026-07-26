# Brave Search MCP Server

Rust 实现的 Brave Search MCP 服务器，支持智能多 Key 负载均衡。

## 特性

- 8 个搜索工具：web、image、video、news、local、summarizer、llm_context、place_search
- 200+ API Key 智能轮询，自动跳过限流/失效 Key
- 实时监控端点：`/keys`、`/keys/summary`
- MCP 协议 2024-11-05

## 快速开始

### Docker

```yaml
# docker-compose.yml
services:
  brave-search-mcp:
    image: ghcr.io/ifpj/brave-search-mcp-server:latest
    ports:
      - "8080:8080"
    environment:
      - BRAVE_API_KEYS_FILE=/app/keys.txt
      - BRAVE_MCP_HOST=0.0.0.0
      - BRAVE_MCP_PORT=8080
    volumes:
      - ./keys.txt:/app/keys.txt
    restart: unless-stopped
```

### 二进制

```bash
brave-search-mcp-server --brave-api-keys-file keys.txt
```

## MCP 客户端配置

```json
{
  "mcpServers": {
    "brave-search": {
      "type": "http",
      "url": "http://localhost:8080/mcp"
    }
  }
}
```

## Key 输入方式

| 方式 | 示例 |
|------|------|
| 命令行 | `--brave-api-keys KEY1,KEY2` |
| 环境变量 | `BRAVE_API_KEYS=KEY1,KEY2` |
| 文件（推荐） | `--brave-api-keys-file keys.txt` |

`keys.txt` 格式：每行一个 key，`#` 开头为注释，空行忽略。

```txt
# 生产 Keys
key1_xxxxx
key2_xxxxx

# 备用 Keys
key3_xxxxx
```

## 配置

| 参数 | 环境变量 | 默认值 | 说明 |
|------|----------|--------|------|
| `--brave-api-keys` | `BRAVE_API_KEYS` | - | API Keys（逗号分隔） |
| `--brave-api-keys-file` | `BRAVE_API_KEYS_FILE` | - | Keys 文件路径 |
| `--host` | `BRAVE_MCP_HOST` | `127.0.0.1` | 监听地址 |
| `--port` | `BRAVE_MCP_PORT` | `8080` | 监听端口 |
| `--allowed-origins` | `BRAVE_MCP_ALLOWED_ORIGINS` | - | 允许的 Origin |
| `--allowed-hosts` | `BRAVE_MCP_ALLOWED_HOSTS` | - | 允许的 Host |
| `--enabled-tools` | `BRAVE_MCP_ENABLED_TOOLS` | 全部 | 启用的工具 |
| `--disabled-tools` | `BRAVE_MCP_DISABLED_TOOLS` | - | 禁用的工具 |
| `--log-level` | `BRAVE_MCP_LOG_LEVEL` | `info` | 日志级别 |

## API 端点

| 端点 | 方法 | 说明 |
|------|------|------|
| `/mcp` | POST | MCP 协议 |
| `/health` | GET | 健康检查 |
| `/keys` | GET | Key 详细状态 |
| `/keys/summary` | GET | Key 汇总统计 |

## 负载均衡策略

- **Round-robin**: 请求均匀分配到可用 Key
- **429 限流**: 自动跳过 + 指数退避（1min → 1h）
- **401/403**: 认证失败自动禁用
- **重试**: 最多 3 次，每次换 Key

## 开发

```bash
cargo build --release
cargo run -- --brave-api-keys YOUR_KEY
```

## 许可证

MIT
