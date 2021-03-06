extern crate getopts;
extern crate hyper;
extern crate time;
extern crate regex;

use std::io::Read;

use getopts::Options;
use std::env;
use regex::Regex;

use std::time::Duration;
use std::thread;

use hyper::Client;
use hyper::header::Connection;
use hyper::status::StatusCode;
use hyper::error;

fn test_site(url: &str, sleep_duration: &Duration) -> error::Result<()> {
    let client = Client::new();
    let mut body = Vec::new();

    loop {
        body.clear();
        let start = time::now();

        let mut res = try!(client.get(url)
            .header(Connection::close())
            .send());

        match res.status {
            StatusCode::Ok => {
                res.read_to_end(&mut body).unwrap();

                let end = time::now();
                let diff = end - start;
                println!("{}: {} in {}.{}s. Read {} bytes", 
                        &start.ctime(), res.status, diff.num_seconds(), 
                        diff.num_milliseconds() % 1000, body.len());

                thread::sleep(sleep_duration.clone());
            }
            other => {
                println!("{}: Request failed. Status: {}", &start.ctime(), &other);
                return Ok(());
            }
        }
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] URL", program);
    print!("{}", opts.usage(&brief));
}

fn run_client(url: &str, duration: &Duration) {
    println!("Watching site {}", url);
    test_site(&url, duration).unwrap();
}

fn main() {
    let args : Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("d", "duration", "set the wait duration between probes postfixed with [smh] for units", "DURATION");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => { panic!(f.to_string()) }
    };

    let duration = if let Some(d) = matches.opt_str("d") {
        let re = Regex::new(r"(?P<duration>\d+)(?P<units>[hms])").unwrap();

        let caps = if let Some(caps) = re.captures(&d) {
            caps
        } else {
            print_usage(&program, opts);
            return;
        };

        match (caps.name("duration").map(|d| d.parse().unwrap()),
               caps.name("units")) {
            (None,_) => {
                print_usage(&program, opts);
                return;
            }
            (Some(d), Some("h")) => Duration::from_secs(d*60*60),
            (Some(d), Some("m")) => Duration::from_secs(d*60),
            (Some(d), Some("s")) => Duration::from_secs(d),
            _ => {
                print_usage(&program, opts);
                return;
            }
        }
    } else {
        Duration::from_secs(30*60)
    };

    let url = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    run_client(&url, &duration);
}
