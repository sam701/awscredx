extern crate clap;
extern crate rusoto_core;
extern crate rusoto_credential;

mod credentials;

fn main() {
    let matches = clap::App::new("awscredx")
        .version("0.1.0")
        .about("AWS credentials management, a.k.a. role assumption made easy")
        .subcommand(clap::SubCommand::with_name("assume-role")
            .about("Prints shell commands to assume a given role"))
        .subcommand(clap::SubCommand::with_name("init")
            .about("Initializes local environment"))
        .get_matches();
    println!("Hello, world!");
}
