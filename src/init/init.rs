use std::env;

use crate::util;

pub const BINARY_NAME: &str = env!("CARGO_PKG_NAME");

pub enum InitType {
    Bootstrap,
    Full,
}

pub fn run(shell: &str, init_type: InitType) {
    let buf = env::current_exe().unwrap();
    let bin = buf.to_str().unwrap();

    use InitType::*;
    match init_type {
        Bootstrap => {
            let cmd = format!(r#""{bin}" init --full {shell}"#, bin = bin, shell = shell);
            if shell == "fish" {
                print!(r#"source ({cmd} | psub)"#, cmd = cmd);
            } else {
                print!(r#"source <({cmd})"#, cmd = cmd);
            }
        }
        Full => {
            print_delete_deprecated_script("script.sh");
            print_delete_deprecated_script("script.fish");
            let tmpl = if shell == "fish" {
                include_str!("templates/script.fish")
            } else {
                include_str!("templates/script.sh")
            }
            .replace("@bin@", super::BINARY_NAME);
            println!("{}", tmpl);
        }
    }
}

fn print_delete_deprecated_script(file: &str) {
    let dir = util::path_to_absolute(util::STORAGE_DIR);
    let file = dir.join(file);
    if file.exists() {
        println!("rm -r {}", file.to_str().unwrap());
    }
}
