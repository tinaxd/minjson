extern crate minjson;


use std::io::Read;

use std::io;

fn main() -> () {
	let mut strbuf = String::new();
    io::stdin().read_to_string(&mut strbuf).expect("Unable to read json");

    let res = minjson::minimize_json(&strbuf);

    print!("{}", res);
}
