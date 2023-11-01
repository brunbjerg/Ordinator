pub mod scheduler_message;
pub mod scheduler_algorithm;
pub mod display;

use std::collections::HashMap;
use actix::prelude::*; 
use actix::Message;
use priority_queue::PriorityQueue;
use std::hash::Hash;
use tracing::{event};
use tokio::time::{sleep, Duration};


use crate::models::work_order::priority::Priority;
use crate::models::work_order::order_type::WorkOrderType;
use crate::agents::scheduler_agent::scheduler_message::{SetAgentAddrMessage, SchedulerMessages, InputMessage};
use crate::models::scheduling_environment::WorkOrders;
use crate::models::order_period::OrderPeriod;
use crate::models::period::Period;
use crate::api::websocket_agent::WebSocketAgent;
use crate::agents::scheduler_agent::scheduler_algorithm::QueueType;
use crate::models::work_order::status_codes::MaterialStatus;
use crate::api::websocket_agent::SchedulerFrontendMessage;

pub struct SchedulerAgent {
    platform: String,
    scheduler_agent_algorithm: SchedulerAgentAlgorithm,
    ws_agent_addr: Option<Addr<WebSocketAgent>>,
}

pub struct SchedulerAgentAlgorithm {
    manual_resources_capacity : HashMap<(String, Period), f64>,
    manual_resources_loading: HashMap<(String, Period), f64>,
    backlog: WorkOrders,
    priority_queues: PriorityQueues<u32, u32>,
    scheduled_work_orders: HashMap<u32, OrderPeriod>,
    periods: Vec<Period>,
}


impl SchedulerAgent {
    pub fn set_ws_agent_addr(&mut self, ws_agent_addr: Addr<WebSocketAgent>) {
        self.ws_agent_addr = Some(ws_agent_addr);
    }

    // TODO: Here the other Agents Addr messages will also be handled.
}

impl Actor for SchedulerAgent {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        self.populate_priority_queues();
        // for _ in 0..self.scheduler_agent_algorithm.priority_queues.normal.len() {
        //     let work_order = self.scheduler_agent_algorithm.priority_queues.normal.pop();
        //     info!("SchedulerAgent normal queue is populated with: {}", work_order.unwrap().0 );
        // }
        // for _ in 0..self.scheduler_agent_algorithm.priority_queues.unloading.len() {
        //     let work_order = self.scheduler_agent_algorithm.priority_queues.unloading.pop(); // pop here removes the element from the queue
        //     info!("SchedulerAgent unloading queue is populated with: {}", work_order.unwrap().0 );
        // }

        ctx.notify(ScheduleIteration {})
    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {
        println!("SchedulerAgent is stopped");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct ScheduleIteration {}


/// I think that the priotity queue should be a struct that is a member of the scheduler agent.
impl Handler<ScheduleIteration> for SchedulerAgent {

    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: ScheduleIteration, ctx: &mut Self::Context) -> Self::Result {
        event!(tracing::Level::INFO , "A round of scheduling has been triggered");
        self.schedule_work_orders_by_type(QueueType::Normal);
        self.schedule_work_orders_by_type(QueueType::Unloading);

        let display_manual_resources = display::DisplayableManualResource(self.scheduler_agent_algorithm.manual_resources_capacity.clone());
        let display_scheduled_work_orders = display::DisplayableScheduledWorkOrders(self.scheduler_agent_algorithm.scheduled_work_orders.clone());

        // println!("manual resources {}", display_manual_resources);
        // println!("Scheduled work orders {}", display_scheduled_work_orders);
        let actor_addr = ctx.address().clone();

        let fut = async move {
            sleep(Duration::from_secs(1)).await;
            

            actor_addr.do_send(ScheduleIteration {});
        };

        ctx.notify(MessageToFrontend {});


        Box::pin(actix::fut::wrap_future::<_, Self>(fut))
    }
}

struct MessageToFrontend {}

impl Message for MessageToFrontend {

    type Result = ();
}

impl Handler<MessageToFrontend> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, msg: MessageToFrontend, ctx: &mut Self::Context) -> Self::Result {
        let scheduling_overview_data = self.extract_state_to_scheduler_overview().clone();

