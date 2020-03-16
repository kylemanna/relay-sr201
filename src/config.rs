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

#[derive(Debug)]
struct ConfigOpt {
    name: &'static str,
    id_set: char,
}

static CONFIG_KEYS: [ConfigOpt; 10] = [
    ConfigOpt {
        name: "ipv4",
        id_set: '2',
    },
    ConfigOpt {
        name: "netmask",
        id_set: '3',
    },
    ConfigOpt {
        name: "gateway",
        id_set: '4',
    },
    ConfigOpt {
        name: "",
        id_set: '5',
    },
    ConfigOpt {
        name: "power_persist",
        id_set: '6',
    },
    ConfigOpt {
        name: "version",
        id_set: '7',
    },
    ConfigOpt {
        name: "serial",
        id_set: ' ',
    },
    ConfigOpt {
        name: "dns",
        id_set: '8',
    },
    ConfigOpt {
        name: "cloud_server",
        id_set: '9',
    },
    ConfigOpt {
        name: "cloud_enabled",
        id_set: 'A',
    },
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
            .zip(values)
            .filter_map(|(k, v)| {
                if k.name.len() > 0 {
                    Some((k.name, String::from(v)))
                } else {
                    None
                }
            })
            .collect();

        Ok(hash)
    }

    pub fn set_config(&mut self, key: &str, value: &str) -> Result<()> {
        let config_opt = CONFIG_KEYS
            .iter()
            .find(|&c| key == c.name)
            .ok_or(make_generic("Key not found"))?;

        let resp = self.cmd(&format!("#{}2222,{};", config_opt.id_set, value))?;

        match &resp as &str {
            "OK" => Ok(()),
            "ERR" => Err(make_generic("Device replied with error")),
            _ => Err(make_generic(&format!("Unknown response: {}", resp))),
        }
    }
}
