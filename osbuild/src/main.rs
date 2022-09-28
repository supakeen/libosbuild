use libosbuild::module::Registry;

fn make_cli() -> clap::Command<'static> {
    clap::command!()
        .propagate_version(true)
        .about("Build operating system images.")
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
        .arg(clap::arg!(-m --module <module> "Path to module(s)").required(false))
        .arg(clap::arg!(<manifest> "Path to manifest to build"))
}

fn main() {
    let _matches = make_cli().get_matches();
    let _registry = Registry::new_empty();

    println!("Hello, world!");
}
