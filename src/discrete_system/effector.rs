use crate::discrete_system::component::Component;
use crate::discrete_system::{DiscreteSystemMessage, Time};
use crate::discrete_system::address::Address;

pub enum ScheduledEventAddress {
    SelfAddress,
    RemoteAddress(Address),
}

pub struct ScheduledEvent<M> {
    pub message: M,
    pub in_time: Time,
    pub address: ScheduledEventAddress,
}

pub struct Effector<M: DiscreteSystemMessage, C: Component<M>> {
    pub events: Vec<ScheduledEvent<M>>,
    pub components: Vec<C>,
}

impl<M: DiscreteSystemMessage, C: Component<M>> Effector<M, C> {
    pub fn new() -> Effector<M, C> {
        Effector {
            events: Vec::new(),
            components: Vec::new(),
        }
    }

    pub fn schedule_in(&mut self, address: Address, in_time: Time, message: M) {
        self.events.push(ScheduledEvent {
            in_time,
            message,
            address: ScheduledEventAddress::RemoteAddress(address),
        })
    }

    pub fn schedule_immediately(&mut self, address: Address, message: M) {
        self.events.push(ScheduledEvent {
            in_time: 0,
            message,
            address: ScheduledEventAddress::RemoteAddress(address),
        })
    }

    pub fn schedule_in_to_self(&mut self, in_time: Time, message: M) {
        self.events.push(ScheduledEvent {
            in_time,
            message,
            address: ScheduledEventAddress::SelfAddress,
        })
    }

    pub fn schedule_to_self_immediately(&mut self, message: M) {
        self.events.push(ScheduledEvent {
            in_time: 0,
            message,
            address: ScheduledEventAddress::SelfAddress,
        })
    }

    pub fn instantiate_new_component(&mut self, data: C) {
        self.components.push(data);
    }
}