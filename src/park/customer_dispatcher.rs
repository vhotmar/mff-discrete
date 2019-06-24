use crate::config;
use crate::config::{CustomerConfig, Id};
use crate::park;
use crate::park::customer::{CarouselInfo, Customer};
use std::cmp::Ordering;
use std::collections::binary_heap::BinaryHeap;
use std::collections::HashMap;
use crate::discrete_system::address::Address;
use crate::discrete_system::effector::Effector;
use crate::discrete_system::Time;
use crate::discrete_system::component::{StartInfo, HandleInfo};
use crate::park::ParkComponent;
use serde::{Deserialize, Serialize};

impl PartialEq for CustomerConfig {
    fn eq(&self, other: &CustomerConfig) -> bool {
        self.arrival_time == other.arrival_time
    }
}

impl Eq for CustomerConfig {}

impl PartialOrd for CustomerConfig {
    fn partial_cmp(&self, other: &CustomerConfig) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CustomerConfig {
    fn cmp(&self, other: &Self) -> Ordering {
        other.arrival_time.cmp(&self.arrival_time) // from low to high
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerDispatcher {
    carousels: HashMap<Id, Address>,
    customers_configs: BinaryHeap<config::CustomerConfig>,
}

/// Only goal for CustomerDispatcher is to take all customers from config file and then add them to
/// the simulation when needed
impl CustomerDispatcher {
    pub fn new(
        carousels: HashMap<Id, Address>,
        customers_configs: Vec<config::CustomerConfig>,
    ) -> CustomerDispatcher {
        CustomerDispatcher {
            carousels,
            customers_configs: BinaryHeap::from(customers_configs),
        }
    }

    fn schedule_next(&mut self, effector: &mut Effector<park::Event, park::Component>, current_time: Time) {
        if let Some(config) = self.customers_configs.peek() {
            effector.schedule_in_to_self(
                config.arrival_time - current_time,
                park::Event::CustomerDispatcherEvent(Event::Tick),
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    Tick,
}

impl ParkComponent for CustomerDispatcher {
    fn start(&mut self, _info: StartInfo) -> Effector<park::Event, park::Component> {
        let mut effector = Effector::new();

        self.schedule_next(&mut effector, 0);

        effector
    }

    fn handle(&mut self, info: HandleInfo, message: park::Event) -> Effector<park::Event, park::Component> {
        let mut effector = Effector::new();

        let message: Option<Event> = message.into();

        match message {
            Some(Event::Tick) => {
                while self.customers_configs.peek().is_some()
                    && self.customers_configs.peek().unwrap().arrival_time == info.current_time
                {
                    let config = self.customers_configs.pop().unwrap();

                    let customer = Customer::new(
                        config
                            .carousels
                            .iter()
                            .map(|id| CarouselInfo {
                                address: self.carousels[id].clone(),
                                id: *id,
                            })
                            .collect(),
                        config
                    );

                    effector.instantiate_new_component(park::Component::Customer(customer));
                }

                self.schedule_next(&mut effector, info.current_time);
            }
            _ => {}
        }

        effector
    }
}
