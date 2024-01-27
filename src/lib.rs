//! equilibrium is a framework for creating automated and controlled systems.
//!
//! It provides a several types of control system paradigms and is agnostic to the
//! underlying hardware. The intention is to provide a framework that can be used in
//! a variety of applications such as aquaponics/hydroponics, aquariums, homebrewing,
//! bioreactors and more.
//!
//! This project is still in the early stages of development and is not yet ready for
//! production use. See the "Roadmap" section below for more information.
//!
//! # Example
//! This example creates 2 controllers:
//! - a grow-light turns on an output at 5:00AM and turns it off after 8 hours,
//! - a heater that turns on an output when the temperature is below 70 degrees.
//!
//! ```
//! use chrono::{Duration, NaiveTime, Utc};
//! use equilibrium::controllers::{Controller, TimedOutput, Threshold};
//! use equilibrium::{Output, Input, ControllerGroup};
//!
//! // this represents a grow-light
//! let time = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
//! let duration = Duration::hours(8);
//! let mut grow_light = TimedOutput::new(
//!     Output::new(|_| {
//!        // low-level code would go here
//!     }),
//!     time,
//!     duration,
//! );
//!
//! // this represents a heater controller
//! let min_temp = 70.0;        // activate heater when temp is below 70 degrees
//! let interval = Duration::minutes(5);    // check temp every 5 minutes
//!
//! let heater = Output::new(|_| {
//!     // low-level code would go here
//! });
//! let temp_sensor = Input::new(|| {
//!     // low-level code would go here
//!     String::from("79.0")
//! });
//! let mut heater_controller = Threshold::new(
//!     min_temp,
//!     temp_sensor,
//!     heater,
//!     interval
//! );
//!
//! // a group can be used to manage multiple controllers
//! let mut group = ControllerGroup::new()
//!     .add_controller(grow_light)
//!     .add_controller(heater_controller);
//!
//! let now = Utc::now();
//! let messages = group.poll(now);
//! ```
//!
//! # Roadmap
//! * Create a group that can manage multiple controllers
//! * Send messages to a message broker
pub mod types;
mod scheduler;
mod input;
mod output;
pub mod controllers;
mod group;

// re-export types
pub use input::Input;
pub use output::Output;

pub use group::ControllerGroup;