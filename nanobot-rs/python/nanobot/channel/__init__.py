"""
Channel Adapters - Connect various chat platforms to nanobot
"""

import asyncio
from abc import ABC, abstractmethod
from typing import Optional, Dict, Any, Callable
from dataclasses import dataclass


@dataclass
class Message:
    """Unified message format across all channels"""

    channel: str
    user_id: str
    content: str
    metadata: Dict[str, Any] = None


class BaseChannel(ABC):
    """
    Base class for channel adapters.

    Each channel implementation should:
    1. Handle platform-specific authentication
    2. Convert platform messages to unified format
    3. Send responses back to the platform
    """

    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self._running = False
        self._message_handler: Optional[Callable] = None

    @abstractmethod
    async def start(self):
        """Start the channel"""
        pass

    @abstractmethod
    async def stop(self):
        """Stop the channel"""
        pass

    @abstractmethod
    async def send(self, user_id: str, content: str):
        """Send a message to a user"""
        pass

    def on_message(self, handler: Callable):
        """Set message handler"""
        self._message_handler = handler

    async def _handle_message(self, message: Message):
        """Internal message handler"""
        if self._message_handler:
            await self._message_handler(message)


class TelegramChannel(BaseChannel):
    """Telegram bot channel"""

    async def start(self):
        token = self.config.get("token")
        if not token:
            raise ValueError("Telegram token required")

        try:
            from telegram import Update
            from telegram.ext import Application, ContextTypes, MessageHandler, filters
        except ImportError:
            raise ImportError("python-telegram-bot not installed")

        self.app = Application.builder().token(token).build()

        async def echo(context: ContextTypes.DEFAULT_TYPE):
            update = context.update
            if update.message:
                msg = Message(
                    channel="telegram",
                    user_id=str(update.effective_user.id),
                    content=update.message.text,
                )
                await self._handle_message(msg)

        self.app.add_handler(MessageHandler(filters.TEXT & ~filters.COMMAND, echo))
        await self.app.run_polling()
        self._running = True

    async def stop(self):
        if hasattr(self, "app"):
            await self.app.stop()
        self._running = False

    async def send(self, user_id: str, content: str):
        if hasattr(self, "app"):
            await self.app.bot.send_message(chat_id=user_id, text=content)


class DiscordChannel(BaseChannel):
    """Discord bot channel"""

    async def start(self):
        token = self.config.get("token")
        if not token:
            raise ValueError("Discord token required")

        try:
            import discord
        except ImportError:
            raise ImportError("discord.py not installed")

        intents = discord.Intents.default()
        intents.message_content = True

        class Bot(discord.Client):
            async def on_message(self, message):
                if message.author == self.user:
                    return
                msg = Message(
                    channel="discord",
                    user_id=str(message.author.id),
                    content=message.content,
                )
                await self._handle_message(msg)

        self.bot = Bot(intents=intents)
        await self.bot.start(token)
        self._running = True

    async def stop(self):
        if hasattr(self, "bot"):
            await self.bot.close()
        self._running = False

    async def send(self, user_id: str, content: str):
        pass


class FeishuChannel(BaseChannel):
    """Feishu/Lark bot channel"""

    async def start(self):
        app_id = self.config.get("app_id")
        app_secret = self.config.get("app_secret")

        if not app_id or not app_secret:
            raise ValueError("Feishu app_id and app_secret required")

        try:
            from lark_oapi import WxWork, AccessToken, IM
        except ImportError:
            raise ImportError("lark-sdk-python not installed")

        self.client = WxWork.create_client()
        self._running = True

    async def stop(self):
        self._running = False

    async def send(self, user_id: str, content: str):
        pass


class CLIAgentChannel(BaseChannel):
    """Interactive CLI channel for direct interaction"""

    async def start(self):
        self._running = True
        print("CLI Channel started. Type 'quit' to exit.")

        while self._running:
            content = input("You: ")
            if content.lower() == "quit":
                break

            msg = Message(
                channel="cli",
                user_id="local_user",
                content=content,
            )
            await self._handle_message(msg)

    async def stop(self):
        self._running = False

    async def send(self, user_id: str, content: str):
        print(f"Bot: {content}")


CHANNEL_REGISTRY = {
    "telegram": TelegramChannel,
    "discord": DiscordChannel,
    "feishu": FeishuChannel,
    "cli": CLIAgentChannel,
}


def create_channel(channel_type: str, config: Dict[str, Any]) -> BaseChannel:
    """Factory function to create channel instances"""
    channel_class = CHANNEL_REGISTRY.get(channel_type)
    if not channel_class:
        raise ValueError(f"Unknown channel type: {channel_type}")
    return channel_class(config)
