use crate::{config, park};
use std::cmp::{min, max};
use std::collections::vec_deque::VecDeque;
use std::mem;
use crate::discrete_system::Time;
use crate::discrete_system::address::Address;
use crate::discrete_system::effector::Effector;
use crate::discrete_system::component::{StartInfo, HandleInfo};
use crate::park::ParkComponent;
use serde::{Deserialize, Serialize};

/// 1. Carousel when
///     * `Idle(next_state)`
///         * Should accept event `CustomerArrived`
///             * If `next_state` is `StandardWaiting`
///                 1) Transition to `StandardWaiting`
///                 2) Schedule event `StandardWaitEnded` in `wait_time`
///             * If `next_state` is `ExtendedWaiting`
///                 1) Transition to `ExtendedWaiting`
///                 2) Schedule event `ExtendedWaitEnded` in `wait_time`
///             * Panic otherwise
///     * `StandardWaiting`
///         * Should accept event `StandardWaitEnded` with correct cycle
///             * If enough people (`inner_queue.len() >= min_capacity`):
///                 1) Transition to `Starting`
///                 2) Schedule event `Start` in `1` to itself
///             * If no people
///                 1) transition to `Idle(ExtendedWaiting)`
///             * If not enough people
///                 1) transition to `ExtendedWaiting`
///                 2) schedule event `ExtendedWaitEnded`
///     * `ExtendedWaiting`
///         * Should accept event `CustomerArrived`
///             * If enough people (waiting people >= min_capacity):
///                 1) Transition to `Starting`
///                 2) Schedule event `Start` in `1` to itself
///         * Should accept event `ExtendedWaitEnded` with correct cycle
///             1) Transition to `Starting`
///             2) Schedule event `Start` in `1` to itself
///     * `Starting(time)`
///         * Should accept event `Start`
///             1) Send people in `inner_queue` event `RideStarted`
///             2) Move all people from `inner_queue` to `on_carousel`
///             3) Move all people possible from `outer_queue` to `inner_queue`
///             3) Transition to `Running`
///             4) Schedule event `End` to itself in `run_time` seconds
///     * `Running`
///         * Should accept event `End`
///             1) Send `RideEnded` to all customers `on_carousel`
///             2) Transition to `StandardWaiting`
///             3) Schedule event `StandardWaitEnded` in `wait_time - 1` (1 unit of time spent in starting)
///             4) Empty `on_carousel`
///     * Every time
///         * Should accept event `CustomerArrived`
///             * If `Starting(time)` and `time != current_time` (when we are starting we still receive customers)
///                 * Put customer in `outer_queue`
///             * Else
///                 * Put customer in `inner_queue` if possible `inner_queue.len() < capacity`
///                 * Else put customer in `outer_queue`

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum State {
    Idle(Box<State>),
    StandardWaiting,
    ExtendedWaiting,
    Starting(Time),
    Running,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    CustomerArrived,
    StandardWaitEnded(u32),
    ExtendedWaitEnded(u32),
    EndRide,
    Start,
}

