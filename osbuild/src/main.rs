use libosbuild::module::{Kind, Registry};

fn make_cli() -> clap::Command<'static> {
    clap::command!()
        .propagate_version(true)
        .about("Upgrade a package or packages on your system")
        .arg(
            clap::arg!(-q --quiet "Quiet operation (less output)")
                .required(false)
                .conflicts_with("verbose"),
        )
        .arg(
            clap::arg!(-v --verbose "Verbose operation (more output)")
                .required(false)
                .conflicts_with("quiet"),
        )
}

fn main() {
    let _matches = make_cli().get_matches();

    let registry = Registry::new_empty();

    println!("Hello, world!");
}
