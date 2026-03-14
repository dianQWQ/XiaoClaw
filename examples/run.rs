use xiao_claw::{Agent, AgentConfig, ToolRegistry, providers::{ZhipuProvider}};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("🤖 XiaoClaw - AI Agent");
    println!("=====================\n");
    
    // 智谱AI (Zhipu/GLM)
    let api_key = "32927116eed74c7caa3de62f4cc6c470.3DZZDNjYa0gnuXzw";
    let model = "glm-4-flash";
    
    println!("Using Zhipu Provider (GLM-4-Flash)");
    
    let provider = Arc::new(ZhipuProvider::new(api_key.to_string()));
    
    let config = AgentConfig {
        model: model.to_string(),
        provider: "zhipu".to_string(),
        temperature: 0.7,
        max_tokens: Some(2048),
        system_prompt: Some("你是一个helpful的AI助手，用中文回答。".into()),
        tools: vec![],
    };
    
    let tools = Arc::new(ToolRegistry::new());
    let mut agent = xiao_claw::Agent::new(config, tools);
    agent.set_provider(provider);
    
    println!("✅ Sending test message...\n");
    
    match agent.process("你好，请介绍一下你自己").await {
        Ok(response) => {
            println!("Bot: {}\n", response.content);
        }
        Err(e) => {
            println!("Error: {}\n", e);
        }
    }
    
    println!("Done!");
}
