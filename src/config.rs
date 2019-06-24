use serde::{Deserialize, Serialize};

pub type Id = u32;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CarouselConfig {
    pub id: Id,
    pub min_capacity: u32, // Minimum number of people for carousel to run
    pub capacity: u32,     // Maximum number of people at the same time on carousel
    pub run_time: u32,     // How long is one run
    pub wait_time: u32,    // How long is carousel waiting before next run
    pub extend_time: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerConfig {
    pub id: Id,
    pub arrival_time: u32,
    pub carousels: Vec<Id>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SystemConfig {
    pub carousels: Vec<CarouselConfig>,
    pub customers: Vec<CustomerConfig>,
}
