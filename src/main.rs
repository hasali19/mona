mod monitors;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use colored::Colorize;

use monitors::PowerMode;

fn main() {
    let matches = App::new("monitor_control")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
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
        .get_matches();

    match matches.subcommand() {
        ("list", _) => list_monitors(),
        ("on", Some(matches)) => set_power_mode(matches, PowerMode::On),
        ("off", Some(matches)) => set_power_mode(matches, PowerMode::Off),
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

    println!("    {} {} {}", "id", "|".bright_black(), "name");
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
            .for_each(|monitor| monitor.set_power_mode(power_mode));
    } else {
        let id: usize = id.parse().unwrap();
        let monitor = monitors
            .get(id - 1)
            .expect("no monitor found with the given id");
        monitor.set_power_mode(power_mode);
    }

    println!("\nOk 👍");
}
