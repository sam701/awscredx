extern crate clap;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_sts;
extern crate toml;
extern crate serde;
extern crate custom_error;
extern crate chrono;

mod config;
mod credentials;
mod init;
mod assume;

fn main() {
    let matches = clap::App::new("awscredx")
        .version("0.1.0")
        .about("AWS credentials management, a.k.a. role assumption made easy")
        .subcommand(clap::SubCommand::with_name("assume-role")
            .about("Prints shell commands to assume a given role"))
        .subcommand(clap::SubCommand::with_name("init")
            .about("Initializes local environment"))
        .get_matches();

    match matches.subcommand_name() {
        Some("assume-role") => unimplemented!(),
        Some("init") => init::run(),
        _ => print_first_time_message()
    }
}

fn print_first_time_message() {
    println!(r#"Welcome to awscredx!

It seems you are running this command for the first time.
Call 'awscredx init' to create the configuration file template and setup a shell helper function."#);
}