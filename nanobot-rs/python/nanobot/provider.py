"""
LLM Provider Registry and Adapters

Provides unified interface for various LLM providers:
- OpenAI, Anthropic, Google Gemini
- OpenRouter, LiteLLM
- DeepSeek, Groq, Ollama, etc.
"""

import os
import json
from typing import Optional, Dict, Any, List
from abc import ABC, abstractmethod
from dataclasses import dataclass

try:
    import litellm
    LITELLM_AVAILABLE = True
except ImportError:
    LITELLM_AVAILABLE = False


@dataclass
class ChatMessage:
    role: str
    content: str


class BaseProvider(ABC):
    """Base class for LLM providers"""
    
    @abstractmethod
    def chat(self, messages: List[ChatMessage], **kwargs) -> str:
        """Send chat request and return response"""
        pass
    
    @abstractmethod
    def chat_stream(self, messages: List[ChatMessage], **kwargs):
        """Stream chat response"""
        pass


class LiteLLMProvider(BaseProvider):
    """
    Provider using litellm for unified API access.
    Supports 100+ LLM providers including:
    - OpenAI, Anthropic, Google Gemini
    - OpenRouter, Azure OpenAI
    - DeepSeek, Groq, Ollama
    - And many more...
    """
    
    def __init__(
        self,
        model: str = "claude-3-haiku",
        api_key: Optional[str] = None,
        base_url: Optional[str] = None,
        **kwargs
    ):
        if not LITELLM_AVAILABLE:
            raise ImportError("litellm not installed. Run: pip install litellm")
        
        self.model = model
        self.api_key = api_key or os.getenv("OPENAI_API_KEY")
        self.base_url = base_url
        self.extra_kwargs = kwargs
        
    def chat(self, messages: List[ChatMessage], **kwargs) -> str:
        messages_dict = [{"role": m.role, "content": m.content} for m in messages]
        
        response = litellm.completion(
            model=self.model,
            messages=messages_dict,
            api_key=self.api_key,
            base_url=self.base_url,
            **{**self.extra_kwargs, **kwargs}
        )
        
        return response.choices[0].message.content
    
    def chat_stream(self, messages: List[ChatMessage], **kwargs):
        messages_dict = [{"role": m.role, "content": m.content} for m in messages]
        
        return litellm.completion(
            model=self.model,
            messages=messages_dict,
            api_key=self.api_key,
            base_url=self.base_url,
            stream=True,
            **{**self.extra_kwargs, **kwargs}
        )


class OpenAIProvider(BaseProvider):
    """OpenAI official provider"""
    
    def __init__(self, api_key: Optional[str] = None, **kwargs):
        try:
            from openai import OpenAI
        except ImportError:
            raise ImportError("openai not installed. Run: pip install openai")
        
        self.client = OpenAI(api_key=api_key or os.getenv("OPENAI_API_KEY"))
        
    def chat(self, messages: List[ChatMessage], **kwargs) -> str:
        response = self.client.chat.completions.create(
            model="gpt-4o",
            messages=[{"role": m.role, "content": m.content} for m in messages],
            **kwargs
        )
        return response.choices[0].message.content


class AnthropicProvider(BaseProvider):
    """Anthropic Claude provider"""
    
    def __init__(self, api_key: Optional[str] = None, **kwargs):
        try:
            from anthropic import Anthropic
        except ImportError:
            raise ImportError("anthropic not installed. Run: pip install anthropic")
        
        self.client = Anthropic(api_key=api_key or os.getenv("ANTHROPIC_API_KEY"))
        
    def chat(self, messages: List[ChatMessage], **kwargs) -> str:
        system = ""
        filtered = []
        for m in messages:
            if m.role == "system":
                system = m.content
            else:
                filtered.append(m)
        
        response = self.client.messages.create(
            model="claude-3-5-sonnet-20241022",
            system=system,
            messages=[{"role": m.role, "content": m.content} for m in filtered],
            **kwargs
        )
        return response.content[0].text


class ProviderRegistry:
    """
    Registry for LLM providers.
    
    Usage:
        registry = ProviderRegistry()
        registry.register("openai", OpenAIProvider)
        registry.register("anthropic", AnthropicProvider)
        
        provider = registry.get("openai")()
    """
    
    _providers: Dict[str, type] = {}
    
    @classmethod
    def register(cls, name: str, provider_class: type):
        """Register a provider class"""
        cls._providers[name] = provider_class
        
    @classmethod
    def get(cls, name: str) -> type:
        """Get a registered provider class"""
        if name == "litellm":
            return LiteLLMProvider
        if name == "openai":
            return OpenAIProvider
        if name == "anthropic":
            return AnthropicProvider
        return cls._providers.get(name, LiteLLMProvider)
    
    @classmethod
    def list_providers(cls) -> List[str]:
        """List all registered providers"""
        return list(cls._providers.keys())


ProviderRegistry.register("openai", OpenAIProvider)
ProviderRegistry.register("anthropic", AnthropicProvider)
ProviderRegistry.register("litellm", LiteLLMProvider)
