mod config;
pub use config::Config;

extern crate simplelog;

use log::error;
use simplelog::*;
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
        _ => key,
    }
}

fn parseargs<'a>(config: &'a config::Config) -> OptResult<()> {
    let longopts = &[
        Opt::new(Some("help".to_owned()), Some('h'), HasArg::NoArgument, "Print this help messgae").unwrap(),
        Opt::new(Some("version".to_owned()), Some('V'), HasArg::NoArgument, "Print version number along branch").unwrap(),
        Opt::new(Some("gen-config".to_owned()), None, HasArg::RequiredArgument, "Generate config at path").unwrap(),
    ];

    match getopt_long(longopts) {
        Ok(p) => {
            for (key, value) in p.args.iter() {
                match resolve_key(key) {
                    "version" => version(),
                    "help" => usage("crubclip", "blazing fast clipboard history manager", env!("CARGO_PKG_VERSION"), longopts),
                    "gen-config" => config.generate_config(value).unwrap(),
                    _ => {
                        error!("Unknown option '{}'", key);
                        usage("crubclip", "blazing fast clipboard history manager", env!("CARGO_PKG_VERSION"), longopts) 
                    }
                }
            }
        },
        Err(e) => println!("{}", e),
    }
    
    Ok(())
}

fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, simplelog::Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        ]
    ).unwrap();

    let mut config: config::Config = Default::default();
    let _ = config.init("/tmp/crubclip/config.toml", "/tmp/crubclip/");
    config.load_config_file("/tmp/crubclip/config.toml");

    let _ = parseargs(&config);

    println!("Hello, world!");
}
