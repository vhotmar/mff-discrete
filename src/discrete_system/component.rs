use crate::discrete_system::{DiscreteSystemMessage, Time};
use crate::discrete_system::effector::Effector;
use crate::discrete_system::address::Address;

pub struct StartInfo {
    pub self_address: Address,
    pub current_time: Time,
}

pub struct HandleInfo {
    pub self_address: Address,
    pub sender_address: Address,
    pub current_time: Time,
}

/// `Component` represents an `Actor` from `ActorModel`
/// It needs to be able to handle messages and send effect the system on
/// start of this component

pub trait Component<M: DiscreteSystemMessage>: Sized {
    fn start(&mut self, info: StartInfo) -> Effector<M, Self>;
    fn handle(&mut self, info: HandleInfo, message: M) -> Effector<M, Self>;
}