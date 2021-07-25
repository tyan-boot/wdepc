use crate::device::{Device, PowerMode};
use anyhow::Result;
use clap::{App, AppSettings, Arg, SubCommand};
use device::EPCSetting;

mod device;
mod ffi;

fn main() -> Result<()> {
    let args = App::new("wdepc")
        .about("Western Digital EPC(Extended Power Condition) control tools")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(clap::crate_version!())
        .author("tyanboot <tyanboot@outlook.com>")
        .subcommand(SubCommand::with_name("check").about("Check device power mode"))
        .subcommand(SubCommand::with_name("info").about("Show device EPC settings"))
        .subcommand(SubCommand::with_name("enable").about("Enable EPC and disable APM"))
        .subcommand(SubCommand::with_name("disable").about("Disable EPC, doesn't enable APM"))
        .subcommand(
            SubCommand::with_name("set-timer")
                .about("Set Power Condition timer")
                .arg(
                    Arg::with_name("save")
                        .help("save setting")
                        .long("save")
                        .short("s"),
                )
                .arg(
                    Arg::with_name("enable")
                        .help("enable timer")
                        .long("enable")
                        .short("e")
                        .possible_values(&["true", "false"])
                        .default_value("true")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("mode")
                        .help("power mode to set")
                        .takes_value(true)
                        .possible_values(&["idle_a", "idle_b", "idle_c", "standby_y", "standby_z"])
                        .required(true),
                )
                .arg(
                    Arg::with_name("timer")
                        .help("timer, unit in 100 milliseconds")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("set-state")
                .about("Set Power Condition state")
                .arg(
                    Arg::with_name("save")
                        .help("save setting")
                        .long("save")
                        .short("s"),
                )
                .arg(
                    Arg::with_name("enable")
                        .help("enable timer")
                        .long("enable")
                        .short("e")
                        .possible_values(&["true", "false"])
                        .default_value("true")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("mode")
                        .help("power mode to set")
                        .takes_value(true)
                        .possible_values(&["idle_a", "idle_b", "idle_c", "standby_y", "standby_z"])
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("Force device goto specific power mode")
                .arg(
                    Arg::with_name("mode")
                        .help("power mode to set")
                        .takes_value(true)
                        .possible_values(&["idle_a", "idle_b", "idle_c", "standby_y", "standby_z"])
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("restore")
                .about(r#"Restore EPC settings
if default is set, set current timer to default, else set current timer to saved timer.
if save is set, save current timer
                "#)
                .arg(
                    Arg::with_name("default")
                        .long("default")
                        .short("d")
                        .help("Restore from default")
                )
                .arg(
                    Arg::with_name("save")
                        .long("save")
                        .short("s")
                        .help("Save current EPC settings(after restore)")
                )
                .arg(
                    Arg::with_name("mode")
                        .help("power mode to set")
                        .takes_value(true)
                        .possible_values(&["idle_a", "idle_b", "idle_c", "standby_y", "standby_z"])
                        .required(true),
                )
        )
        .arg(
            Arg::with_name("device")
                .long("device")
                .short("d")
                .help("device path, eg /dev/sda")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let device = args.value_of("device").unwrap();
    let mut device = Device::open(device)?;

    match args.subcommand() {
        ("info", _) => {
            let setting = device.query_epc_setting()?;

            let EPCSetting {
                idle_a,
                idle_b,
                idle_c,
                standby_y,
                standby_z,
                ..
            } = setting;

            println!("* = enabled");
            println!("All times are in 100 milliseconds");
            println!();

            println!(
                "{:<9}  {:<13} {:<13} {:<11} {:<13} {:<10} {:<7}",
                "Name",
                "Current Timer",
                "Default Timer",
                "Saved Timer",
                "Recovery Time",
                "Changeable",
                "Savable"
            );

            println!(
                "{:<9}  {:<13} {:<13} {:<11} {:<13} {:<10} {:<7}",
                "Idle A",
                if idle_a.current_enable {
                    format!("*{}", idle_a.current_timer)
                } else {
                    idle_a.current_timer.to_string()
                },
                if idle_a.default_enable {
                    format!("*{}", idle_a.default_timer)
                } else {
                    idle_a.default_timer.to_string()
                },
                if idle_a.saved_enable {
                    format!("*{}", idle_a.saved_timer)
                } else {
                    idle_a.saved_timer.to_string()
                },
                idle_a.recovery_time,
                idle_a.changeable,
                idle_a.savable
            );

            println!(
                "{:<9}  {:<13} {:<13} {:<11} {:<13} {:<10} {:<7}",
                "Idle B",
                if idle_b.current_enable {
                    format!("*{}", idle_b.current_timer)
                } else {
                    idle_b.current_timer.to_string()
                },
                if idle_b.default_enable {
                    format!("*{}", idle_b.default_timer)
                } else {
                    idle_b.default_timer.to_string()
                },
                if idle_b.saved_enable {
                    format!("*{}", idle_b.saved_timer)
                } else {
                    idle_b.saved_timer.to_string()
                },
                idle_b.recovery_time,
                idle_b.changeable,
                idle_b.savable
            );

            println!(
                "{:<9}  {:<13} {:<13} {:<11} {:<13} {:<10} {:<7}",
                "Idle C",
                if idle_c.current_enable {
                    format!("*{}", idle_c.current_timer)
                } else {
                    idle_c.current_timer.to_string()
                },
                if idle_c.default_enable {
                    format!("*{}", idle_c.default_timer)
                } else {
                    idle_c.default_timer.to_string()
                },
                if idle_c.saved_enable {
                    format!("*{}", idle_c.saved_timer)
                } else {
                    idle_c.saved_timer.to_string()
                },
                idle_c.recovery_time,
                idle_c.changeable,
                idle_c.savable
            );

            println!(
                "{:<9}  {:<13} {:<13} {:<11} {:<13} {:<10} {:<7}",
                "Standby Y",
                if standby_y.current_enable {
                    format!("*{}", standby_y.current_timer)
                } else {
                    standby_y.current_timer.to_string()
                },
                if standby_y.default_enable {
                    format!("*{}", standby_y.default_timer)
                } else {
                    standby_y.default_timer.to_string()
                },
                if standby_y.saved_enable {
                    format!("*{}", standby_y.saved_timer)
                } else {
                    standby_y.saved_timer.to_string()
                },
                standby_y.recovery_time,
                standby_y.changeable,
                standby_y.savable
            );

            println!(
                "{:<9}  {:<13} {:<13} {:<11} {:<13} {:<10} {:<7}",
                "Standby Z",
                if standby_z.current_enable {
                    format!("*{}", standby_z.current_timer)
                } else {
                    standby_z.current_timer.to_string()
                },
                if standby_z.default_enable {
                    format!("*{}", standby_z.default_timer)
                } else {
                    standby_z.default_timer.to_string()
                },
                if standby_z.saved_enable {
                    format!("*{}", standby_z.saved_timer)
                } else {
                    standby_z.saved_timer.to_string()
                },
                standby_z.recovery_time,
                standby_z.changeable,
                standby_z.savable
            );
        }
        ("set-timer", Some(args)) => {
            let mode = args.value_of("mode").unwrap();
            let timer: u16 = args
                .value_of("timer")
                .and_then(|it| it.parse().ok())
                .unwrap();

            let save = args.is_present("save");
            let enable: bool = args
                .value_of("enable")
                .and_then(|it| it.parse().ok())
                .unwrap();

            let mode = match mode {
                "idle_a" => PowerMode::IdleA,
                "idle_b," => PowerMode::IdleB,
                "idle_c" => PowerMode::IdleC,
                "standby_y" => PowerMode::StandbyY,
                "standby_z" => PowerMode::StandbyZ,
                _ => unreachable!(),
            };

            device.set_timer(mode, timer, enable, save)?;
        }
        ("set-state", Some(args)) => {
            let mode = args.value_of("mode").unwrap();

            let save = args.is_present("save");
            let enable: bool = args
                .value_of("enable")
                .and_then(|it| it.parse().ok())
                .unwrap();

            let mode = match mode {
                "idle_a" => PowerMode::IdleA,
                "idle_b," => PowerMode::IdleB,
                "idle_c" => PowerMode::IdleC,
                "standby_y" => PowerMode::StandbyY,
                "standby_z" => PowerMode::StandbyZ,
                _ => unreachable!(),
            };

            device.set_state(mode, enable, save)?;
        }
        ("set", Some(args)) => {
            let mode = args.value_of("mode").unwrap();
            let mode = match mode {
                "idle_a" => PowerMode::IdleA,
                "idle_b," => PowerMode::IdleB,
                "idle_c" => PowerMode::IdleC,
                "standby_y" => PowerMode::StandbyY,
                "standby_z" => PowerMode::StandbyZ,
                _ => unreachable!(),
            };

            device.goto_cond(mode)?;
        }
        ("enable", _) => {
            device.enable_epc()?;
        }
        ("disable", _) => {
            device.disable_epc()?;
        }
        ("restore", Some(args)) => {
            let mode = args.value_of("mode").unwrap();
            let mode = match mode {
                "idle_a" => PowerMode::IdleA,
                "idle_b," => PowerMode::IdleB,
                "idle_c" => PowerMode::IdleC,
                "standby_y" => PowerMode::StandbyY,
                "standby_z" => PowerMode::StandbyZ,
                _ => unreachable!(),
            };

            let default = args.is_present("default");
            let save = args.is_present("save");

            device.restore(mode, default, save)?;
        }
        ("check", _) => {
            let mode = device.query_mode()?;
            let mode = match mode {
                PowerMode::Active => "active or idle",
                PowerMode::IdleA => "idle a",
                PowerMode::IdleB => "idle b",
                PowerMode::IdleC => "idle c",
                PowerMode::StandbyY => "standby y",
                PowerMode::StandbyZ => "standby z",
                PowerMode::Unknown => "unknown",
            };

            println!("{}", mode);
        }
        _ => {}
    }

    Ok(())
}