impl Into<park::Event> for Event {
    fn into(self) -> park::Event {
        park::Event::CarouselEvent(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CustomerInfo {
    arrival_time: Time,
    address: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Carousel {
    pub config: config::CarouselConfig,
    state: State,
    customers_inner_queue: Vec<CustomerInfo>,
    customers_outer_queue: VecDeque<CustomerInfo>,
    customers_on_ride: Vec<CustomerInfo>,
    cycle: u32,
    rides: u32,
    avg_customers_on_ride: f64,
    max_customers_queue_len: u32,
    idle_time: u32,
    idle_started: Time
}

impl Carousel {
    pub fn new(config: config::CarouselConfig) -> Carousel {
        Carousel {
            config,
            state: State::Idle(Box::new(State::StandardWaiting)),
            cycle: 0,
            customers_inner_queue: Vec::new(),
            customers_outer_queue: VecDeque::new(),
            customers_on_ride: Vec::new(),
            rides: 0,
            avg_customers_on_ride: 0.0,
            max_customers_queue_len: 0,
            idle_time: 0,
            idle_started: 0
        }
    }

    fn start_ride(&mut self, time: Time, effector: &mut Effector<park::Event, park::Component>) {
        self.state = State::Starting(time);
        self.cycle += 1;

        effector.schedule_in_to_self(1, Event::Start.into());
    }

    fn do_ride(&mut self, effector: &mut Effector<park::Event, park::Component>) {
        self.state = State::Running;

        self.customers_on_ride = mem::replace(&mut self.customers_inner_queue, Vec::new());
        self.customers_on_ride.iter().for_each(|customer| {
            effector.schedule_immediately(
                customer.address.clone(),
                park::customer::Event::RideStarted.into(),
            )
        });

        let customers_to_move = min(
            self.config.capacity,
            self.customers_outer_queue.len() as u32,
        );

        for _ in 0..customers_to_move {
            self.customers_inner_queue
                .push(self.customers_outer_queue.pop_front().unwrap());
        }

        effector.schedule_in_to_self(self.config.run_time - 1, Event::EndRide.into());
    }

    fn end_ride(&mut self, effector: &mut Effector<park::Event, park::Component>) {
        self.avg_customers_on_ride = ((self.rides as f64) * (self.avg_customers_on_ride) + (self.customers_on_ride.len() as f64)) / ((self.rides + 1) as f64);
        self.rides += 1;

        self.customers_on_ride.drain(..).for_each(|info| {
            effector.schedule_immediately(info.address, park::customer::Event::RideEnded.into())
        });

        self.start_standard_wait(effector);
    }

    fn start_standard_wait(&mut self, effector: &mut Effector<park::Event, park::Component>) {
        self.state = State::StandardWaiting;

        effector.schedule_in_to_self(
            self.config.wait_time,
            Event::StandardWaitEnded(self.cycle).into(),
        )
    }

    fn start_extended_wait(&mut self, effector: &mut Effector<park::Event, park::Component>) {
        self.state = State::ExtendedWaiting;

        effector.schedule_in_to_self(
            self.config.extend_time,
            Event::ExtendedWaitEnded(self.cycle).into(),
        )
    }
}

impl ParkComponent for Carousel {
    fn start(&mut self, _info: StartInfo) -> Effector<park::Event, park::Component> {
        Effector::new()
    }

    fn handle(&mut self, info: HandleInfo, message: park::Event) -> Effector<park::Event, park::Component> {
        let mut effector = Effector::new();

        let message: Option<Event> = message.into();

        self.max_customers_queue_len = max((self.customers_inner_queue.len() + self.customers_outer_queue.len()) as u32, self.max_customers_queue_len);

        if let Some(Event::CustomerArrived) = message {
            let customer_info = CustomerInfo {
                address: info.sender_address,
                arrival_time: info.current_time,
            };

            match self.state {
                State::Starting(time) if info.current_time != time => {
                    self.customers_outer_queue.push_back(customer_info);
                }
                _ => {
                    if self.customers_inner_queue.len() < self.config.capacity as usize {
                        self.customers_inner_queue.push(customer_info);
                    } else {
                        self.customers_outer_queue.push_back(customer_info);
                    }
                }
            }
        }

        match &self.state {
            State::Idle(next_state) => {
                self.idle_time += info.current_time - self.idle_started;

                match message {
                    Some(Event::CustomerArrived) => match **next_state {
                        State::StandardWaiting => {
                            self.start_standard_wait(&mut effector);
                        }
                        State::ExtendedWaiting => {
                            self.start_extended_wait(&mut effector);
                        }
                        _ => {
                            panic!("Idle has invalid next_state");
                        }
                    },
                    _ => {}
                }
            },
            State::StandardWaiting => match message {
                Some(Event::StandardWaitEnded(cycle)) if self.cycle == cycle => {
                    if self.customers_inner_queue.len() >= self.config.min_capacity as usize {
                        self.start_ride(info.current_time, &mut effector);
                    } else if self.customers_inner_queue.len() == 0 {
                        self.idle_started = info.current_time;
                        self.state = State::Idle(Box::new(State::ExtendedWaiting));
                    } else {
                        self.start_extended_wait(&mut effector);
                    }
                }
                _ => {}
            },
            State::ExtendedWaiting => match message {
                Some(Event::CustomerArrived) => {
                    if self.customers_inner_queue.len() >= self.config.min_capacity as usize {
                        self.start_ride(info.current_time, &mut effector);
                    }
                }
                Some(Event::ExtendedWaitEnded(cycle)) if self.cycle == cycle => {
                    self.start_ride(info.current_time, &mut effector)
                }
                _ => {}
            },
            State::Running => match message {
                Some(Event::EndRide) => self.end_ride(&mut effector),
                _ => {}
            },
            State::Starting(_) => match message {
                Some(Event::Start) => self.do_ride(&mut effector),
                _ => {}
            },
        }

        effector
    }
}
