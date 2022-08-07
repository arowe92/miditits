use std::error::Error;

use midir::{MidiInput, Ignore};

mod ui {
    use std::io::{stdin, stdout, Write};
    pub fn ask_for_port_input(mut input: &mut String, port_names: &[String] ) {
        println!("\nAvailable input ports:");
        for (i, p) in port_names.iter().enumerate() {
            println!("{}: {}", i, p);
        }
        print!("Please select input port: ");
        stdout().flush().unwrap();
        let _ = stdin().read_line(&mut input);
    }
    pub fn notify_default_port(port_name: &str) {
        println!("Choosing the only available input port: {}", port_name);
    }

    pub fn notify_open_success() {
        println!("\nOpening connection");
    }
    pub fn notify_port_read(in_port_name: &str) {
        println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);
    }
    pub fn log_midi_msg(timestamp: u64, message: &[u8]) {
        println!("{}: {:?} (len = {})", timestamp, message, message.len());
    }
    pub fn wait_for_input(mut input: &mut String) {
        input.clear();
        stdin().read_line(&mut input).unwrap(); // wait for next enter key press
    }

}
pub fn run() -> Result<(), Box<dyn Error>> {
    let mut ui_input = String::new();

    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read user input if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            let default_port_name = midi_in.port_name(&in_ports[0]).unwrap();
            ui::notify_default_port(&default_port_name);
            &in_ports[0]
        },
        _ => {
            // Enumerate all ports and prompt user for a selection.
            let port_names = in_ports.iter()
                .map(|p| {
                    let name = midi_in.port_name(&p).unwrap();
                    String::from(name)
                })
                .collect::<Vec<String>>();

            ui::ask_for_port_input(&mut ui_input, &port_names);
            in_ports.get(ui_input.trim().parse::<usize>()?)
                     .ok_or("invalid input port selected")?
        }
    };

    ui::notify_open_success();
    let in_port_name = midi_in.port_name(in_port)?;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |stamp, message, _| {
        ui::log_midi_msg(stamp, &message);
    }, ())?;

    ui::notify_port_read(&in_port_name);
    ui::wait_for_input(&mut ui_input);
    Ok(())
}
