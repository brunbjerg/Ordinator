use actix::Message;
use serde::{Deserialize, Serialize};

use crate::{resources::Id, LevelOfDetail, LogLevel};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrchestratorRequest {
    GetWorkOrderStatus(u32, LevelOfDetail),
    GetWorkOrdersState(LevelOfDetail),
    GetPeriods,
    GetDays,
    GetAgentStatus,
    CreateSupervisorAgent(Id),
    DeleteSupervisorAgent(String),
    CreateOperationalAgent(Id),
    DeleteOperationalAgent(String),
    SetLogLevel(LogLevel),
    SetProfiling(LogLevel),
    Export,
}

impl Message for OrchestratorRequest {
    type Result = String;
}
