use select::document::Document;
use select::predicate::{Class, Name, Attr};
use select::node::Node;

use std::collections::HashMap;
use rustc_serialize::json;

use std::io;

use hyper;

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Section {
    title: String,
    table_body: HashMap<String, String>,
}

fn table_to_section(table: Node) -> Section {
    let title = match table.find(Class("modul-overskrift")).first() {
        Some(title) => title.text(),
        None => "No title".to_string(),
    };

    let table_body = table.find(Name("th"))
        .iter()
        .zip(table.find(Name("td")).iter())
        .map(|(th, td)| (th.text(), td.text()))
        .collect::<HashMap<_, _>>();

    Section {
        title: title,
        table_body: table_body,
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum VegvesenetError {
        Hyper(err: hyper::Error) { from() }
        Io(err: io::Error) { from() }
        JSON(err: json::EncoderError) { from() }
        NoSuchRegistrationNumber {}
        RateLimitation {}
    }
}

pub fn get_registration_number(registration_number: &str) -> Result<String, VegvesenetError> {
    let api_url = "http://www.vegvesen.\
                   no/Kjoretoy/Kjop+og+salg/Kjøretøyopplysninger?registreringsnummer=";
    let request_url = format!("{}{}", api_url, registration_number);

    let client = hyper::Client::new();
    let response = try!(client.get(&request_url).send());
    let dom = try!(Document::from_read(response));

    //if dom.find(Attr("id", "readspeak")).find(Name("p")).len() == 3 {
    if 3 == 4 {
        return Err(VegvesenetError::NoSuchRegistrationNumber);
    }

    let tables = dom.find(Class("kjoretoy-table"))
        .iter()
        .map(table_to_section)
        .collect::<Vec<_>>();

    match tables.len() {
        0 => Err(VegvesenetError::RateLimitation),
        _ => Ok(try!(json::encode(&tables))),
    }
}
