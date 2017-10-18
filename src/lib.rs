#![deny(missing_docs)]

//! simplehttpserver is a re-implementation of Python's SimpleHTTPServer module in Rust. It's a
//! small utility that spawns a small HTTP server, serving static files from the current directory,
//! and is useful for serving HTML pages in a server context.
//!
//! By default, simplehttpserver runs on port 8000.

extern crate rocket;
extern crate rocket_contrib;
extern crate serde;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

use rocket::Outcome;
use rocket::request::{Request, FromRequest};
use rocket::response::{self, Responder, NamedFile};
use rocket_contrib::Template;
use serde::ser::{Serialize, Serializer, SerializeSeq};

/// A custom response enum, that serves either a template or a static file depending on the path
/// that's requested.
pub enum Response {
    /// A listing is handled by a template renderer.
    Listing(Template),
    /// A static file that can be served directly.
    File(NamedFile),
}

impl<'r> Responder<'r> for Response {
    /// Forwards the Response enumerated type to the specific response handler.
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        match self {
            Response::Listing(t) => Template::respond_to(t, req),
            Response::File(f) => NamedFile::respond_to(f, req),
        }
    }
}

/// A custom context value that makes it easier to construct a context to pass to handlebars-rust
/// for rendering the template.
enum ContextValue {
    /// A generic string type.
    String(String),
    /// A list of strings, used for generating the directory listing.
    FileNameList(Vec<String>),
}

impl Serialize for ContextValue {
    /// Converts a ContextValue into serde-compatible data forms, so it can be accessed by the
    /// template rendering engine.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            &ContextValue::String(ref s) => {
                return serializer.serialize_str(s.as_str());
            }
            &ContextValue::FileNameList(ref v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    seq.serialize_element(item)?;
                }
                return seq.end();
            }
        }
    }
}

/// A response guard that collects information from the Request object that cannot normally be
/// obtained through the view handler and passes it on to the actual view handler. This guard will
/// always return successfully.
pub struct Directory {
    /// The relative name of the current directory that's displayed on the page.
    name: String,
    /// A PathBuf representing the directory that was requested.
    path: PathBuf,
}

impl<'a, 'r> FromRequest<'a, 'r> for Directory {
    #[doc(hidden)]
    type Error = ();
    /// Generates a Directory response guard from a Request object.
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

/// A handler for both the root path and any other path. This is necessary because the Rocket route
/// handler for /<path> will not match /, so two handlers are necessary.
pub fn generic_handler(directory: Directory) -> Response {
    if fs::metadata(directory.path.clone()).unwrap().is_dir() {
        let mut context: HashMap<&str, ContextValue> = HashMap::new();
        context.insert("name", ContextValue::String(directory.name));
        let mut filelist: Vec<String> = vec![];
        let names = fs::read_dir(directory.path).unwrap();
        for name in names {
            filelist.push(String::from(name.unwrap().file_name().to_str().unwrap()));
        }
        filelist.sort();
        context.insert("files", ContextValue::FileNameList(filelist));
        return Response::Listing(Template::render("index", context));
    } else {
        return Response::File(NamedFile::open(directory.path).unwrap());
    }
}
