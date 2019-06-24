use crate::discrete_system::component::{Component as SystemComponent, HandleInfo, StartInfo};
use crate::discrete_system::effector::Effector;
use serde::{Deserialize, Serialize};

pub mod carousel;
pub mod customer;
pub mod customer_dispatcher;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    CustomerDispatcherEvent(customer_dispatcher::Event),
    CustomerEvent(customer::Event),
    CarouselEvent(carousel::Event),
}

impl Into<Option<customer_dispatcher::Event>> for Event {
    fn into(self) -> Option<customer_dispatcher::Event> {
        match self {
            Event::CustomerDispatcherEvent(event) => Some(event),
            _ => None,
        }
    }
}

impl Into<Option<customer::Event>> for Event {
    fn into(self) -> Option<customer::Event> {
        match self {
            Event::CustomerEvent(event) => Some(event),
            _ => None,
        }
    }
}

impl Into<Option<carousel::Event>> for Event {
    fn into(self) -> Option<carousel::Event> {
        match self {
            Event::CarouselEvent(event) => Some(event),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Component {
    CustomerDispatcher(customer_dispatcher::CustomerDispatcher),
    Customer(customer::Customer),
    Carousel(carousel::Carousel),
}

impl Into<Component> for customer_dispatcher::CustomerDispatcher {
    fn into(self) -> Component {
        Component::CustomerDispatcher(self)
    }
}

impl Into<Component> for customer::Customer {
    fn into(self) -> Component {
        Component::Customer(self)
    }
}

impl Into<Component> for carousel::Carousel {
    fn into(self) -> Component {
        Component::Carousel(self)
    }
}

trait ParkComponent {
    fn start(&mut self, info: StartInfo) -> Effector<Event, Component>;
    fn handle(&mut self, info: HandleInfo, message: Event) -> Effector<Event, Component>;
}

impl SystemComponent<Event> for Component {
    fn start(&mut self, info: StartInfo) -> Effector<Event, Component> {
        match self {
            Component::Carousel(carousel) => carousel.start(info),
            Component::Customer(customer) => customer.start(info),
            Component::CustomerDispatcher(customer_dispatcher) => customer_dispatcher.start(info),
        }
    }

    fn handle(&mut self, info: HandleInfo, message: Event) -> Effector<Event, Component> {
        match self {
            Component::Carousel(carousel) => carousel.handle(info, message),
            Component::Customer(customer) => customer.handle(info, message),
            Component::CustomerDispatcher(customer_dispatcher) => customer_dispatcher.handle(info, message),
        }
    }
}
