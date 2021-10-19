use evdev::{Device, Key};
use anyhow::{Result};

use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent};
use std::convert::TryInto;
use std::iter::Map;
use std::thread::sleep;
use std::time::Duration;
use std::io::{stdin, stdout, Write};

use midir::{ConnectError, Ignore, MidiInput, MidiInputConnection};

struct KeyboardMsg {
    is_press: bool,
    button_to_press: u8
}

fn receive_midi_msg_for_device(device: &mut evdev::uinput::VirtualDevice, _stamp: u64, message: &[u8], octave_pitch: i8) {
    // 144 is normal press, 128 is deactivating key
    if message[0] == 144 || message[0] == 128 {
        // 75 = D#5
        // 72 => C5
        // 72-12 => 60 => C4

        let bound_pitch: u8 = match (60-12-octave_pitch * 12).try_into() {
            Ok(value) => value,
            Err(err) => {
                println!("Value out of current pitch, clamping");
                0
            },
        };

        // message[1] starts at C3
        let mut note: u8 = message[1] - bound_pitch;
        note = std::cmp::max(note, 0);
        note = std::cmp::min(note, 12+12+11);

        let msg = KeyboardMsg {
            is_press: (message[0] == 144 && message[2] >= 50),
            button_to_press: note
        };
        generate_button_press(device, msg);
    }
}

const BUTTON_LUT: [evdev::Key; 12*3] = [
    Key::KEY_Z,
    Key::KEY_X,
    Key::KEY_S,
    Key::KEY_D,
    Key::KEY_E,
    Key::KEY_F,
    Key::KEY_G,
    Key::KEY_B,
    Key::KEY_H,
    Key::KEY_N,
    Key::KEY_J,
    Key::KEY_M,

    Key::KEY_Q,
    Key::KEY_2,
    Key::KEY_W,
    Key::KEY_3,
    Key::KEY_E,
    Key::KEY_R,
    Key::KEY_5,
    Key::KEY_T,
    Key::KEY_6,
    Key::KEY_Y,
    Key::KEY_7,
    Key::KEY_U,

    Key::KEY_I,
    Key::KEY_8,
    Key::KEY_O,
    Key::KEY_9,
    Key::KEY_P,
    Key::KEY_LEFTBRACE,
    Key::KEY_0,
    Key::KEY_RIGHTBRACE,
    Key::KEY_MINUS,
    Key::KEY_COMMA,
    Key::KEY_DOT,
    Key::KEY_1
];

fn generate_button_press(device: &mut evdev::uinput::VirtualDevice, keyboard_msg: KeyboardMsg) {
    //println!("Pressed button: {}", keyboard_msg.button_to_press);
    //println!("Would press button: {}", button_lut[keyboard_msg.button_to_press as usize]);
    let type_ = EventType::KEY;

    let press_event = InputEvent::new(type_, BUTTON_LUT[keyboard_msg.button_to_press as usize].code(), keyboard_msg.is_press as i32);
    device.emit(&[press_event]).unwrap();
}

fn initialize_kbd_device() -> Result<evdev::uinput::VirtualDevice, std::io::Error> {
    let mut keys = AttributeSet::<Key>::new();
    for key in BUTTON_LUT.iter() {
        keys.insert(*key);
    }

    let mut device = VirtualDeviceBuilder::new()?
        .name("Fake Keyboard")
        .with_keys(&keys)?
        .build();
    return device;
}

fn initialize_midi_device<F, T: Send> (callback: F, data: T) -> Result<MidiInputConnection<T>>
where F: FnMut(u64, &[u8], &mut T) + Send + 'static {
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
    let _conn_in = midi_in.connect(in_port, "midir-read-input", callback, data);

    if _conn_in.is_err() {
        println!("Failed to open MIDI Input connection");
    }

    println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);
    input.clear();
    match _conn_in {
        Ok(con) => Ok(con),
        Err(why) => Err(anyhow::anyhow!("Unable to open Midi Input Connection"))
    }

}

// impl From<ConnectError<MidiInput>> for anyhow::Error {
//     fn from(error: ConnectError<MidiInput>) {
//         anyhow::anyhow!("Failed to connect Midi Input")
//     } 
// }

fn main() -> Result<()> {
    let device = initialize_kbd_device()?;
    let pitch = -1;
    let midi_device = initialize_midi_device(move |stamp, message, device| {
        println!("{}: {:?} (len = {})", stamp, message, message.len());
        receive_midi_msg_for_device(device, stamp, message,pitch);
        //receive_midi_msg(stamp, message);
    }, device)?;
    let mut input = String::new();
    stdin().read_line(&mut input)?; // wait for next enter key press

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
    let midi_device = initialize_midi_device(move |stamp, message, _| {
        println!("{}: {:?} (len = {})", stamp, message, message.len());
        //receive_midi_msg(stamp, message);
    }, ())?;

    let mut input = String::new();
    stdin().read_line(&mut input)?; // wait for next enter key press
    println!("Closing connection");
    Ok(())
}