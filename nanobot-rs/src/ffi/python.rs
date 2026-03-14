#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyDict;

#[cfg(feature = "python")]
use crate::{
    agent::{Agent, AgentConfig, Message, MessageRole, Response},
    config::Config,
    memory::{MemoryEntry, MemoryStore},
    session::SessionManager,
    tools::{ToolRegistry, ToolResult, ToolInfo},
    Error,
};

#[cfg(feature = "python")]
#[pyclass]
pub struct PyAgent {
    inner: Agent,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyAgent {
    #[new]
    fn new(config_json: &str) -> PyResult<Self> {
        let config: AgentConfig = serde_json::from_str(config_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        
        let tools = Arc::new(ToolRegistry::new());
        let agent = Agent::new(config, tools);
        
        Ok(Self { inner: agent })
    }
    
    fn process(&self, input: &str) -> PyResult<String> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        runtime.block_on(async {
            self.inner.process(input).await
        })
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
        .content
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }
    
    fn set_system_prompt(&self, prompt: &str) {
        self.inner.set_system_prompt(prompt.to_string());
    }
    
    fn clear_messages(&self) {
        self.inner.clear_messages();
    }
}

#[cfg(feature = "python")]
#[pyclass]
pub struct PyMemoryStore {
    inner: MemoryStore,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyMemoryStore {
    #[new]
    fn new(db_path: &str, max_messages: usize) -> PyResult<Self> {
        MemoryStore::new(db_path, max_messages)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
            .map(|inner| Self { inner })
    }
    
    fn add(&self, session_id: &str, role: &str, content: &str) -> PyResult<()> {
        let entry = MemoryEntry::new(session_id, role, content);
        self.inner.add(entry)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }
    
    fn get_session(&self, session_id: &str, limit: Option<usize>) -> PyResult<Vec<String>> {
        self.inner.get_session(session_id, limit)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
            .map(|entries| {
                entries.into_iter()
                    .map(|e| serde_json::json!({
                        "role": e.role,
                        "content": e.content,
                    }).to_string())
                    .collect()
            })
    }
    
    fn search(&self, session_id: &str, query: &str) -> PyResult<Vec<String>> {
        self.inner.search(session_id, query)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
            .map(|entries| {
                entries.into_iter()
                    .map(|e| e.content)
                    .collect()
            })
    }
    
    fn clear_session(&self, session_id: &str) -> PyResult<()> {
        self.inner.clear_session(session_id)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }
}

#[cfg(feature = "python")]
#[pyclass]
pub struct PySessionManager {
    inner: SessionManager,
}

#[cfg(feature = "python")]
#[pymethods]
impl PySessionManager {
    #[new]
    fn new(timeout_minutes: u64) -> Self {
        Self {
            inner: SessionManager::new(timeout_minutes),
        }
    }
    
    fn get_or_create(&self, channel: &str, user_id: &str) -> String {
        let session = self.inner.get_or_create(channel, user_id);
        session.read().id.clone()
    }
    
    fn list_sessions(&self) -> Vec<String> {
        self.inner.list()
            .into_iter()
            .map(|s| serde_json::json!({
                "id": s.id,
                "channel": s.channel,
                "user_id": s.user_id,
            }).to_string())
            .collect()
    }
    
    fn cleanup_expired(&self) -> usize {
        self.inner.cleanup_expired()
    }
}

#[cfg(feature = "python")]
#[pyclass]
pub struct PyToolRegistry {
    inner: Arc<ToolRegistry>,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyToolRegistry {
    #[new]
    fn new() -> Self {
        Self {
            inner: Arc::new(ToolRegistry::new()),
        }
    }
    
    fn list_tools(&self) -> Vec<String> {
        self.inner.list_tools()
            .into_iter()
            .map(|t| serde_json::json!({
                "name": t.name,
                "description": t.description,
                "parameters": t.parameters,
            }).to_string())
            .collect()
    }
    
    fn execute(&self, name: &str, args_json: &str) -> PyResult<String> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        let args: serde_json::Value = serde_json::from_str(args_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        
        runtime.block_on(async {
            self.inner.execute(name, &args).await
        })
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        .map(|r| r.content)
    }
}

#[cfg(feature = "python")]
pub fn create_module() -> PyResult<PyModule> {
    let m = PyModule::new("nanobot_core")?;
    
    m.add_class::<PyAgent>()?;
    m.add_class::<PyMemoryStore>()?;
    m.add_class::<PySessionManager>()?;
    m.add_class::<PyToolRegistry>()?;
    
    m.add_function(wrap_pyfunction!(load_config, m)?)?;
    
    Ok(m)
}

#[cfg(feature = "python")]
#[pyfunction]
fn load_config(path: &str) -> PyResult<String> {
    crate::config::load_config(path)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
        .map(|c| serde_json::to_string_pretty(&c).unwrap())
}
