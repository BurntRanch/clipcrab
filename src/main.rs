mod config;
pub use config::Config;

extern crate simplelog;

use log::error;
use simplelog::*;
use getopt_long::*;
use std::{collections::HashMap, io::Read};
use openssl::rand::rand_priv_bytes;

use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Error, Seat};
use secret_service::{SecretService, EncryptionType};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, simplelog::Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        ]
    ).unwrap();

    let mut config: config::Config = Default::default();
    let _ = config.init("/tmp/crubclip/config.toml", "/tmp/crubclip/");
    config.load_config_file("/tmp/crubclip/config.toml");

    let _ = parseargs(&config);
    
    let result = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text);

    let ss = SecretService::connect(EncryptionType::Dh).await?;

    let collection = ss.get_default_collection().await?;

    // Search for our secret. If we can't find it, create a new one.
    let item_search_results: Vec<secret_service::Item<'_>> = collection.search_items(
        HashMap::from([("label", "clipcrab")])
    ).await?;

    let new_item: Option<secret_service::Item<'_>>;
    let item: Option<&secret_service::Item<'_>>;

    println!("Reading secret.. You may receive a pop-up from your PCs Secret Service");

    if item_search_results.is_empty() {
        collection.unlock().await?;

        let mut buf = [0; 256];
        rand_priv_bytes(&mut buf).expect("openssl error");

        new_item = Some(collection.create_item(
            "clipcrab", HashMap::from([("label", "clipcrab")]),
            &buf, false, "key/plain"
        ).await?);

        item = new_item.as_ref();
    } else {
        item = Some(item_search_results.first().unwrap());
        item.unwrap().unlock().await?;
    }

    assert!((item != None));
    let _secret: Vec<u8> = item.unwrap().get_secret().await?;

    item.unwrap().lock().await?;

    match result {
        Ok((mut pipe, _mime_type)) => {
            let mut contents = vec![];
            pipe.read_to_end(&mut contents)?;
            println!("Clipboard content: {}", String::from_utf8(contents).expect("Clipboard contents should be valid utf8."));
        }

        Err(Error::NoSeats) | Err(Error::ClipboardEmpty) | Err(Error::NoMimeType) => {
            // The clipboard is empty, nothing to worry about.
            println!("Error: Clipboard empty.");
        }

        Err(err) => Err(err)?
    };

    Ok(())
}
