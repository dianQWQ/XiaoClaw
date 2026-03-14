use std::fs;
use std::path::Path;

#[derive(Clone)]
pub struct ContextBuilder {
    workspace: Option<String>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self { workspace: None }
    }

    pub fn with_workspace(mut self, workspace: impl Into<String>) -> Self {
        self.workspace = Some(workspace.into());
        self
    }

    pub fn build(&self) -> Context {
        let soul = self.load_template("SOUL.md");
        let user = self.load_template("USER.md");
        let agents = self.load_template("AGENTS.md");
        let heartbeat = self.load_template("HEARTBEAT.md");

        Context {
            soul,
            user,
            agents,
            heartbeat,
        }
    }

    fn load_template(&self, name: &str) -> Option<String> {
        let workspace = self.workspace.as_ref()?;
        let path = Path::new(workspace).join("templates").join(name);
        fs::read_to_string(path).ok()
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub soul: Option<String>,
    pub user: Option<String>,
    pub agents: Option<String>,
    pub heartbeat: Option<String>,
}

impl Context {
    pub fn to_prompt(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref soul) = self.soul {
            parts.push(format!("# Soul\n{}", soul));
        }

        if let Some(ref user) = self.user {
            parts.push(format!("# User Context\n{}", user));
        }

        if let Some(ref agents) = self.agents {
            parts.push(format!("# Agents\n{}", agents));
        }

        if let Some(ref heartbeat) = self.heartbeat {
            parts.push(format!("# Heartbeat\n{}", heartbeat));
        }

        parts.join("\n\n")
    }
}
