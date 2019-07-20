//! The (not so) boring config reading and parsing module

/// The struct where the parsed config is stored
#[derive(Debug)]
pub struct Config {
    pub concurrency: u32,
    pub url: hyper::Uri,
}

/// Use this to get the `Config` struct filled in with parsed values
pub fn get_config() -> Config {
    let matches = build_matches();

    Config {
        concurrency: parse_u32(&matches, "concurrency", 10),
        url: parse_url(&matches, "url"),
    }
}

/// Parse a u32 from given `name` in `matches` struct. Default is used if anything fails.
fn parse_u32(matches: &clap::ArgMatches<'_>, name: &str, default: u32) -> u32 {
    value_t!(matches, name, u32).unwrap_or_else(|e| {
        println!("{}\nUsing default one.", e);
        default
    })
}

/// Parse a valid `hyper::Uri` from given `name` in `matches` struct. Panic if anything fails.
fn parse_url(matches: &clap::ArgMatches<'_>, name: &str) -> hyper::Uri {
    matches
        .value_of(name)
        .unwrap()
        .parse::<hyper::Uri>()
        .unwrap()
}

/// Get the `clap` matches struct from input args.
fn build_matches() -> clap::ArgMatches<'static> {
    clap::App::new("Annoyer")
        .version("0.1.0")
        .about("Annoying HTML load generator.")
        .args_from_usage(
            "-u, --url <string> 'The URL to be called'
             -c, --concurrency <number> 'Indicates the number of parallel workers'",
        )
        .get_matches()
}
