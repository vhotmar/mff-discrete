use crate::park;
use std::collections::vec_deque::VecDeque;
use crate::config::{Id, CustomerConfig};
use crate::discrete_system::address::Address;
use crate::discrete_system::effector::Effector;
use crate::discrete_system::component::{StartInfo, HandleInfo};
use crate::park::ParkComponent;
use serde::{Deserialize, Serialize};
use crate::discrete_system::Time;

/// 1. `Customer` when
///     * `WaitingOnCarousel`
///         * Should accept event `RideStarted`
///             1) transition to `OnCarousel`
///     * `OnCarousel`
///         * Should accept event `RideEnded`
///             1) pop carousels queue -> send event to carousel `PersonArrived`
///             2) transition to `WaitingOnCarousel`
///             3) if no carousel transition to `Idle`

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum State {
    WaitingOnCarousel(Id),
    OnCarousel(Id),
    Idle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    RideStarted,
    RideEnded,
}

impl Into<park::Event> for Event {
    fn into(self) -> park::Event {
        park::Event::CustomerEvent(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CarouselInfo {
    pub id: Id,
    pub address: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Customer {
    state: State,
    pub config: CustomerConfig,
    carousels: VecDeque<CarouselInfo>,
    started_waiting_on: Time,
    number_of_rides: u32,
    total_waiting_time: u32,
    total_time: u32,
}

impl Customer {
    pub fn new(carousels: VecDeque<CarouselInfo>, config: CustomerConfig) -> Customer {
        Customer {
            state: State::Idle,
            carousels,
            config,
            started_waiting_on: 0,
            number_of_rides: 0,
            total_waiting_time: 0,
            total_time: 0
        }
    }

    fn next_run(&mut self, effector: &mut Effector<park::Event, park::Component>, time: Time) {
        self.started_waiting_on = time;
        self.total_time = time - self.config.arrival_time;

        if let Some(carousel) = self.carousels.pop_front() {
            effector.schedule_immediately(
                carousel.address,
                park::carousel::Event::CustomerArrived.into(),
            );

            self.state = State::WaitingOnCarousel(carousel.id);
        } else {
            self.state = State::Idle;
        }
    }
}

impl ParkComponent for Customer {
    fn start(&mut self, info: StartInfo) -> Effector<park::Event, park::Component> {
        let mut effector: Effector<park::Event, park::Component> = Effector::new();

        self.next_run(&mut effector, info.current_time);

        effector
    }

    fn handle(&mut self, info: HandleInfo, message: park::Event) -> Effector<park::Event, park::Component> {
        let mut effector = Effector::new();

        let message: Option<Event> = message.into();

        match self.state {
            State::OnCarousel(_) => match message {
                Some(Event::RideEnded) => { self.next_run(&mut effector, info.current_time); },
                _ => {}
            },
            State::WaitingOnCarousel(id) => match message {
                Some(Event::RideStarted) => {
                    self.state = State::OnCarousel(id);
                    self.total_waiting_time += info.current_time - self.started_waiting_on - 1;
                    self.number_of_rides += 1;
                },
                _ => {}
            },
            _ => {}
        }

        effector
    }
}
