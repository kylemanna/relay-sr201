use regex::Regex;
use structopt::StructOpt;

use relay_sr201;

#[derive(StructOpt, Debug)]
struct GlobalOptions {
    // normal comments are just comments
    /// Remote address of the SR-201 network device
    #[structopt(short, long, default_value = "192.168.1.100")]
    host: String,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Network Relay CLI Tool for SR-201 PCB")]
struct Options {
    #[structopt(flatten)]
    global: GlobalOptions,

    #[structopt(subcommand)]
    cmd: CliCommand,
}

//fn parse_hex(src: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
fn parse_relaylist(src: &str) -> Result<Vec<u8>, String> {
    let max_idx = 7;

    if src == "all" {
        return Ok(vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }

    if let Ok(idx) = src.parse() {
        if idx <= max_idx {
            return Ok(vec![idx]);
        }
    }

    let re = Regex::new(r"(\d+)?\.\.(=)?(\d+)?").unwrap();
    if let Some(cap) = re.captures(src) {
        let inclusive: bool = cap.get(2).is_some() || cap.get(3).is_none();

        let start: u8 = match cap.get(1) {
            Some(x) => x.as_str().parse::<u8>().unwrap_or(0),
            _ => 0,
        };

        let end: u8 = match cap.get(3) {
            Some(x) => x.as_str().parse::<u8>().unwrap_or(max_idx),
            _ => max_idx,
        };

        if start > end || end > max_idx {
            return Err(format!("Bad range for expression \"{}\"", src));
        }

        let v: Vec<u8>;
        if inclusive {
            v = (start..=end).map(u8::from).collect();
        } else {
            v = (start..end).map(u8::from).collect();
        }
        return Ok(v);
    }

    //Err(std::num::ParseIntError{kind: std::num::IntErrorKind::Empty})
    Err(format!("Failed to parse \"{}\"", src))
}

type FancyList = Vec<u8>;

#[derive(StructOpt, Debug)]
struct RelayList {
    /// Relay list to operate on, default means "all".
    #[structopt(default_value = "all", parse(try_from_str = parse_relaylist))]
    relay: FancyList,
}

#[derive(StructOpt, Debug)]
enum RelayCommand {
    /// Open specified relay(s)
    Open(RelayList),
    /// Close specified relay(s)
    Close(RelayList),
    /// Print closed status of specified relay(s) as true or false
    Status(RelayList),
}

#[derive(StructOpt, Debug)]
enum ConfigCommand {
    /// Set configuration values for given key
    Set { key: String, value: String },
    /// Get configuration values for given keys
    Get { key: Vec<String> },
}

#[derive(StructOpt, Debug)]
enum CliCommand {
    Config(ConfigCommand),
    Cmd(RelayCommand),
}

fn handle_config(
    config_cmd: &ConfigCommand,
    options: &GlobalOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("handle_config: {:?}", config_cmd);

    let mut sr201_config = relay_sr201::Config::connect(&options.host)?;
    sr201_config.debug(options.verbose > 0);

    match config_cmd {
        ConfigCommand::Get { key } => {
            let config = sr201_config.get_config()?;

            if key.len() == 0 {
                println!("{:?}", config);
            } else {
                let result: std::collections::HashMap<&str, Option<&String>> = key
                    .iter()
                    .map(|k| (k as &str, config.get(k as &str)))
                    .collect();

                println!("{:?}", result);
            }
        }

        ConfigCommand::Set { key, value } => {
            let config = sr201_config.set_config(key, value)?;
            println!("Result: {:?}", config);
        }
    }

    Ok(())
}

fn handle_cmd(
    cmd: &RelayCommand,
    options: &GlobalOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut relay = relay_sr201::Cmd::connect(&options.host)?;
    relay.debug(options.verbose > 0);

    match cmd {
        RelayCommand::Status(list) => {
            let status = relay.status()?;

            let result: std::collections::BTreeMap<u8, bool> = list
                .relay
                .iter()
                .map(|i| (*i, status[*i as usize]))
                .collect();

            println!("Closed: {:?}", result);
        }

        RelayCommand::Open(list) => {
            let results: Vec<bool> = list
                .relay
                .iter()
                .map(|&x| match relay.open(x) {
                    Ok(s) => s[x as usize] == false,
                    _ => false,
                })
                .collect();

            if results.iter().any(|&x| x == false) {
                return Err(Box::new(relay_sr201::make_generic_error(
                    "Channel not opened when expected",
                )));
            }
        }

        RelayCommand::Close(list) => {
            let results: Vec<bool> = list
                .relay
                .iter()
                .map(|&x| match relay.close(x) {
                    Ok(s) => s[x as usize] == true,
                    _ => false,
                })
                .collect();

            if results.iter().any(|&x| x == false) {
                return Err(Box::new(relay_sr201::make_generic_error(
                    "Channel not closed when expected",
                )));
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::from_args();

    match &options.cmd {
        CliCommand::Cmd(x) => handle_cmd(&x, &options.global)?,
        CliCommand::Config(x) => handle_config(&x, &options.global)?,
    };

    Ok(())
}
