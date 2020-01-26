mod host;

use clap::{SubCommand, Arg, App};
use std::net::IpAddr;
use std::process::exit;
use regex::Regex;

fn main() {
    let matches = App::new("netf")
        .name("netf")
        .about("Sends files over network to a target machine\nCURRENTLY UNENCRYPTED!!!")
        .subcommand(SubCommand::with_name("host")
            .arg(Arg::with_name("file")
                .short("-f")
                .long("file")
                .takes_value(true)
                .multiple(false)
                .required(true))
            .arg(Arg::with_name("target")
                .short("t")
                .long("target")
                .takes_value(true)
                .multiple(false)
                .default_value("0.0.0.0:1742"))
        )
        .get_matches();

    if let Some(sub_matches) = matches.subcommand_matches("host") {
        let file = sub_matches.value_of("file").unwrap().to_string();
        let target = sub_matches.value_of("target").unwrap().to_string();

        verify_target(&target);

        host::host_file_tcp(file, target);

        exit(0);
    }
}

fn verify_target(target: &String) -> Option<String> {
    let port_match = Regex::new(":([1-6][0-9]{0,4}|[1-9][0-9]{0,3})").unwrap();
    let res = match port_match.captures(target) {
        Some(x) => x,
        None => {
            eprintln!("Error processing target");
            exit(1);
        }
    };
    println!("{} {}", res.get(0).unwrap().as_str(), res.len());
    None
}
