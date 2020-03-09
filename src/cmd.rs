use std::io::prelude::*;
use std::net::TcpStream;

use crate::error::{make_generic, Result};

#[derive(Debug)]
pub struct Cmd {
    sock: TcpStream,
    debug: bool,
    check_resp: bool,
}

fn char_list_to_array(resp: &str) -> Result<[bool; 8]> {
    let mut status = [false; 8];

    if resp.len() != status.len() {
        return Err(make_generic("Unexpected char length"));
    }

    resp.chars()
        .enumerate()
        .for_each(|(i, c)| status[i] = c == '1');
    Ok(status)
}

impl Cmd {
    pub fn connect(addr: &str) -> Result<Cmd> {
        let dst_cmd = format!("{}:6722", addr);
        Ok(Cmd {
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

        Ok(String::from(resp))
    }

    pub fn status(&mut self) -> Result<[bool; 8]> {
        let resp = self.cmd("00")?;
        char_list_to_array(&resp)
    }

    fn _close(&mut self, ch: u8, delay: i32) -> Result<[bool; 8]> {
        let mut cmd = String::from(format!("1{}", ch + 1));
        if delay < 0 {
            // Close then open in about 500 ms
            cmd += "*";
        } else if delay > 0 {
            // Close then open in delay seconds
            cmd = format!("{}:{}", cmd, delay);
        }

        let resp = self.cmd(&cmd)?;
        let closed = char_list_to_array(&resp)?;

        if self.check_resp && closed[ch as usize] == false {
            return Err(make_generic("Channel not closed when expected"));
        }

        Ok(closed)
    }

    pub fn close(&mut self, ch: u8) -> Result<[bool; 8]> {
        self._close(ch, 0)
    }

    pub fn close_then_open(&mut self, ch: u8, delay: u16) -> Result<[bool; 8]> {
        self._close(ch, delay as i32)
    }

    pub fn close_then_open_quick(&mut self, ch: u8) -> Result<[bool; 8]> {
        self._close(ch, -1)
    }

    pub fn open(&mut self, ch: u8) -> Result<[bool; 8]> {
        let cmd = format!("2{}", ch + 1);
        let resp = self.cmd(&cmd)?;
        char_list_to_array(&resp)
    }
}