        let scheduler_frontend_message = SchedulerFrontendMessage {
            frontend_message_type: "frontend_scheduler_overview".to_string(),
            scheduling_overview_data: scheduling_overview_data,
        };
        match self.ws_agent_addr.as_ref() {
            Some(ws_agent) => {ws_agent.do_send(scheduler_frontend_message)}
            None => {println!("The websocket agent address is not set")}
        }
       
    }
}

impl Handler<SchedulerMessages> for SchedulerAgent {
    type Result = ();
    fn handle(&mut self, msg: SchedulerMessages, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SchedulerMessages::Input(msg) => {
                println!("SchedulerAgentReceived a FrontEnd message");
                let input_message: InputMessage = msg.into();
               
                self.update_scheduler_state(input_message);

                // TODO - modify state of scheduler agent
            }
            SchedulerMessages::WorkPlanner(msg) => {
               println!("SchedulerAgentReceived a WorkPlannerMessage message");
            },
            SchedulerMessages::ExecuteIteration => {
                // TODO - execute one optimization iteration of the scheduler agent
                self.execute_iteration(ctx);
            }
        }	
    }
}

impl Handler<SetAgentAddrMessage<WebSocketAgent>> for SchedulerAgent {
    type Result = ();

    fn handle(&mut self, msg: SetAgentAddrMessage<WebSocketAgent>, ctx: &mut Self::Context) -> Self::Result {
        self.set_ws_agent_addr(msg.addr);
    }
}

impl SchedulerAgent {
    pub fn execute_iteration(&mut self, ctx: &mut <SchedulerAgent as Actor>::Context) {

        println!("I am running a single iteration");  
        ctx.notify(SchedulerMessages::ExecuteIteration)
    }
}

impl SchedulerAgent {
    pub fn new(
        platform: String, 
        scheduler_agent_algorithm: SchedulerAgentAlgorithm,
        ws_agent_addr: Option<Addr<WebSocketAgent>>) 
            -> Self {
  
        Self {
            platform,
            scheduler_agent_algorithm,
            ws_agent_addr,
        }
    }
}

impl SchedulerAgentAlgorithm {
    pub fn new(
        manual_resources_capacity: HashMap<(String, Period), f64>, 
        manual_resources_loading: HashMap<(String, Period), f64>, 
        backlog: WorkOrders, 
        priority_queues: PriorityQueues<u32, u32>,
        scheduled_work_orders: HashMap<u32, OrderPeriod>, 
        periods: Vec<Period>,
    ) -> Self {
        SchedulerAgentAlgorithm {
            manual_resources_capacity,
            manual_resources_loading,
            backlog,
            priority_queues,
            scheduled_work_orders,
            periods            
        }
    }
}


impl SchedulerAgent {
    pub fn update_scheduler_state(&mut self, input_message: InputMessage) {
        dbg!(self.scheduler_agent_algorithm.manual_resources_capacity.clone());
        self.scheduler_agent_algorithm.manual_resources_capacity = input_message.get_manual_resources();
        dbg!(self.scheduler_agent_algorithm.manual_resources_capacity.clone());
    }
}


impl SchedulerAgent {

    fn populate_priority_queues(&mut self) -> () {
        for (key, work_order) in self.scheduler_agent_algorithm.backlog.inner.iter() {
            if work_order.unloading_point.present {
                event!(tracing::Level::INFO , "Work order {} has been added to the unloading queue", key);
                self.scheduler_agent_algorithm.priority_queues.unloading.push(*key, work_order.order_weight);
            } else if work_order.revision.shutdown || work_order.vendor {
                event!(tracing::Level::INFO , "Work order {} has been added to the shutdown/vendor queue", key);
                self.scheduler_agent_algorithm.priority_queues.shutdown_vendor.push(*key, work_order.order_weight);
            } else {
                event!(tracing::Level::INFO , "Work order {} has been added to the normal queue", key);

                self.scheduler_agent_algorithm.priority_queues.normal.push(*key, work_order.order_weight);
            }
        }
    }


}


