use std::env;
use std::path::Path;

use crate::data_processing::sources;
use crate::models::SchedulingEnvironment;

pub fn create_scheduling_environment(number_of_periods: u32) -> SchedulingEnvironment {
    let mut scheduling_environment =
        initialize_scheduling_environment(number_of_periods).expect("No data file was provided.");
    scheduling_environment.initialize_work_orders();
    scheduling_environment
}

fn initialize_scheduling_environment(number_of_periods: u32) -> Option<SchedulingEnvironment> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let file_path = Path::new(&args[1]);
        let scheduling_environment = sources::excel::load_data_file(file_path, number_of_periods)
            .expect("Could not load data file.");
        println!("{}", scheduling_environment);
        return Some(scheduling_environment);
    }
    None
}
