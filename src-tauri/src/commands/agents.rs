use std::collections::HashMap;
use crate::llm::agents::{
    executor::CommandExecutor,
    types::{Agent, AgentManager},
};

#[tauri::command]
pub async fn add_agent(agent: Agent) -> Result<(), String> {
    let manager = AgentManager::new().map_err(|e| e.to_string())?;
    manager.save_agent(&agent).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_agent(name: String) -> Result<(), String> {
    let mut manager = AgentManager::new().map_err(|e| e.to_string())?;
    manager.remove_agent(&name).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_agent(name: String) -> Result<Option<Agent>, String> {
    let manager = AgentManager::new().map_err(|e| e.to_string())?;
    Ok(manager.get_agent(&name).cloned())
}

#[tauri::command]
pub async fn list_agents() -> Result<Vec<Agent>, String> {
    let manager = AgentManager::new().map_err(|e| e.to_string())?;
    Ok(manager.list_agents().into_iter().cloned().collect())
}

#[tauri::command]
pub async fn execute_agent_command(
    agent_name: String,
    command: String,
    env: Option<HashMap<String, String>>,
) -> Result<String, String> {
    let manager = AgentManager::new().map_err(|e| e.to_string())?;
    let agent = manager.get_agent(&agent_name).ok_or("Agent not found")?;

    let mut command_env = agent.env.clone();
    if let Some(additional_env) = env {
        command_env.extend(additional_env);
    }

    let executor = CommandExecutor::new(command_env);
    executor.execute(&command).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_agent(old_name: String, agent: Agent) -> Result<(), String> {
    let mut manager = AgentManager::new().map_err(|e| e.to_string())?;
    manager
        .update_agent(&old_name, agent)
        .map_err(|e| e.to_string())
} 