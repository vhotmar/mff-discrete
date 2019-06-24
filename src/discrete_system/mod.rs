use crate::discrete_system::component::{Component, StartInfo, HandleInfo};
use std::collections::{HashMap, BinaryHeap};
use crate::discrete_system::address::{Address, AddressGenerator};
use std::cmp::Ordering;
use crate::discrete_system::effector::{Effector, ScheduledEventAddress};
use serde::{Deserialize, Serialize};

pub mod address;
pub mod component;
pub mod effector;

pub type Time = u32;

pub trait DiscreteSystemMessage: Clone {}
impl<T: Clone> DiscreteSystemMessage for T {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<M: DiscreteSystemMessage> {
    time: Time,
    pub to_address: Address,
    pub from_address: Address,
    pub message: M,
}

impl<M: DiscreteSystemMessage> PartialEq for Event<M> {
    fn eq(&self, other: &Event<M>) -> bool {
        self.time == other.time
    }
}

impl<M: DiscreteSystemMessage> Eq for Event<M> {}

impl<M: DiscreteSystemMessage> PartialOrd for Event<M> {
    fn partial_cmp(&self, other: &Event<M>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<M: DiscreteSystemMessage> Ord for Event<M> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.cmp(&self.time)
    }
}

#[derive(Serialize, Deserialize)]
pub struct DiscreteSystem<M: DiscreteSystemMessage, C: Component<M>> {
    pub current_time: u32,
    pub components: HashMap<Address, C>,
    events: BinaryHeap<Event<M>>,
    address_generator: AddressGenerator,
}

/// `DiscreteSystem` manages discrete system, which composes of components
/// and information which the components are sending between themselves

impl<M: DiscreteSystemMessage, C: Component<M>> DiscreteSystem<M, C> {
    pub fn new() -> DiscreteSystem<M, C> {
        DiscreteSystem {
            current_time: 0,
            components: HashMap::new(),
            events: BinaryHeap::new(),
            address_generator: AddressGenerator::new(),
        }
    }

    pub fn register_component(&mut self, c: C) -> Address {
        let addr = self.address_generator.next();

        self.components.insert(addr.clone(), c);

        addr
    }

    fn start_component(&mut self, address: Address) {
        let effector = self.components.get_mut(&address).unwrap().start(StartInfo {
            self_address: address.clone(),
            current_time: self.current_time,
        });

        self.apply_effector(address.clone(), effector);
    }

    fn apply_effector(&mut self, from_address: Address, effector: Effector<M, C>) {
        for event in effector.events.into_iter() {
            let to_address = match event.address {
                ScheduledEventAddress::SelfAddress => from_address.clone(),
                ScheduledEventAddress::RemoteAddress(remote) => remote,
            };

            self.events.push(Event {
                from_address: from_address.clone(),
                to_address,
                message: event.message,
                time: self.current_time + event.in_time,
            });
        }

        for component in effector.components.into_iter() {
            let addr = self.register_component(component);

            self.start_component(addr.clone());
        }
    }

    pub fn tick(&mut self) -> Vec<Event<M>> {
        let mut events = Vec::new();

        if self.events.is_empty() {
            return events;
        }

        self.current_time = self.events.peek().unwrap().time;

        while self.events.peek().is_some() && self.events.peek().unwrap().time == self.current_time
            {
                let event = self.events.pop().unwrap();

                events.push(event.clone());

                let effector = self.components.get_mut(&event.to_address).unwrap().handle(
                    HandleInfo {
                        self_address: event.to_address.clone(),
                        sender_address: event.from_address.clone(),
                        current_time: self.current_time,
                    },
                    event.message.clone(),
                );

                self.apply_effector(event.to_address.clone(), effector);
            }

        events
    }

    pub fn start(&mut self) {
        let addresses: Vec<_> = self.components.keys().cloned().collect();

        addresses
            .into_iter()
            .for_each(|address| self.start_component(address));

        if self.events.peek().is_some() && self.events.peek().unwrap().time == 0 {
            self.tick();
        }
    }

    pub fn run(&mut self) {
        self.start();

        while !self.events.is_empty() {
            self.tick();
        }
    }

    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }
}
