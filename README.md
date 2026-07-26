# Brave Search MCP Server

一个用 Rust 实现的 Brave Search MCP 服务器，支持 HTTP Streamable 传输和智能多 API Key 负载均衡。

## 特性

- 🚀 **高性能**: Rust 原生编译，低内存占用
- 🔑 **智能多 Key 负载均衡**: 
  - 支持 200+ API Key 轮询
  - 自动跳过被限流的 Key（429 错误）
  - 指数退避冷却（1分钟 → 1小时）
  - 认证错误自动禁用（401/403）
  - 最多重试 3 次，每次使用不同 Key
- 🌐 **HTTP Streamable**: 符合 MCP 2024-11-05 协议
- 🛡️ **安全防护**: DNS 重绑定攻击防护、CORS 支持
- 📊 **实时监控**: 查看 Key 使用状态和成功率
- 📦 **容器化**: 提供 Docker 镜像和 docker-compose
- 🔧 **8 个搜索工具**: 网页、图片、视频、新闻、本地商家、AI 摘要、LLM 上下文、地点搜索

## 快速开始

### 使用 Cargo 安装

```bash
# 单个 Key
cargo install --path .
brave-search-mcp-server --brave-api-keys YOUR_API_KEY

# 多个 Key（逗号分隔）
brave-search-mcp-server --brave-api-keys KEY1,KEY2,KEY3

# 从文件加载（推荐用于 200+ Key）
brave-search-mcp-server --brave-api-keys-file keys.txt
```

### 使用 Docker

```bash
# 环境变量方式
docker run -p 8080:8080 \
  -e BRAVE_API_KEYS=KEY1,KEY2,KEY3 \
  ghcr.io/ifpj/brave-search-mcp-server:latest

# 文件方式（推荐）
docker run -p 8080:8080 \
  -v ./keys.txt:/app/keys.txt \
  -e BRAVE_API_KEYS_FILE=/app/keys.txt \
  ghcr.io/ifpj/brave-search-mcp-server:latest
```

### 使用 Docker Compose

```bash
# 创建 keys.txt 文件
cat > keys.txt << EOF
KEY1
KEY2
KEY3
EOF

# 启动服务
docker-compose up -d
```

## 多 Key 管理

### Key 文件格式

`keys.txt` 文件支持：
- 每行一个 Key
- `#` 开头的注释行
- 空行会被自动忽略

```bash
# keys.txt 示例
# 生产环境 API Keys
key1_xxxxx
key2_xxxxx

# 备用 Keys
key3_xxxxx
key4_xxxxx
```

### 负载均衡策略

服务器会自动管理所有 Key：

1. **Round-robin 轮转**: 请求均匀分布到所有可用 Key
2. **429 限流处理**: 
   - 自动跳过被限流的 Key
   - 指数退避冷却：60秒 → 120秒 → 240秒 → ... → 最大 3600秒
3. **401/403 禁用**: 认证失败的 Key 被永久禁用（24小时冷却）
4. **自动重试**: 最多重试 3 次，每次使用不同 Key
5. **并发安全**: 使用 AtomicUsize 保证线程安全

## 配置

### 命令行参数

```bash
brave-search-mcp-server \
  --brave-api-keys KEY1,KEY2,KEY3 \
  --brave-api-keys-file keys.txt \
  --host 0.0.0.0 \
  --port 8080 \
  --allowed-origins "http://localhost:3000,https://example.com" \
  --allowed-hosts "example.com,api.example.com" \
  --enabled-tools brave_web_search,brave_news_search \
  --log-level info
```

