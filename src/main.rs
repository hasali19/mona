mod monitors;

use clap::{App, AppSettings, SubCommand};

fn main() {
    let matches = App::new("monitor_control")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(SubCommand::with_name("list").about("Lists all connected monitors"))
        .get_matches();

    match matches.subcommand_name() {
        Some("list") => list_monitors(),
        _ => {}
    }
}

fn list_monitors() {
    let monitors = monitors::get_monitors();

    if monitors.is_empty() {
        println!("\nNo monitors found");
        return;
    }

    println!("\n{} monitor(s) found:\n", monitors.len());
    println!("    id | name");
    println!("    --------------------------------------");

    for monitor in monitors {
        println!("    {:2} | {:?}", monitor.id(), monitor.name());
    }
}
