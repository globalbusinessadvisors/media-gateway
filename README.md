# Media Gateway

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0+-blue.svg)](https://www.typescriptlang.org/)

> **AI-Native Entertainment Discovery Platform** - Solving the "45-minute decision problem" with unified search across 150+ streaming platforms.

Media Gateway is a production-ready platform that helps users find what to watch through natural language search, AI-powered personalization (SONA), and real-time cross-device synchronization. Built with **Rust** for performance-critical services and **TypeScript** for AI agent integration.

---

## Key Features

| Feature | Description | Performance |
|---------|-------------|-------------|
| **Unified Search** | Natural language queries across 150+ streaming platforms | <500ms p95 latency |
| **SONA Personalization** | AI-powered recommendations using ONNX neural models | <5ms inference |
| **Cross-Device Sync** | Real-time watchlist & progress sync via CRDT | <100ms latency |
| **MCP Integration** | Model Context Protocol server for AI agents | 10+ tools exposed |
| **ARW Protocol** | Agent-Ready Web specification implementation | 85% token reduction |

---

## Architecture

### Technology Stack

```yaml
Backend (80% Rust):
  - Rust 2021 Edition (1.75+)
  - Actix-web 4.x for HTTP services
  - Tokio async runtime
  - SQLx + PostgreSQL
  - Redis for caching
  - Qdrant for vector search
  - gRPC with Tonic

Frontend/AI Integration (20% TypeScript):
  - MCP Server with Model Context Protocol SDK
  - AgentDB for AI memory & learning
  - Agentic-Flow for orchestration
```

### Service Architecture

| Service | Port | Language | Description |
|---------|------|----------|-------------|
| **API Gateway** | 8080 | Rust | Request routing, rate limiting, auth validation |
| **Discovery Service** | 8081 | Rust | Natural language search, content lookup |
| **SONA Engine** | 8082 | Rust | AI-powered personalization & recommendations |
| **Sync Service** | 8083 | Rust | CRDT-based cross-device state synchronization |
| **Auth Service** | 8084 | Rust | OAuth 2.0 + PKCE, JWT tokens |
| **Ingestion Service** | 8085 | Rust | Platform data fetching & normalization |
| **Playback Service** | 8086 | Rust | Device management, deep link generation |
| **MCP Server** | 3000 | TypeScript | AI agent integration via Model Context Protocol |

---

## Project Structure

```
media-gateway/
├── crates/                    # Rust workspace
│   ├── api/                   # API Gateway service
│   ├── auth/                  # Authentication service
│   ├── core/                  # Shared types & utilities
│   ├── discovery/             # Search & content discovery
│   ├── ingestion/             # Platform data ingestion
│   ├── mcp-server/            # Rust MCP bindings
│   ├── playback/              # Device & playback management
│   ├── sona/                  # AI recommendation engine
│   └── sync/                  # Cross-device synchronization
│
├── apps/                      # TypeScript applications
│   ├── agentdb/               # AI memory & vector database (v2.0)
│   ├── agentic-flow/          # Agent orchestration platform
│   ├── agentic-synth/         # Synthesis tools
│   ├── arw-chrome-extension/  # ARW Inspector extension
│   ├── cli/                   # Command-line tools
│   ├── health-dashboard/      # System monitoring
│   ├── mcp-server/            # MCP server implementation
│   └── media-discovery/       # Next.js demo app
│
├── migrations/                # Database migrations
├── infrastructure/            # Deployment configs
├── tests/                     # Integration & E2E tests
└── docs/                      # Documentation
```

---

## Apps Overview

### AgentDB v2.0

High-performance AI memory database with vector search:

- **150x faster** than SQLite for vector operations
- HNSW indexing with WASM acceleration
- ReasoningBank for adaptive learning
- Causal memory graphs & reflexion patterns
- MCP integration with 20+ tools

```bash
cd apps/agentdb
npm install
npm run build
npx agentdb init
```

### Agentic-Flow

Production AI agent orchestration:

- 66 specialized agents
- 213 MCP tools
- ReasoningBank learning memory
- Distributed consensus protocols
- GitHub integration

```bash
cd apps/agentic-flow
npm install
npm run build
```

### MCP Server

Model Context Protocol server for AI agents:

```bash
cd apps/mcp-server
npm install
npm run build
npm run start:stdio  # For Claude Desktop
npm run start:sse    # For web integrations
```

---

## Getting Started

### Prerequisites

- Rust 1.75+ with cargo
- Node.js 18+
- PostgreSQL 15+
- Redis 7+
- Docker (optional, for local development)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/globalbusinessadvisors/media-gateway.git
cd media-gateway

# Install Rust dependencies
cargo build --workspace

# Install Node.js dependencies
npm install

# Set up environment
cp .env.example .env
# Edit .env with your configuration

# Run database migrations
cargo run -p mg-migrate

# Start services (development)
cargo run -p media-gateway-api      # API Gateway on :8080
cargo run -p media-gateway-discovery # Discovery on :8081
```

### Docker Development

```bash
# Start infrastructure services
docker-compose up -d postgres redis qdrant

# Run all services
docker-compose up
```

---

## API Reference

### REST Endpoints

```yaml
/api/v1:
  /search:
    GET /semantic         # Natural language search
    GET /autocomplete     # Search suggestions

  /content:
    GET /movies           # List movies
    GET /tv               # List TV shows
    GET /:id              # Content details

  /recommendations:
    GET /for-you          # Personalized recommendations
    GET /similar/:id      # Similar content

  /user:
    GET /profile          # User profile
    GET /watchlist        # User watchlist
    PATCH /preferences    # Update preferences
```

### MCP Tools

```typescript
// Available MCP tools for AI agents
semantic_search       // Natural language content search
get_recommendations   // Personalized suggestions
check_availability    // Platform availability check
get_content_details   // Full content metadata
list_devices         // User's registered devices
initiate_playback    // Start content on device
control_playback     // Play/pause/seek controls
update_preferences   // Modify user preferences
```

---

## Performance Targets

| Operation | Target | SLA |
|-----------|--------|-----|
| Content Lookup | 20ms | p95 |
| Keyword Search | 200ms | p95 |
| NL Search | 350ms | p95 |
| SONA Inference | 2ms | p95 |
| Cross-Device Sync | 50ms | p95 |
| JWT Validation | 3ms | p95 |

### Infrastructure Targets

- **Availability**: 99.9% uptime
- **API Gateway**: 10,000 RPS capacity
- **Database**: 50K QPS (PostgreSQL)
- **Cache**: 200K QPS (Redis)
- **Cost**: <$4,000/month at 100K users

---

## Development

### Building

```bash
# Build all Rust crates
cargo build --workspace

# Build with optimizations
cargo build --release --workspace

# Build TypeScript apps
cd apps/agentdb && npm run build
cd apps/agentic-flow && npm run build
cd apps/mcp-server && npm run build
```

### Testing

```bash
# Run Rust tests
cargo test --workspace

# Run with offline SQLx (CI mode)
SQLX_OFFLINE=true cargo test --workspace

# Run TypeScript tests
cd apps/agentdb && npm test
```

### Linting

```bash
# Rust
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check

# TypeScript
npm run lint
```

---

## SPARC Development Methodology

This project follows the **SPARC** methodology with Claude-Flow orchestration:

1. **Specification** - Requirements analysis
2. **Pseudocode** - Algorithm design
3. **Architecture** - System design
4. **Refinement** - TDD implementation
5. **Completion** - Integration & hardening

Progress is tracked through batch task files in `plans/batches/`:
- BATCH_001-012 completed
- Focus areas: Core services, Auth, Search, SONA, Sync, Testing

---

## MCP Integration (Claude Desktop)

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "media-gateway": {
      "command": "node",
      "args": ["apps/mcp-server/dist/index.js", "--transport", "stdio"]
    }
  }
}
```

---

## Contributing

See [CLAUDE.md](CLAUDE.md) for development guidelines including:
- SPARC methodology for systematic development
- Concurrent execution patterns
- File organization rules
- Agent coordination protocols

---

## License

This project is licensed under the [MIT License](LICENSE).

---

## Links

- **Repository**: [github.com/globalbusinessadvisors/media-gateway](https://github.com/globalbusinessadvisors/media-gateway)
- **Documentation**: [/docs](./docs)
- **Architecture**: [/src/ARCHITECTURE_CONTEXT.md](./src/ARCHITECTURE_CONTEXT.md)

---

<div align="center">

**Media Gateway** - AI-Native Entertainment Discovery

*Solving the 45-minute decision problem*

</div>
