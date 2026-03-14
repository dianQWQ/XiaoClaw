use clap::{Parser, Subcommand};
use xiao_claw::{Agent, AgentConfig, ToolRegistry, providers::{OpenAIProvider, AnthropicProvider, OpenRouterProvider, ZhipuProvider, LLMProvider}};
use std::sync::Arc;
use std::env;

#[derive(Parser)]
#[command(name = "xiao-claw")]
#[command(about = "XiaoClaw - AI Agent CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive chat
    Chat {
        /// Provider: openai, anthropic, openrouter, zhipu
        #[arg(short, long, default_value = "zhipu")]
        provider: String,
        
        /// Model name
        #[arg(short, long)]
        model: Option<String>,
        
        /// System prompt
        #[arg(short, long)]
        system_prompt: Option<String>,
    },
    /// Send a single message
    Send {
        /// Your message
        message: String,
        
        /// Provider
        #[arg(short, long, default_value = "zhipu")]
        provider: String,
        
        /// Model
        #[arg(short, long)]
        model: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Chat { provider, model, system_prompt } => {
            chat_mode(provider, model, system_prompt).await?;
        }
        Commands::Send { message, provider, model } => {
            send_message(message, provider, model).await?;
        }
    }
    
    Ok(())
}

async fn chat_mode(provider: String, model: Option<String>, system_prompt: Option<String>) -> anyhow::Result<()> {
    println!("🤖 XiaoClaw - AI Agent");
    println!("=====================\n");
    
    let (provider, model, api_key) = get_provider_config(&provider, model)?;
    
    println!("Using {} Provider ({})", provider, model);
    println!("Type 'quit' or 'exit' to stop.\n");
    
    let config = AgentConfig {
        model: model.clone(),
        provider: provider.clone(),
        temperature: 0.7,
        max_tokens: Some(2048),
        system_prompt: system_prompt.or_else(|| Some("You are a helpful AI assistant.".to_string())),
        tools: vec![],
    };
    
    let tools = Arc::new(ToolRegistry::new());
    let agent = Agent::new(config, tools);
    agent.set_provider(create_provider(&provider, &model, &api_key)?);
    
    loop {
        print!("You: ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if input == "quit" || input == "exit" {
            break;
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
    Ok(())
}

async fn send_message(message: String, provider: String, model: Option<String>) -> anyhow::Result<()> {
    let (provider, model, api_key) = get_provider_config(&provider, model)?;
    
    let config = AgentConfig {
        model: model.clone(),
        provider: provider.clone(),
        temperature: 0.7,
        max_tokens: Some(2048),
        system_prompt: Some("You are a helpful AI assistant.".to_string()),
        tools: vec![],
    };
    
    let tools = Arc::new(ToolRegistry::new());
    let agent = Agent::new(config, tools);
    agent.set_provider(create_provider(&provider, &model, &api_key)?);
    
    match agent.process(&message).await {
        Ok(response) => {
            println!("{}", response.content);
        }
        Err(e) => {
            anyhow::bail!("Error: {}", e);
        }
    }
    
    Ok(())
}

fn get_provider_config(provider: &str, model: Option<String>) -> anyhow::Result<(String, String, String)> {
    let provider = provider.to_lowercase();
    
    match provider.as_str() {
        "openai" => {
            let api_key = env::var("OPENAI_API_KEY")
                .expect("Please set OPENAI_API_KEY environment variable");
            let model = model.unwrap_or_else(|| "gpt-4o-mini".to_string());
            Ok((provider, model, api_key))
        }
        "anthropic" => {
            let api_key = env::var("ANTHROPIC_API_KEY")
                .expect("Please set ANTHROPIC_API_KEY environment variable");
            let model = model.unwrap_or_else(|| "claude-3-haiku-20240307".to_string());
            Ok((provider, model, api_key))
        }
        "openrouter" => {
            let api_key = env::var("OPENROUTER_API_KEY")
                .expect("Please set OPENROUTER_API_KEY environment variable");
            let model = model.unwrap_or_else(|| "openai/gpt-4o-mini".to_string());
            Ok((provider, model, api_key))
        }
        "zhipu" => {
            let api_key = env::var("ZHIPU_API_KEY")
                .expect("Please set ZHIPU_API_KEY environment variable");
            let model = model.unwrap_or_else(|| "glm-4-flash".to_string());
            Ok((provider, model, api_key))
        }
        _ => {
            anyhow::bail!("Unknown provider: {}. Use: openai, anthropic, openrouter, zhipu", provider);
        }
    }
}

fn create_provider(provider: &str, model: &str, api_key: &str) -> anyhow::Result<Arc<dyn LLMProvider + Send + Sync>> {
    let provider = provider.to_lowercase();
    
    match provider.as_str() {
        "openai" => Ok(Arc::new(OpenAIProvider::new(api_key.to_string()))),
        "anthropic" => Ok(Arc::new(AnthropicProvider::new(api_key.to_string()))),
        "openrouter" => Ok(Arc::new(OpenRouterProvider::new(api_key.to_string()))),
        "zhipu" => Ok(Arc::new(ZhipuProvider::new(api_key.to_string()))),
        _ => anyhow::bail!("Unknown provider: {}", provider),
    }
}
