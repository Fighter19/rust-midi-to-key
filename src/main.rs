use evdev::{Device, Key};
use anyhow::{Result};

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
