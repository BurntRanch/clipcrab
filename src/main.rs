use getopt_long::*;

fn version() {
    println!("{} version {} branch main", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    std::process::exit(0);
}

fn resolve_key<'a>(key: &'a str) -> &'a str {
    println!("key = {}", key);
    match key {
        "h" | "help" => "help",
        "V" | "version" => "version",
        _ => "unknown",
    }
}

fn parseargs() -> OptResult<()> {
    let longopts = &[
        Opt::new(Some("help".to_owned()), Some('h'), HasArg::NoArgument, "Print this help messgae").unwrap(),
        Opt::new(Some("version".to_owned()), Some('V'), HasArg::NoArgument, "Print version number along branch").unwrap()
    ];

    match getopt_long(longopts) {
        Ok(p) => {
            for (key, _) in p.args.iter() {
                match resolve_key(key) {
                    "version" => version(),
                    "help" => usage("crubclip", "blazing fast clipboard history manager", "0.1.0", longopts),
                    "unknown" => {
                        println!("Unknown option '{}'", key);
                        usage("crubclip", "blazing fast clipboard history manager", "0.1.0", longopts) 
                    }
                    _ => unreachable!()
                }
            }
        },
        Err(e) => println!("{}", e),
    }
    
    Ok(())
}

fn main() {
    let _ = parseargs();
    println!("Hello, world!");
}
