use evdev::{Device, Key};
use anyhow::{Result};

use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent};
use std::thread::sleep;
use std::time::Duration;
use std::io::{stdin, stdout, Write};

use midir::{MidiInput, Ignore};

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

#[test]
fn test_midir() -> Result<()> {
    let mut input = String::new();

    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);
    
    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err(anyhow::anyhow!("no input port found")),
        1 => {
            println!("Choosing the only available input port: {}", midi_in.port_name(&in_ports[0]).unwrap());
            &in_ports[0]
        },
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports.get(input.trim().parse::<usize>()?)
                     .ok_or(anyhow::anyhow!("invalid input port selected"))?
        }
    };
    
    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |stamp, message, _| {
        println!("{}: {:?} (len = {})", stamp, message, message.len());
    }, ());

    if _conn_in.is_err() {
        println!("Failed to open MIDI Input connection");
    }
    
    println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}