pub struct PriorityQueues<T, P> 
    where T: Hash + Eq,
          P: Ord
{ 
    unloading: PriorityQueue<T, P>,
    shutdown_vendor: PriorityQueue<T, P>,
    normal: PriorityQueue<T, P>,
}

impl PriorityQueues<u32, u32> {
    pub fn new() -> Self{
        Self {
            unloading: PriorityQueue::<u32, u32>::new(),
            shutdown_vendor: PriorityQueue::<u32, u32>::new(),
            normal: PriorityQueue::<u32, u32>::new(),
        }
    }
}

#[derive(Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SchedulingOverviewData {
    scheduled_period: String,
    scheduled_start: String,
    unloading_point: String,
    material_date: String,
    work_order_number: u32,
    activity: String,
    work_center: String,
    work_remaining: String,
    number: u32,
    notes_1: String,
    notes_2: String,
    order_description: String,
    object_description: String,
    order_user_status: String,
    order_system_status: String,
    functional_location: String,
    revision: String,
    earliest_start_datetime: String,
    earliest_finish_datetime: String,
    earliest_allowed_starting_date: String,
    latest_allowed_finish_date: String,
    order_type: String,
    priority: String,
}



impl SchedulerAgent {
    fn extract_state_to_scheduler_overview(&self) -> Vec<SchedulingOverviewData> {
        let mut scheduling_overview_data: Vec<SchedulingOverviewData> = Vec::new();
        for (work_order_number, work_order) in self.scheduler_agent_algorithm.backlog.inner.iter() {
            for (operation_number, operation) in work_order.operations.clone() {
                let scheduling_overview_data_item = SchedulingOverviewData {
                    scheduled_period: match self.scheduler_agent_algorithm.scheduled_work_orders.get(work_order_number) {
                        Some(order_period) => order_period.period.period_string.clone(),
                        None => "not scheduled".to_string(),
                    },
                    scheduled_start: work_order.order_dates.basic_start_date.to_string(),
                    unloading_point: work_order.unloading_point.clone().string, 

                    material_date: match work_order.status_codes.material_status {
                        MaterialStatus::Smat => "SMAT".to_string(),
                        MaterialStatus::Nmat => "NMAT".to_string(),
                        MaterialStatus::Cmat => "CMAT".to_string(),
                        MaterialStatus::Wmat => "WMAT".to_string(),
                        MaterialStatus::Pmat => "PMAT".to_string(),
                        MaterialStatus::Unknown => "Implement control tower".to_string(),
                    },
                    
                    work_order_number: work_order_number.clone(),
                    activity: operation_number.clone().to_string(),
                    work_center: operation.work_center.clone(),
                    work_remaining: operation.work_remaining.to_string(),
                    number: operation.number,
                    notes_1: work_order.order_text.notes_1.clone(),
                    notes_2: work_order.order_text.notes_2.clone().to_string(),
                    order_description: work_order.order_text.order_description.clone(),
                    object_description: work_order.order_text.object_description.clone(),
                    order_user_status: work_order.order_text.order_user_status.clone(),
                    order_system_status: work_order.order_text.order_system_status.clone(),
                    functional_location: work_order.functional_location.clone().string,
                    revision: work_order.revision.clone().string,
                    earliest_start_datetime: operation.earliest_start_datetime.to_string(),
                    earliest_finish_datetime: operation.earliest_finish_datetime.to_string(),
                    earliest_allowed_starting_date: work_order.order_dates.earliest_allowed_start_date.to_string(),
                    latest_allowed_finish_date: work_order.order_dates.latest_allowed_finish_date.to_string(),
                    order_type: match work_order.order_type.clone() {
                        WorkOrderType::WDF(wdf_priority) => "WDF".to_string(),
                        WorkOrderType::WGN(wgn_priority) => "WGN".to_string(),
                        WorkOrderType::WPM(wpm_priority) => "WPM".to_string(),
                        WorkOrderType::Other => "Missing Work Order Type".to_string(),
                    },
                    priority: match work_order.priority.clone() {
                        Priority::IntValue(i) => i.to_string(),
                        Priority::StringValue(s) => s.to_string(),
                    },
                };
                scheduling_overview_data.push(scheduling_overview_data_item);
            }
        }
        scheduling_overview_data

    }
}