### 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `BRAVE_API_KEYS` | API Keys（逗号分隔） | - |
| `BRAVE_API_KEYS_FILE` | Keys 文件路径 | - |
| `BRAVE_MCP_HOST` | 监听地址 | `127.0.0.1` |
| `BRAVE_MCP_PORT` | 监听端口 | `8080` |
| `BRAVE_MCP_ALLOWED_ORIGINS` | 允许的 CORS 源（逗号分隔） | - |
| `BRAVE_MCP_ALLOWED_HOSTS` | 允许的主机名（逗号分隔） | - |
| `BRAVE_MCP_ENABLED_TOOLS` | 启用的工具（逗号分隔） | 全部启用 |
| `BRAVE_MCP_DISABLED_TOOLS` | 禁用的工具（逗号分隔） | - |
| `BRAVE_MCP_LOG_LEVEL` | 日志级别 | `info` |

## API 端点

### MCP 协议

- **POST `/mcp`**: MCP 协议端点，用于工具调用

### 监控端点

- **GET `/health`**: 健康检查
- **GET `/keys`**: 查看所有 Key 的详细状态
- **GET `/keys/summary`**: 查看 Key 池汇总统计

### 监控示例

```bash
# 查看 Key 池汇总
curl http://localhost:8080/keys/summary
```

返回示例：
```json
{
  "total_keys": 200,
  "available_keys": 195,
  "rate_limited_keys": 3,
  "disabled_keys": 2,
  "success_rate": 98.5
}
```

```bash
# 查看每个 Key 的详情
curl http://localhost:8080/keys | jq '.keys[:3]'
```

返回示例：
```json
{
  "keys": [
    {
      "id": "key_001",
      "success_count": 150,
      "failure_count": 2,
      "is_available": true,
      "rate_limited_until": null,
      "last_error": null
    },
    {
      "id": "key_002",
      "success_count": 145,
      "failure_count": 0,
      "is_available": true,
      "rate_limited_until": null,
      "last_error": null
    },
    {
      "id": "key_003",
      "success_count": 120,
      "failure_count": 5,
      "is_available": false,
      "rate_limited_until": "2026-01-27T10:30:00Z",
      "last_error": "429 Too Many Requests"
    }
  ]
}
```

## 可用工具

1. **brave_web_search** - 网页搜索（支持网页、FAQ、讨论、新闻、视频结果）
2. **brave_image_search** - 图片搜索
3. **brave_video_search** - 视频搜索
4. **brave_news_search** - 新闻搜索
5. **brave_local_search** - 本地商家搜索（两步调用）
6. **brave_summarizer** - AI 摘要生成（轮询机制）
7. **brave_llm_context** - LLM 上下文检索
8. **brave_place_search** - 地点搜索

## 使用示例

### 1. 基础搜索

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "brave_web_search",
      "arguments": {
        "query": "rust programming",
        "count": 5
      }
    }
  }'
```

### 2. 多 Key 负载均衡（200+ Keys）

```bash
# 创建 keys.txt
for i in {1..200}; do
  echo "key_${i}_xxxxx" >> keys.txt
done

# 启动服务器
brave-search-mcp-server --brave-api-keys-file keys.txt

# 查看负载均衡状态
curl http://localhost:8080/keys/summary
```

### 3. Docker 部署

```bash
# docker-compose.yml
version: '3.8'
services:
  mcp-server:
    image: ghcr.io/ifpj/brave-search-mcp-server:latest
    ports:
      - "8080:8080"
    volumes:
      - ./keys.txt:/app/keys.txt
    environment:
      - BRAVE_API_KEYS_FILE=/app/keys.txt
      - BRAVE_MCP_HOST=0.0.0.0
      - BRAVE_MCP_PORT=8080
```

### 4. 安全配置

```bash
brave-search-mcp-server \
  --brave-api-keys-file keys.txt \
  --allowed-origins "https://myapp.com,https://admin.myapp.com" \
  --allowed-hosts "api.myapp.com"
```

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

# 构建 Docker 镜像
docker build -t brave-search-mcp-server .
```

## 性能特点

- **内存占用**: ~10-20MB（相比 Node.js 版本的 100MB+）
- **响应时间**: 原生编译，比 Node.js 快 2-3 倍
- **并发处理**: Tokio 异步运行时，支持高并发
- **Key 管理**: 无锁原子操作，线程安全

## 许可证

MIT
