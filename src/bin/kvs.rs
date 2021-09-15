extern crate clap;
use clap::{Arg, App, SubCommand};
use std::env::Args;


fn main() {
    let matches = App::new("kv")
        .version("1.0")
        .author("ztelur")
        .about("kv client")
        .arg(Arg::with_name("version")
            .short("V")
            .long("version"))
        .subcommand(SubCommand::with_name("get")
            .about(" get value")
            .version("1.0")
            .author("ztelur")
            .arg(Arg::with_name("key")
                .help("the key")
                .index(1)
                .required(true))
        ).subcommand(SubCommand::with_name("set")
            .about("set key value")
        .version("1.0")
        .arg(Arg::with_name("key")
            .help("the key")
            .index(1)
            .required(true))
        .arg(Arg::with_name("value")
            .help("the value")
            .index(2)
            .required(true))
        ).subcommand(SubCommand::with_name("rm")
                                           .arg(Arg::with_name("key")
                                            .index(1)
            .required(true))).get_matches();



    match matches.subcommand_name() {
        Some("set") => panic!(),
        Some("get") => panic!(),
        Some("del") => panic!(),
        _ => panic!()

    }
}