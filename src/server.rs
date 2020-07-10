use monitors::PowerMode;
use std::error::Error;
use std::net::UdpSocket;

use crate::monitors;

pub fn run() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let socket = UdpSocket::bind("0.0.0.0:7890")?;
    let mut buffer = [0; 8];

    log::info!("server running on port 7890...");

    loop {
        let (received, from) = match socket.recv_from(&mut buffer) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let buffer = &buffer[..received];
        let message = String::from_utf8_lossy(&buffer);

        log::debug!("received: {}", message);

        let (cmd, args) = match parse_message(&message) {
            Some(v) => v,
            None => {
                log::error!("invalid message format: {}", message);
                continue;
            }
        };

        let res = match cmd {
            "list" => list(),
            "set" => set_power_mode(args),
            _ => {
                log::error!("invalid command: {}({})", cmd, args.join(","));
                continue;
            }
        };

        match res {
            Ok(response) => {
                socket.send_to(response.as_bytes(), from).ok();
            }
            Err(e) => {
                log::error!("{}", e);
                // TODO: Improve error responses
                socket.send_to("error".as_bytes(), from).ok();
            }
        }
    }
}

fn list() -> Result<String, Box<dyn Error>> {
    let monitors = monitors::get_monitors();
    let mut response = String::new();

    for monitor in monitors {
        response.push_str(&format!(
            "{};{};{}\n",
            monitor.id(),
            monitor.name(),
            encode_power_mode(monitor.power_mode())
        ));
    }

    Ok(response)
}

fn set_power_mode(args: Vec<&str>) -> Result<String, Box<dyn Error>> {
    if args.len() != 2 {
        Err(format!(
            "invalid arguments ({}): {}",
            args.len(),
            args.join(",")
        ))?;
    }

    let id = args[0].parse()?;
    let monitors = monitors::get_monitors();
    let monitor = monitors
        .iter()
        .find(|m| m.id() == id)
        .ok_or_else(|| format!("no monitor found with id {}", id))?;

    let mode = args[1];
    let mode = decode_power_mode(mode).ok_or_else(|| format!("invalid power mode: {}", mode))?;

    monitor.set_power_mode(mode).map(|_| "ok".to_owned())
}

fn encode_power_mode(mode: PowerMode) -> char {
    match mode {
        PowerMode::Off => '1',
        PowerMode::On => '2',
    }
}

fn decode_power_mode(value: &str) -> Option<PowerMode> {
    match value {
        "1" => Some(PowerMode::Off),
        "2" => Some(PowerMode::On),
        _ => None,
    }
}

fn parse_message(message: &str) -> Option<(&str, Vec<&str>)> {
    let i = message.chars().take_while(|&c| c != ':').count();
    if i < message.len() {
        Some((&message[..i], message[i + 1..].split(',').collect()))
    } else {
        None
    }
}
