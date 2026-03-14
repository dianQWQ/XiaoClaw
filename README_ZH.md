# XiaoClaw

[English](./README.md)

---

> ⚡️ 使用 AI 重构的 [nanobot](https://github.com/HKUDS/nanobot) 版本，使用 Rust 重写以获得更好的性能。

超轻量级 AI Agent 核心，使用 Rust 实现，支持 Python FFI 绑定。

## 架构

```
┌─────────────────────────────────────────┐
│           Python 集成层                  │
│    (频道适配器, LLM 提供商封装)          │
└──────────────────┬──────────────────────┘
                   │ PyO3 FFI
┌──────────────────┴──────────────────────┐
│              Rust 核心                   │
│  - Agent 循环 (LLM + 工具调用)           │
│  - 上下文构建器                          │
│  - 工具注册表                           │
│  - LLM 提供商 (OpenAI/Anthropic/智谱)   │
│  - 记忆系统 (SQLite)                    │
│  - 会话管理                             │
└─────────────────────────────────────────┘
```

## 项目结构

```
XiaoClaw/
├── Cargo.toml              # Rust 项目配置
├── src/
│   ├── lib.rs             # 库入口
│   ├── agent/             # 核心 Agent 逻辑
│   ├── config/            # 配置系统
│   ├── tools/             # 工具系统
│   ├── providers/         # LLM 提供商
│   ├── memory/           # SQLite 记忆存储
│   ├── session/          # 会话管理
│   └── ffi/              # PyO3 绑定
├── examples/
│   └── run.rs            # CLI 示例
└── python/               # Python 模块
```

## 功能特性

- **核心 Agent 循环**: LLM 推理 + 工具调用（最多 20 次迭代）
- **工具系统**: 内置工具 + 自定义工具注册
- **LLM 提供商**: OpenAI, Anthropic, OpenRouter, 智谱AI
- **记忆系统**: 基于 SQLite 的持久化记忆
- **会话管理**: 用户会话跟踪
- **Python FFI**: PyO3 绑定

## 快速开始

```bash
git clone https://github.com/dianQWQ/XiaoClaw.git
cd XiaoClaw
export OPENAI_API_KEY=sk-...
cargo run --example run
```

## 支持的提供商

| 提供商 | 环境变量 | 模型示例 |
|--------|----------|----------|
| OpenAI | `OPENAI_API_KEY` | `gpt-4o`, `gpt-4o-mini` |
| Anthropic | `ANTHROPIC_API_KEY` | `claude-3-5-sonnet` |
| OpenRouter | `OPENROUTER_API_KEY` | `openai/gpt-4o` |
| 智谱AI | `ZHIPU_API_KEY` | `glm-4-flash` |

## 构建

```bash
cargo build --release
```

## 使用方式

```rust
use xiao_claw::{Agent, AgentConfig, ToolRegistry, providers::ZhipuProvider};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = AgentConfig {
        model: "glm-4-flash".into(),
        provider: "zhipu".into(),
        temperature: 0.7,
        max_tokens: Some(2048),
        system_prompt: Some("你是一个有用的AI助手。".into()),
        tools: vec![],
    };
    
    let tools = Arc::new(ToolRegistry::new());
    let agent = Agent::new(config, tools);
    
    let provider = Arc::new(ZhipuProvider::new("你的-api-key".to_string()));
    agent.set_provider(provider);
    
    let response = agent.process("你好！").await;
    println!("回复: {:?}", response);
}
```

## 许可证

MIT
