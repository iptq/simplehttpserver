#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate serde;

use std::env;
use std::process;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

use rocket::Outcome;
use rocket::request::{Request, FromRequest};
use rocket::response::{self, Responder, NamedFile};
use rocket::config::{Config, Environment};
use rocket_contrib::Template;
use serde::ser::{Serialize, Serializer, SerializeSeq};

enum Response {
    Listing(Template),
    File(NamedFile),
}

impl<'r> Responder<'r> for Response {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        match self {
            Response::Listing(t) => Template::respond_to(t, req),
            Response::File(f) => NamedFile::respond_to(f, req),
        }
    }
}

enum ContextValue {
    String(String),
    FileList(Vec<String>),
}

impl Serialize for ContextValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            &ContextValue::String(ref s) => {
                return serializer.serialize_str(s.as_str());
            }
            &ContextValue::FileList(ref v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    seq.serialize_element(item)?;
                }
                return seq.end();
            }
        }
    }
}

struct Directory {
    name: String,
    path: PathBuf,
}

impl<'a, 'r> FromRequest<'a, 'r> for Directory {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> rocket::request::Outcome<Directory, ()> {
        let uri = request.uri().as_str();
        println!("uri: {}", uri);
        let mut target = env::current_dir().unwrap();
        if uri.len() > 1 {
            let push_uri = &uri[1..];
            target.push(push_uri);
        }
        return Outcome::Success(Directory {
            name: String::from(uri),
            path: target,
        });
    }
}

fn generic_handler(directory: Directory) -> Response {
    if fs::metadata(directory.path.clone()).unwrap().is_dir() {
        let mut context: HashMap<&str, ContextValue> = HashMap::new();
        context.insert("name", ContextValue::String(directory.name));
        let mut filelist: Vec<String> = vec![];
        let names = fs::read_dir(directory.path).unwrap();
        for name in names {
            filelist.push(String::from(name.unwrap().file_name().to_str().unwrap()));
        }
        filelist.sort();
        context.insert("files", ContextValue::FileList(filelist));
        return Response::Listing(Template::render("index", context));
    } else {
        return Response::File(NamedFile::open(directory.path).unwrap());
    }
}

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
