// Dev tool: print every MIDI message from every input for a few seconds.
use std::time::Duration;
fn main() {
    let input = midir::MidiInput::new("probe").unwrap();
    let mut conns = vec![];
    for p in input.ports() {
        let name = input.port_name(&p).unwrap();
        println!("port: {name}");
        let i = midir::MidiInput::new("probe").unwrap();
        let n = name.clone();
        conns.push(i.connect(&p, "probe", move |_, b, _| println!("{n}: {b:?}"), ()).unwrap());
    }
    std::thread::sleep(Duration::from_secs(std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(4)));
}
