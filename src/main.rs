#![feature(plugin, proc_macro)]
#![plugin(rocket_codegen)]

extern crate rocket_contrib;
extern crate rocket;
extern crate time;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

use rocket_contrib::Template;
use std::process::Command;
use std::io::{BufWriter, BufReader};
use std::io::prelude::*;
use std::fs::{OpenOptions, File};
use std::time::Duration;
use std::{thread};

static LOG_FILE_PATH: &'static str = "/tmp/temperature.log";

#[derive(Serialize)]
struct TemplateContext {
    contents: String
}

fn run_command() -> String {
    let output = Command::new("/home/gnzh/bin/temperature-test.sh")
                         .output()
                         .expect("Failed to read temperature");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    info!("status: {}", output.status);
    info!("stdout: {}", stdout);
    info!("stderr: {}", stderr);

    stdout.to_string()
}

fn log_to_file(input: &String) {
    let f = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(LOG_FILE_PATH)
                        .expect("Unable to open log file");
    let mut f = BufWriter::new(f);
    let now = time::get_time().sec;

    f.write_fmt(format_args!("{},{}", now, input)).expect("Unable to write to log file");
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

fn read_file() -> String {
    let f = File::open(LOG_FILE_PATH).expect("Unable to open log file");
    let mut f = BufReader::new(f);
    let mut output = String::new();

    f.read_to_string(&mut output).expect("Unable to read log file");

    output
}

#[get("/")]
fn index() -> Template {
    let context = TemplateContext {
        contents: read_file()
    };

    Template::render("index", &context)
}

fn main() {
    let _ = start_logging_loop();
    rocket::ignite().mount("/", routes![index]).launch()
}
