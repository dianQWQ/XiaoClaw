use xiao_claw::{Agent, AgentConfig, ToolRegistry};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("🤖 XiaoClaw - AI Agent");
    println!("=====================\n");
    
    let config = AgentConfig {
        model: "claude-3-haiku".into(),
        provider: "openai".into(),
        temperature: 0.7,
        max_tokens: Some(4096),
        system_prompt: Some("You are a helpful AI assistant.".into()),
        tools: vec![],
    };
    
    let tools = Arc::new(ToolRegistry::new());
    let _agent = Agent::new(config, tools);
    
    println!("✅ XiaoClaw initialized!");
    println!("\nNote: LLM provider not configured. Configure a provider to enable chat.");
    println!("Use Python FFI or implement a custom LLM provider.");
}
