mod db;
mod installer;
mod monitors;
mod server;
mod win;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use colored::Colorize;

use monitors::PowerMode;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(SubCommand::with_name("run").about("Runs the command server"))
        .subcommand(SubCommand::with_name("list").about("Lists all connected monitors"))
        .subcommand(
            SubCommand::with_name("on")
                .about("Turns on the specified monitor")
                .arg(
                    Arg::with_name("id")
                        .required(true)
                        .default_value("all")
                        .help("The id of the monitor to turn on, or 'all' to turn on all monitors"),
                ),
        )
        .subcommand(
            SubCommand::with_name("off")
                .about("Turns off the specified monitor")
                .arg(
                    Arg::with_name("id")
                        .required(true)
                        .default_value("all")
                        .help(
                            "The id of the monitor to turn off, or 'all' to turn off all monitors",
                        ),
                ),
        )
        .subcommand(
            SubCommand::with_name("install")
                .about("Installs the scheduled task to start mona on login"),
        )
        .subcommand(
            SubCommand::with_name("uninstall")
                .about("Removes the scheduled task to start mona on login"),
        )
        .get_matches();

    match matches.subcommand() {
        ("run", _) => server::run().unwrap(),
        ("list", _) => list_monitors(),
        ("on", Some(matches)) => set_power_mode(matches, PowerMode::On),
        ("off", Some(matches)) => set_power_mode(matches, PowerMode::Off),
        ("install", _) => installer::install().unwrap(),
        ("uninstall", _) => installer::uninstall().unwrap(),
        _ => {}
    }
}

fn list_monitors() {
    let monitors = monitors::get_monitors();

    if monitors.is_empty() {
        println!("\nNo monitors found");
        return;
    }

    println!(
        "\n{} {}\n",
        monitors.len().to_string().yellow(),
        "monitor(s) found:".yellow()
    );

    println!("    id {} name", "|".bright_black());
    println!(
        "{}",
        "    --------------------------------------".bright_black()
    );

    for monitor in monitors {
        let id = monitor.id().to_string();
        let name = monitor.name().to_string();
        let separator = "|".bright_black();

        match monitor.power_mode() {
            PowerMode::On => println!("    {:2} {} {}", id.green(), separator, name.green()),
            PowerMode::Off => println!("    {:2} {} {}", id.red(), separator, name.red()),
        };
    }
}

fn set_power_mode(matches: &ArgMatches, power_mode: PowerMode) {
    let id = matches.value_of("id").unwrap();
    let monitors = monitors::get_monitors();
    if id == "all" {
        monitors
            .iter()
            .for_each(|monitor| monitor.set_power_mode(power_mode).unwrap());
    } else {
        let id: usize = id.parse().unwrap();
        let monitor = monitors
            .get(id - 1)
            .expect("no monitor found with the given id");
        monitor.set_power_mode(power_mode).unwrap();
    }

    println!("\nOk 👍");
}
