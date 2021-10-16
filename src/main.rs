use evdev::{Device, Key};
use anyhow::{Result};

use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent};
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<()> {
    let device = Device::open("/dev/input/event9")?;
    // This is example from the documentation of the evdev crate, checks if device has an enter key
    if device.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
        println!("are you prepared to ENTER the world of evdev?");
    } else {
        println!(":(");
    }
    Ok(())
}

#[test]
fn test_create_device() -> Result<()> {
    let mut keys = AttributeSet::<Key>::new();
    keys.insert(Key::KEY_Z);

    let mut device = VirtualDeviceBuilder::new()?
        .name("Fake Keyboard")
        .with_keys(&keys)?
        .build()
        .unwrap();

    let type_ = EventType::KEY;
    // Note this will ACTUALLY PRESS the button on your computer.
    // Hopefully you don't have BTN_DPAD_UP bound to anything important.
    //let code = Key::BTN_DPAD_UP.code();
    let code = Key::KEY_Z.code();

    sleep(Duration::from_secs(2));
    let down_event = InputEvent::new(type_, code, 1);
    device.emit(&[down_event]).unwrap();
    println!("Pressed.");
    sleep(Duration::from_secs(2));

    let up_event = InputEvent::new(type_, code, 0);
    device.emit(&[up_event]).unwrap();
    println!("Released.");
    sleep(Duration::from_secs(2));
    Ok(())
}