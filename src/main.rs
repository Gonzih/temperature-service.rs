#![feature(plugin, proc_macro)]
#![plugin(rocket_codegen)]

extern crate rocket_contrib;
extern crate rocket;
extern crate time;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use rocket_contrib::Template;
use std::process::Command;
use std::io::{BufWriter, BufReader, Error};
use std::io::prelude::*;
use std::fs::{OpenOptions, File};
use std::time::Duration;
use std::thread;

static LOG_FILE_PATH: &'static str = "/tmp/temperature.log";

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    payload: Vec<TemperatureData>,
    last: TemperatureData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct TemperatureData {
    humidity: f64,
    temperature: f64,
    nseconds: i64,
}

fn parse_data(input: String) -> TemperatureData {
    info!("Trying to parse input {}", input);
    let tnow = time::get_time();
    let now = tnow.sec * 1000;
    let values: Vec<f64> = input.trim().split(",").map(|x| x.parse().unwrap()).collect();

    TemperatureData {
        temperature: values[0],
        humidity: values[1],
        nseconds: now,
    }
}

fn run_command() -> TemperatureData {
    let output = Command::new("/home/gnzh/bin/temperature-test.sh")
        .output()
        .expect("Failed to read temperature");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    info!("status: {}", output.status);
    info!("stdout: {}", stdout);
    info!("stderr: {}", stderr);

    parse_data(stdout.to_string())
}

fn log_to_file(input: &TemperatureData) {
    let f = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(LOG_FILE_PATH)
        .expect("Unable to open log file");
    let mut f = BufWriter::new(f);
    let input_string = serde_json::to_string(&input).unwrap();
    f.write_fmt(format_args!("{}\n", input_string)).expect("Unable to write to log file");
}

fn start_logging_loop() -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let output = run_command();
            log_to_file(&output);
            let ten_minutes = Duration::from_secs(10 * 60);
            thread::sleep(ten_minutes);
        }
    })
}

fn open_file() -> BufReader<File> {
    let f = File::open(LOG_FILE_PATH).expect("Unable to open log file");
    BufReader::new(f)
}

fn read_file() -> Vec<TemperatureData> {
    let len = open_file().lines().count();
    let f = open_file();
    let num: usize = 24 * 6;
    let to_skip = if len > num { len - num } else { 0 };
    let result = f.lines()
        .skip(to_skip)
        .map(|line| {
            let line = line.unwrap();
            serde_json::from_str(&line).unwrap()
        })
        .collect();

    result
}

#[get("/")]
fn index() -> Template {
    let data = read_file();
    let last = read_file().last().unwrap().clone();
    let payload = Payload {
        payload: data,
        last: last,
    };

    Template::render("index", &payload)
}

#[get("/public/<folder>/<fname>")]
fn public(folder: &str, fname: &str) -> Option<File> {
    File::open(format!("public/{}/{}", folder, fname)).ok()
}

fn main() {
    let _ = start_logging_loop();
    rocket::ignite()
        .mount("/", routes![index])
        .mount("/", routes![public])
        .launch()
}
