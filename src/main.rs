#[macro_use]
extern crate quick_error;
extern crate regex;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate iron;
#[macro_use]
extern crate router;
extern crate urlencoded;
extern crate params;
extern crate hyper;
extern crate select;
extern crate rustc_serialize;

use iron::prelude::*;
use iron::status;
use router::Router;
use regex::Regex;

mod disk_cache;
mod vegvesen_api;

fn query_hander(request: &mut Request) -> IronResult<Response> {
    let ref path = iexpect!(request.extensions.get::<Router>());
    let query = iexpect!(path.find("query"));

    let re = Regex::new(r"\A[a-zA-Z]{2}\d{4,5}\z").unwrap();
    if !re.is_match(query) {
        return Ok(Response::with((status::BadRequest, "Invalid number format\n")));
    }

    match disk_cache::get_cached_or_compute(query, vegvesen_api::get_registration_number) {
        Ok(value) => Ok(Response::with((status::Ok, value))),
        Err(disk_cache::CacheError::Callback(description)) => {
            Ok(Response::with((status::BadRequest, description)))
        }
        Err(_) => Ok(Response::with((status::BadRequest))),
    }
}

fn main() {
    env_logger::init().unwrap();

    let port = "0.0.0.0:3000";
    let router = router!(index: get "/"       => query_hander,
                         query: get "/:query" => query_hander);

    warn!("Server started at {}", port);

    Iron::new(router).http(port).unwrap();
}
