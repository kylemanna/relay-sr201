use structopt::StructOpt;

use relay_sr201;

#[derive(StructOpt, Debug)]
#[structopt(name = "Network Relay CLI Tool for SR-201 PCB")]
struct Options {
    // normal comments are just comments
    /// Remote address of the SR-201 network device
    #[structopt(short, long, default_value = "192.168.1.100")]
    host: String,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Configuration mode
    #[structopt(long)]
    config: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::from_args();

    if options.config {
        let mut sr201_config = relay_sr201::Config::connect(&options.host)?;
        sr201_config.debug(options.verbose > 0);
        let config = sr201_config.get_config()?;
        println!("Config: {:?}", config);
        return Ok(());
    }

    let mut relay = relay_sr201::Cmd::connect(&options.host)?;

    relay.debug(options.verbose > 0);

    let status = relay.status()?;
    println!("Initial status: {:?}", status);

    let status = relay.close(1)?;
    println!("Status: {:?}", status);

    let status = relay.open(1)?;
    println!("Status: {:?}", status);

    let status = relay.close_then_open(1, 2)?;
    println!("Status: {:?}", status);

    let status = relay.close_then_open_quick(2)?;
    println!("Status: {:?}", status);

    println!("Polling...");

    for _ in 1..10 {
        let status = relay.status()?;
        println!("Status: {:?}", status);

        let delay = std::time::Duration::from_millis(1000);
        std::thread::sleep(delay);
    }

    Ok(())
}
