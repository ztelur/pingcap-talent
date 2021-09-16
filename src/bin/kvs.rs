extern crate clap;

use clap::{App, AppSettings, Arg, SubCommand};
use std::process::exit;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(
            SubCommand::with_name("get")
                .about(" get value")
                .version("1.0")
                .author("ztelur")
                .arg(Arg::with_name("key").help("the key").required(true)),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("set key value")
                .version("1.0")
                .arg(
                    Arg::with_name("key")
                        .help("the key")
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::with_name("value")
                        .help("the value")
                        .index(2)
                        .required(true),
                ),
        )
        .subcommand(SubCommand::with_name("rm").arg(Arg::with_name("key").index(1).required(true)))
        .get_matches();

    match matches.subcommand() {
        ("set", Some(_matches)) => {
            eprintln!("unimplemented");
            println!(
                "set key {} to value {}",
                _matches.value_of("key").unwrap(),
                _matches.value_of("value").unwrap()
            );
            exit(1)
        }
        ("get", Some(_matches)) => {
            eprintln!("unimplemented");
            println!("get key {}", _matches.value_of("key").unwrap());
            exit(1)
        }
        ("rm", Some(_matches)) => {
            eprintln!("unimplemented");
            println!("del key {}", _matches.value_of("key").unwrap());
            exit(1)
        }
        _ => unreachable!(),
    }
}
