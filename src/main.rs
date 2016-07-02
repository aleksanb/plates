#[macro_use] extern crate quick_error;

extern crate rustc_serialize;
extern crate select;
extern crate hyper;
extern crate regex;

#[macro_use] extern crate iron;
#[macro_use] extern crate router;
extern crate urlencoded;
extern crate params;

use iron::prelude::*;
use iron::status;
use router::Router;

use select::document::Document;
use select::predicate::{Class, Name};
use select::node::Node;
use rustc_serialize::json;

use std::collections::HashMap;
use std::io::prelude::*;

use regex::Regex;

mod disk_cache;

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Section {
    title: String,
    table_body: HashMap<String, String>,
}

fn table_to_section(table: Node) -> Section {
    let title = table.find(Class("modul-overskrift")).first().unwrap().text();
    let table_body = table.find(Name("th")).iter()
        .zip(table.find(Name("td")).iter())
        .map(|(th, td)| (th.text(), td.text()))
        .collect::<HashMap<_, _>>();

    Section {
        title: title,
        table_body: table_body
    }
}

fn get_registration_number(registration_number: &str) -> Result<String, String> {
    let api_url = "http://www.vegvesen.no/Kjoretoy/Kjop+og+salg/Kjøretøyopplysninger?registreringsnummer=";
    let request_url = format!("{}{}", api_url, registration_number);

    let client = hyper::Client::new();
    let mut response = client.get(&request_url).send().unwrap();
    let mut body = String::new();
    response.read_to_string(&mut body).unwrap();

    let dom = Document::from_str(&body);

    let tables = dom.find(Class("kjoretoy-table")).iter()
        .map(table_to_section)
        .collect::<Vec<_>>();

    if tables.len() > 0 {
        Ok(json::encode(&tables).unwrap())
    } else {
        Ok("No such registration number or API Timeout. Please try again later.\n".to_string())
    }
}

fn query_hander(request: &mut Request) -> IronResult<Response> {
    let ref path = iexpect!(request.extensions.get::<Router>());
    let query = iexpect!(path.find("query"));

    let re = Regex::new(r"\A[a-zA-Z]{2}\d{4,5}\z").unwrap();
    if !re.is_match(query) {
        return Ok(Response::with((status::BadRequest, "Invalid number format\n")));
    }

    let result = itry!(disk_cache::get_cached_or_compute(query, get_registration_number));
    Ok(Response::with((status::Ok, result)))
}

fn main() {
    let router = router!(get "/" => query_hander,
                         get "/:query" => query_hander);
    Iron::new(router).http("0.0.0.0:3000").unwrap();
}
