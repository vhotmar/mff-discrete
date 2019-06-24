#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

#[macro_use]
extern crate failure;

use failure::{Error, Fail};
use crate::discrete_system::DiscreteSystem;
use std::collections::{HashSet, HashMap};
use crate::park::carousel::Carousel;
use crate::config::{Id, SystemConfig};
use crate::discrete_system::address::Address;
use crate::park::customer_dispatcher::CustomerDispatcher;
use serde::{Serialize};
use rocket_contrib::json::Json;
use std::fs::File;
use std::env;

mod config;
mod discrete_system;
mod park;

#[derive(Debug, Fail)]
#[fail(display = "validation failed because of \"{}\"", error)]
struct ValidationError {
    error: String,
}

fn validate_config(config: &config::SystemConfig) -> Result<(), Error> {
    let mut s = HashSet::new();

    for carousel in config.carousels.iter() {
        if s.contains(&carousel.id) {
            return Err(ValidationError {
                error: format!("There is carousel id \"{}\" collision", carousel.id),
            }
                .into());
        }

        s.insert(carousel.id);

        if carousel.run_time <= 0 || carousel.extend_time <= 0 || carousel.wait_time <= 0 {
            return Err(ValidationError {
                error: format!("There is carousel \"{}\" with invalid times", carousel.id),
            }
                .into());
        }

        if carousel.capacity <= 0 {
            return Err(ValidationError {
                error: format!("There is carousel \"{}\" with invalid capacity", carousel.id),
            }.into())
        }

        if carousel.min_capacity <= 0 || carousel.min_capacity > carousel.capacity {
            return Err(ValidationError {
                error: format!("There is carousel \"{}\" with invalid minimal capacity", carousel.id),
            }.into())
        }
    }

    for customer in config.customers.iter() {
        for id in customer.carousels.iter() {
            if !s.contains(&id) {
                return Err(ValidationError { error: format!("There does not exist carousel with id \"{}\" requested by user with id \"{}\"", id, customer.id) }.into());
            }
        }
    }

    return Ok(());
}

fn bootstrap_system(config: SystemConfig) -> Result<DiscreteSystem<park::Event, park::Component>, Error> {
    validate_config(&config)?;

    let mut system: DiscreteSystem<park::Event, park::Component> = DiscreteSystem::new();

    let carousels_map = config
        .carousels
        .iter()
        .map(|carousel| {
            (
                carousel.id,
                system.register_component(Carousel::new(carousel.clone()).into()),
            )
        })
        .collect::<HashMap<Id, Address>>();

    system.register_component(CustomerDispatcher::new(carousels_map, config.customers).into());

    system.start();

    Ok(system)
}

#[derive(Serialize)]
struct TickResponse {
    events: Vec<discrete_system::Event<park::Event>>,
    system: DiscreteSystem<park::Event, park::Component>,
}

#[post("/bootstrap", format = "application/json", data = "<config>")]
fn server_bootstrap_system(config: Json<SystemConfig>) -> Json<DiscreteSystem<park::Event, park::Component>> {
    let system = bootstrap_system(config.into_inner()).unwrap();

    Json(system)
}

#[post("/tick", format = "application/json", data = "<system>")]
fn server_tick(mut system: Json<DiscreteSystem<park::Event, park::Component>>) -> Json<TickResponse> {
    let events = system.tick();

    let resp = TickResponse {
        events,
        system: system.into_inner(),
    };

    Json(resp)
}

fn run_server() -> Result<(), Error> {
    let cors = rocket_cors::CorsOptions::default().to_cors()?;

    rocket::ignite().attach(cors).mount("/", routes![server_bootstrap_system, server_tick]).launch();

    Ok(())
}

fn get_config(path: String) -> Result<config::SystemConfig, Error> {
    let file = File::open(&path)?;

    let config = serde_json::from_reader(file)?;

    Ok(config)
}

fn run_local() -> Result<(), Error> {
    let config = get_config(format!("{}/config.json", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or(config::SystemConfig::default());

    let mut system = bootstrap_system(config).unwrap();

    while system.has_events() {
        let events = system.tick();

        for event in events {
            print!("In {} - ", system.current_time);

            let s = system.components.get(&event.from_address).unwrap();

            match s {
                park::Component::Carousel(carousel) => print!("Carousel({})", carousel.config.id),
                park::Component::Customer(customer) => print!("Customer({})", customer.config.id),
                park::Component::CustomerDispatcher(_) => print!("Customer Dispatcher"),
            }

            print!(" sending to ");

            let s = system.components.get(&event.to_address).unwrap();

            match s {
                park::Component::Carousel(carousel) => print!("Carousel({})", carousel.config.id),
                park::Component::Customer(customer) => print!("Customer({})", customer.config.id),
                park::Component::CustomerDispatcher(_) => print!("Customer Dispatcher"),
            }

            print!(" - ");

            match event.message {
                park::Event::CarouselEvent(event) => match event {
                    park::carousel::Event::CustomerArrived => print!("Customer arrived"),
                    park::carousel::Event::EndRide => print!("Ride ended"),
                    park::carousel::Event::ExtendedWaitEnded(_) => print!("Extended wait ended"),
                    park::carousel::Event::StandardWaitEnded(_) => print!("Standard wait ended"),
                    park::carousel::Event::Start => print!("Ride starting"),
                },
                park::Event::CustomerDispatcherEvent(event) => match event {
                    park::customer_dispatcher::Event::Tick => print!("Tick"),
                }
                park::Event::CustomerEvent(event) => match event {
                    park::customer::Event::RideEnded => print!("Ride started"),
                    park::customer::Event::RideStarted => print!("Ride started"),
                }
            }

            println!();
        }
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 && args[1] == "-console" {
        run_local();
    } else {
        run_server();
    }
}
