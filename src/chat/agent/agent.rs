use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Ok, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::RwLock;

// 상담원 정보
#[derive(Debug, Clone, Serialize)]
pub struct Agent {
    pub agent_id: String,
    pub name: String,
    pub status: AgentStatus,
    pub active_room_id: Option<String>, // 현재 상담 중인 방
    pub last_active: DateTime<Utc>,
}

impl Agent {
    pub fn is_available(&self) -> bool {
        self.status == AgentStatus::Available
    }

    pub fn update_agent_status(&mut self, agent_status: AgentStatus) {
        self.status = agent_status;
        self.last_active = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum AgentStatus {
    Available,
    Busy,
    Away,
}

#[derive(Debug, Clone)]
pub struct Agents {
    agents: Arc<RwLock<HashMap<i64, Agent>>>,
}

impl Agents {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn find_available_agent(&self) -> Option<i64> {
        let agents = self.agents.read().await;
        agents
            .iter()
            .find(|(_, agent)| agent.status == AgentStatus::Available)
            .map(|(agent_id, _)| agent_id.clone())
    }

    pub async fn update_agent_status(&self, agent_id: i64, status: AgentStatus) -> Result<i64> {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(&agent_id) {
            agent.update_agent_status(status);
            Ok(agent_id)
        } else {
            Err(anyhow!("Can't find agent : {}", agent_id))
        }
    }
}
