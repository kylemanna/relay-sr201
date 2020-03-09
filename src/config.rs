use std::collections::HashMap;
use std::io::prelude::*;
use std::net::TcpStream;

use crate::error::{make_generic, Result};

#[derive(Debug)]
pub struct Config {
    sock: TcpStream,
    debug: bool,
    check_resp: bool,
}

static CONFIG_KEYS: [&str; 10] = [
    "ipv4",
    "netmask",
    "gateway",
    "",
    "power_persist",
    "version",
    "serial",
    "dns",
    "cloud_server",
    "cloud_enabled",
];

impl Config {
    pub fn connect(addr: &str) -> Result<Config> {
        let dst_cmd = format!("{}:5111", addr);
        Ok(Config {
            sock: TcpStream::connect(dst_cmd)?,
            debug: false,
            check_resp: true,
        })
    }

    pub fn debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    fn cmd(&mut self, req: &str) -> Result<String> {
        if self.debug {
            println!("> {}", req);
        }

        self.sock.write(req.as_bytes())?;

        let mut buf = [0; 128];
        let n_bytes = self.sock.read(&mut buf)?;
        if n_bytes == buf.len() {
            return Err(make_generic("Buffer too small"));
        }

        let resp = std::str::from_utf8(&buf[0..n_bytes])?;

        if self.debug {
            println!("< {}", resp);
        }

        // Check that first character is a ">"
        let mut resp_chars = resp.chars();
        if resp_chars.next().unwrap_or_default() != '>' {
            return Err(make_generic("Bad format for first char"));
        }
        // Check that last character is a ";"
        if resp_chars.last().unwrap_or_default() != ';' {
            return Err(make_generic("Bad format for last char"));
        }
        let resp = &resp[1..(resp.len() - 1)];

        Ok(String::from(resp))
    }

    pub fn get_config(&mut self) -> Result<HashMap<&str, String>> {
        let resp = self.cmd("#12222;")?;

        let values: Vec<&str> = resp.split(",").collect();

        if values.len() != CONFIG_KEYS.len() {
            return Err(make_generic("Unexpected number of values"));
        }

        let hash: HashMap<&str, String> = CONFIG_KEYS
            .iter()
            .cloned()
            .zip(values)
            .filter_map(|(k, v)| {
                if k.len() > 0 {
                    Some((k, String::from(v)))
                } else {
                    None
                }
            })
            .collect();

        Ok(hash)
    }
}
