use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;

fn main() {
    let version = env!("CARGO_PKG_VERSION");

    let mut cli_yml_in = OpenOptions::new()
        .read(true)
        .open("src/bin/cli.yml").unwrap();

    let mut cli_string = String::new();
    cli_yml_in.read_to_string(&mut  cli_string).unwrap();

    cli_string = cli_string.replace("@version_string@", version);

    let mut cli_yml_out = OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .open("src/bin/cli-build.yml").unwrap();

    cli_yml_out.write_all(cli_string.as_bytes()).unwrap();
}