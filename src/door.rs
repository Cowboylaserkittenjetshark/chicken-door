use thiserror::Error;
use crate::settings::Settings;
use std::sync::{LazyLock, Mutex, MutexGuard};

static DOOR_STATE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State::Closed));

const LIMIT_PIN: u8 = 24;
const MOTOR_FLIP_FLOP_PIN: u8 = 5;
const MOTOR_ENABLE_PIN: u8 = 6;
const DOOR_CLOSE_SECS: u64 = 5;
const MFF_SAFETY_MSECS: u64 = 250;
const OPEN_TIMEOUT_SECS: u64 = 6;

pub fn close() {
    use rppal::gpio::{Gpio, Trigger};
    use std::time::Duration;
    use std::thread;
    match DOOR_STATE.lock() {
        Ok(mut guard) => match *guard {
            State::Open => {
                let gpio = Gpio::new().expect("failed to open gpio interface");
                let mut mff_pin = gpio.get(MOTOR_FLIP_FLOP_PIN).expect("failed to get motor flip flop pin").into_output();
                let mut me_pin = gpio.get(MOTOR_ENABLE_PIN).expect("failed to get motor enable pin").into_output();
                mff_pin.set_reset_on_drop(false);
                me_pin.set_reset_on_drop(false);
    
                me_pin.set_low();
                println!("Sleeping for {MFF_SAFETY_MSECS} milliseconds");
                thread::sleep(Duration::from_millis(MFF_SAFETY_MSECS));
                mff_pin.set_high();
                me_pin.set_high();
                println!("Sleeping for {DOOR_CLOSE_SECS} seconds");
                thread::sleep(Duration::from_secs(DOOR_CLOSE_SECS));
                mff_pin.set_low();
                me_pin.set_low();
                *guard = State::Closed;
                println!("Finished close routine");
            },
            State::Closed => println!("Door already closed"),
            _ => println!("Door in flight"),
        },
        Err(_) => println!("Could not aquire state lock, not closing"),
    }
}

pub fn open() {
    use rppal::gpio::{Gpio, Trigger};
    use std::time::Duration;
    use std::thread;

    match DOOR_STATE.lock() {
        Ok(mut guard) => match *guard {
            State::Closed => {
                let gpio = Gpio::new().expect("failed to open gpio interface");
                let mut limit_pin = gpio.get(LIMIT_PIN).expect("failed to get limit switch pin").into_input_pullup();
                let mut mff_pin = gpio.get(MOTOR_FLIP_FLOP_PIN).expect("failed to get motor flip flop pin").into_output();
                let mut me_pin = gpio.get(MOTOR_ENABLE_PIN).expect("failed to get motor enable pin").into_output();
                mff_pin.set_reset_on_drop(false);
                me_pin.set_reset_on_drop(false);
                limit_pin.set_interrupt(Trigger::Both, Some(Duration::from_millis(10)));
    
                me_pin.set_low();
                println!("Sleeping for {MFF_SAFETY_MSECS} milliseconds");
                thread::sleep(Duration::from_millis(MFF_SAFETY_MSECS));
                mff_pin.set_low();
                me_pin.set_high();
                println!("Waiting for switch interrupt (timout {OPEN_TIMEOUT_SECS} seconds)");
                match limit_pin.poll_interrupt(true, Some(Duration::from_secs(OPEN_TIMEOUT_SECS))) {
                    Ok(None) => println!("Timeout reached, switch was not hit"),
                    Ok(Some(_)) => println!("Limit switch hit, door opened"),
                    _ => println!("Error waiting for interrupt"),
                }
                mff_pin.set_low();
                me_pin.set_low();
                *guard = State::Open;
                println!("Finished open routine");
            },
            State::Open => println!("Door already open"),
            _ => println!("Door in flight"),
        },
        Err(_) => println!("Could not aquire state lock, not opening"),
    }
}

pub fn get_settings() -> Result<Settings, SettingsIOError> {
    use std::fs::read_to_string;
    use std::path::Path;
    let settings_file = Path::new("./settings.toml");
    let settings: Settings;
    if settings_file.exists() {
        let settings_str = read_to_string("settings.toml")?;
        settings = toml::from_str(settings_str.as_str())?;
    } else {
        settings = Settings::default();
    }
    return Ok(settings);
}

pub fn write_settings(settings: Settings) -> Result<(), SettingsIOError> {
    use std::fs::write;
    use toml;
    let settings_str = toml::to_string_pretty(&settings)?;
    return Ok(write("./settings.toml", settings_str)?);
}

pub fn light_level() -> Result<f64, LightLevelError> {
    use rppal::spi::{Bus, Mode, Segment, SlaveSelect, Spi};

    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0)?;
    let write_buffer: [u8; 3] = [6, 0, 0];
    let mut read_buffer = [0u8; 5];
    
    spi.transfer(&mut read_buffer, &write_buffer);
    
    let mut result: u32 = 0;
    for (i, byte) in read_buffer.iter().enumerate() {
        result |= (u32::from(*byte) << (u32::from(2 - i as u8) * 8)) as u32;
    }
    let result = (1.0 - ((result as f64)/4096.0)) * 100.0;
    dbg!(&result);
    return Ok(result);
}

#[derive(Error, Debug)]
pub enum SettingsIOError {
    #[error("could not access settings.toml")]
    FileAccess(#[from] std::io::Error),
    #[error("could not serialize settings")]
    Serialize(#[from] toml::ser::Error),
    #[error("could deserialize settings.toml")]
    Deserialize(#[from] toml::de::Error),
}

#[derive(Error, Debug)]
pub enum LightLevelError {
    #[error("could not access MCP3208")]
    SPI(#[from] rppal::spi::Error),
}

enum State {
    Open,
    Opening,
    Closed,
    Closing,
}
