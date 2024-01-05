use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::Mutex;

use crate::agents::scheduler_agent::scheduler_algorithm::OptimizedWorkOrder;
use crate::agents::scheduler_agent::scheduler_algorithm::OptimizedWorkOrders;
use crate::agents::scheduler_agent::scheduler_algorithm::PriorityQueues;
use crate::agents::scheduler_agent::scheduler_algorithm::SchedulerAgentAlgorithm;
use crate::agents::scheduler_agent::SchedulerAgent;
use crate::models::SchedulingEnvironment;
use crate::models::WorkOrders;

// We should not clone in the work orders here. They should reference the same thing to work
// properly.
pub fn build_scheduler_agent(
    scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
) -> Addr<SchedulerAgent> {
    let cloned_work_orders = scheduling_environment.lock().unwrap().clone_work_orders();

    let optimized_work_orders: OptimizedWorkOrders =
        create_optimized_work_orders(&cloned_work_orders);

    // The periods should really not be where it is here. It should be in the SchedulingEnvironment.
    // What is the problem? The problem is that we would either need to clone the periods or
    // increment the reference count. We should increment the reference count, but this leads to the
    // problem of having to points of access to the scheduling environment for the SchedulerAgent.
    // This is not good, especially as the SchedulerAgentAlgorithm will be using a much more
    // efficient data structure in the future. This means that we should actually use the scheduling
    // environments periods and for now we can simply clone them and then later on we can
    // change the data structure to something more efficient. Yes the key insight here is that we
    // do not need the algorithm to have direct access to the SchedulingEnvironment only that the
    // SchedulingAgent will be able to update the SchedulerAgentAlgorithm when the
    // SchedulingEnvironment changes. This is a much better design.
    let scheduler_agent_algorithm = SchedulerAgentAlgorithm::new(
        0.0,
        HashMap::new(),
        HashMap::new(),
        cloned_work_orders,
        PriorityQueues::new(),
        optimized_work_orders,
        scheduling_environment.lock().unwrap().clone_periods(),
        true,
    );

    let scheduler_agent = SchedulerAgent::new(
        String::from("Dan F"),
        scheduling_environment,
        scheduler_agent_algorithm,
        None,
        None,
    );
    scheduler_agent.start()
}

fn create_optimized_work_orders(work_orders: &WorkOrders) -> OptimizedWorkOrders {
    let mut optimized_work_orders: HashMap<u32, OptimizedWorkOrder> = HashMap::new();

    for (work_order_number, work_order) in &work_orders.inner {
        if work_order.unloading_point.present {
            let period = work_order.unloading_point.period.clone();
            optimized_work_orders.insert(
                *work_order_number,
                OptimizedWorkOrder::new(period.clone(), period, HashSet::new()),
            );
        }
    }
    OptimizedWorkOrders::new(optimized_work_orders)
}
