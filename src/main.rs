#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate simplehttpserver;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;

use std::env;
use std::process;

use rocket::config::{Config, Environment};
use rocket_contrib::Template;
use simplehttpserver::{Directory, Response, generic_handler};

#[get("/")]
fn index(directory: Directory) -> Response {
    return generic_handler(directory);
}

#[get("/<_path>")]
fn other(directory: Directory, _path: String) -> Response {
    return generic_handler(directory);
}

fn main() {
    let mut port: u16 = 8000;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let port_arg: String = args[1].clone();
        match port_arg.parse::<u16>() {
            Ok(n) => port = n,
            Err(_) => {
                eprintln!("Invalid port: {}", port_arg);
                process::exit(1);
            }
        }
    }
    let config = Config::build(Environment::Staging)
        .address("0.0.0.0")
        .port(port)
        .unwrap();
    rocket::custom(config, true)
        .attach(Template::fairing())
        .mount("/", routes![index])
        .mount("/", routes![other])
        .launch();
}
