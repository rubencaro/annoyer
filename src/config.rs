#[derive(Debug)]
pub struct Config {
    pub concurrency: u32,
    pub url: hyper::Uri,
}

pub fn get_config() -> Config {
    let matches = build_matches();

    Config {
        concurrency: parse_u32(&matches, "concurrency", 10),
        url: parse_url(&matches, "url"),
    }
}

fn parse_u32(matches: &clap::ArgMatches<'_>, name: &str, default: u32) -> u32 {
    value_t!(matches, name, u32).unwrap_or_else(|e| {
        println!("{}\nUsing default one.", e);
        default
    })
}

fn parse_url(matches: &clap::ArgMatches<'_>, name: &str) -> hyper::Uri {
    matches
        .value_of(name)
        .unwrap()
        .parse::<hyper::Uri>()
        .unwrap()
}

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
