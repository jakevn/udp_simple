
#![feature(globs)]
extern crate iron;
use std::io::net::ip::Ipv4Addr;
use iron::prelude::*;
use iron::response::modifiers::{Status, Redirect};
use iron::{Url, status};

fn redirect(_: &mut Request) -> IronResult<Response> {
	let url = Url::parse("http://rust-lang.org").unwrap();
		Ok(Response::new()
			.set(Status(status::Ok))
			.set(Redirect(url)))
}

fn main() {
	println!("Attempting 3000")
	Iron::new(redirect).listen(Ipv4Addr(127, 0, 0, 1), 3000).unwrap();
	println!("On 3000");
}