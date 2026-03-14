use xiao_claw::{Agent, AgentConfig, ToolRegistry, providers::{OpenAIProvider, AnthropicProvider, OpenRouterProvider, LLMProvider}};
use std::sync::Arc;
use std::env;

#[tokio::main]
async fn main() {
    println!("🤖 XiaoClaw - AI Agent");
    println!("=====================\n");
    
    let api_key = env::var("OPENAI_API_KEY").or_else(|_| env::var("ANTHROPIC_API_KEY").or_else(|_| env::var("OPENROUTER_API_KEY")))
        .expect("Please set OPENAI_API_KEY, ANTHROPIC_API_KEY, or OPENROUTER_API_KEY");
    
    let provider: Arc<dyn LLMProvider>;
    let model: String;
    
    if env::var("OPENAI_API_KEY").is_ok() {
        println!("Using OpenAI Provider");
        provider = Arc::new(OpenAIProvider::new(api_key));
        model = "gpt-4o-mini".to_string();
    } else if env::var("ANTHROPIC_API_KEY").is_ok() {
        println!("Using Anthropic Provider");
        provider = Arc::new(AnthropicProvider::new(api_key));
        model = "claude-3-haiku-20240307".to_string();
    } else {
        println!("Using OpenRouter Provider");
        provider = Arc::new(OpenRouterProvider::new(api_key));
        model = "openai/gpt-4o-mini".to_string();
    }
    
    let config = AgentConfig {
        model,
        provider: "default".to_string(),
        temperature: 0.7,
        max_tokens: Some(4096),
        system_prompt: Some("You are a helpful AI assistant.".into()),
        tools: vec![],
    };
    
    let tools = Arc::new(ToolRegistry::new());
    let agent = xiao_claw::Agent::new(config, tools);
    
    println!("\n✅ Ready! Type 'quit' to exit.\n");
    
    loop {
        print!("You: ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        let input = input.trim();
        if input == "quit" || input == "exit" {
            break;
        }
        
        if input.is_empty() {
            continue;
        }
        
        match agent.process(input).await {
            Ok(response) => {
                println!("Bot: {}\n", response.content);
            }
            Err(e) => {
                println!("Error: {}\n", e);
            }
        }
    }
    
    println!("Goodbye!");
}
