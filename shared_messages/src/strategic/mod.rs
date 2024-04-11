pub mod strategic_periods_message;
pub mod strategic_resources_message;
pub mod strategic_scheduling_message;
pub mod strategic_status_message;

use std::fmt::{self};

use actix::Message;
use serde::{Deserialize, Serialize};

use crate::{agent_error::AgentError, Asset};

use self::{
    strategic_periods_message::StrategicTimeMessage,
    strategic_resources_message::{ManualResource, StrategicResourceMessage},
    strategic_scheduling_message::StrategicSchedulingMessage,
    strategic_status_message::StrategicStatusMessage,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "strategic_message_type")]
pub struct StrategicRequest {
    pub asset: Asset,
    pub strategic_request_message: StrategicRequestMessage,
}

impl StrategicRequest {
    pub fn asset(&self) -> &Asset {
        &self.asset
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StrategicRequestMessage {
    Status(StrategicStatusMessage),
    Scheduling(StrategicSchedulingMessage),
    Resources(StrategicResourceMessage),
    Periods(StrategicTimeMessage),
}

impl Message for StrategicRequestMessage {
    type Result = Result<String, AgentError>;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimePeriod {
    pub period_string: String,
}

impl TimePeriod {
    pub fn get_period_string(&self) -> String {
        self.period_string.clone()
    }
}
impl fmt::Display for StrategicRequestMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            StrategicRequestMessage::Status(strategic_status_message) => {
                write!(f, "status: {}", strategic_status_message)?;
                Ok(())
            }
            StrategicRequestMessage::Scheduling(scheduling_message) => {
                write!(f, "scheduling_message: {:?}", scheduling_message)?;

                Ok(())
            }
            StrategicRequestMessage::Resources(resources_message) => {
                for manual_resource in resources_message.get_manual_resources().iter() {
                    writeln!(f, "manual_resource: {:?}", manual_resource)?;
                }
                Ok(())
            }
            StrategicRequestMessage::Periods(period_message) => {
                write!(f, "period_message: {:?}", period_message)?;
                Ok(())
            }
        }
    }
}

impl fmt::Display for ManualResource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "resource: {:?}, period: {}, capacity: {}",
            self.resource, self.period.period_string, self.capacity
        )
    }
}

impl TimePeriod {
    pub fn new(period_string: String) -> Self {
        Self { period_string }
    }
}
