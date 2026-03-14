"""
Nanobot Client - High-level Python API
"""

import json
import asyncio
from typing import Optional, Dict, Any, List, Callable
from dataclasses import dataclass, field

try:
    from nanobot_core import PyAgent, PyMemoryStore, PyToolRegistry, load_config
except ImportError:
    PyAgent = None


@dataclass
class AgentConfig:
    model: str = "claude-3-haiku"
    provider: str = "openai"
    temperature: float = 0.7
    max_tokens: Optional[int] = 4096
    system_prompt: Optional[str] = None
    tools: List[str] = field(default_factory=list)
    
    def to_json(self) -> str:
        return json.dumps({
            "model": self.model,
            "provider": self.provider,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
            "system_prompt": self.system_prompt,
            "tools": self.tools,
        })


class NanobotClient:
    """
    High-level client for interacting with nanobot agent.
    
    Usage:
        client = NanobotClient(config_path="~/.nanobot/config.json")
        response = await client.chat("Hello!")
    """
    
    def __init__(
        self,
        config_path: Optional[str] = None,
        agent_config: Optional[AgentConfig] = None,
        provider: Optional[Any] = None,
    ):
        if PyAgent is None:
            raise ImportError("nanobot-core not installed. Run: pip install nanobot-core")
        
        self.config = None
        if config_path:
            try:
                self.config = load_config(config_path)
            except Exception as e:
                print(f"Warning: Failed to load config: {e}")
        
        if agent_config is None:
            agent_config = AgentConfig()
        
        self._agent = PyAgent(agent_config.to_json())
        self._provider = provider
        self._memory = None
        self._tools = None
        
    def set_provider(self, provider: Any):
        """Set the LLM provider (e.g., LiteLLM wrapper)"""
        self._provider = provider
        
    def set_memory_store(self, db_path: str, max_messages: int = 100):
        """Enable memory persistence"""
        self._memory = PyMemoryStore(db_path, max_messages)
        
    def set_system_prompt(self, prompt: str):
        """Set the system prompt"""
        self._agent.set_system_prompt(prompt)
        
    def clear_history(self):
        """Clear conversation history"""
        self._agent.clear_messages()
        
    def chat(self, message: str) -> str:
        """
        Send a message and get response.
        This is a sync wrapper around async chat.
        """
        if self._provider:
            return self._provider.chat(message)
        return self._agent.process(message)
    
    async def chat_async(self, message: str) -> str:
        """Async version of chat"""
        return await asyncio.to_thread(self.chat, message)
    
    def add_tool(self, name: str, func: Callable, description: str, parameters: Dict):
        """
        Register a custom tool.
        
        Args:
            name: Tool name
            func: Callable that executes the tool
            description: Tool description
            parameters: JSON schema for parameters
        """
        pass


class NanobotGateway:
    """
    Gateway that connects multiple channels to the agent.
    Handles message routing between channels.
    """
    
    def __init__(self, client: NanobotClient):
        self.client = client
        self._channels: Dict[str, Any] = {}
        
    def register_channel(self, name: str, channel: Any):
        """Register a channel handler"""
        self._channels[name] = channel
        
    async def start(self):
        """Start all registered channels"""
        for name, channel in self._channels.items():
            asyncio.create_task(channel.start())
            
    async def stop(self):
        """Stop all channels"""
        for channel in self._channels.values():
            await channel.stop()
