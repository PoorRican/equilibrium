equilibrium is a framework for creating distributed control systems.

It provides a several types of control system paradigms and is agnostic to the
underlying hardware. The intention is to provide a framework that can be used in
a variety of applications such as aquaponics/hydroponics, aquariums, homebrewing,
bioreactors and more.


# Example
This example creates 2 controllers:
- a grow-light turns on an output at 5:00AM and turns it off after 8 hours,
- a heater that turns on an output when the temperature is below 70 degrees.

Then a runtime is created which polls the controllers every second. The output of the
controllers are messages which are sent to a message broker operating on localhost.

```rust
use chrono::{Duration, NaiveTime, Utc};
use equilibrium::controllers::{Controller, TimedOutput, Threshold};
use equilibrium::{Output, Input, ControllerGroup};

#[tokio::main]
fn main() {
    // this represents a grow-light
    let time = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
    let duration = Duration::hours(8);
    let mut grow_light = TimedOutput::new(
        Output::new(|_| {
            // low-level code would go here
        }),
        time,
        duration,
    );

    // this represents a heater controller
    let min_temp = 70.0;        // activate heater when temp is below 70 degrees
    let interval = Duration::minutes(5);    // check temp every 5 minutes

    let heater = Output::new(|_| {
        // low-level code would go here
    });
    let temp_sensor = Input::new(|| {
        // low-level code would go here
        String::from("79.0")
    });
    let mut heater_controller = Threshold::new(
        min_temp,
        temp_sensor,
        heater,
        interval
    );

    // a group can be used to manage multiple controllers
    let mut group = ControllerGroup::new()
        .add_controller(grow_light)
        .add_controller(heater_controller);

    // create a runtime which polls every second and build an emitter
    let runtime = Runtime::new(
        group,
        chrono::Duration::seconds(1)
    ).build_emitter("http://localhost:8000");
}
```

# Features

## Controller Types
- `TimedOutput`: turns on an output at a specific time and turns it off after a duration
- `Threshold`: turns on an output when a threshold is met
- `BidirectionalThreshold`: increases or decreases an output when a threshold is met

More controller types (i.e.: PID) will be added in the future.

## Outputs

Currently, only binary output devices are supported.

# Roadmap
- [ ] Add support for a more common message broker such as MQTT
- [ ] Add support for more types of controllers (i.e.: PID, etc.)
- [ ] Create examples for GPIO (i.e.: Atmel, RPi, ARM, RISC-V, etc.)
- [ ] Add support for more types of inputs (i.e.: analog, digital, etc.)
- [ ] Add support for more types of outputs (i.e.: PWM, digital, etc.)
- [ ] Simplify API by introducing macros