extern crate minjson;

use std::env;
use std::io::Read;

use std::io;

fn main() -> () {
	let args: Vec<String> = env::args().collect();
	if args.len() != 2 || (args[1] != "minify" && args[1] != "pretty") {
		eprintln!("Wrong number of arguments");
		eprintln!("Usage: minjson <operation>");
		eprintln!("<operation> := minify | pretty");
		return;
	}

	let mut strbuf = String::new();
    io::stdin().read_to_string(&mut strbuf).expect("Unable to read json");

    let res: String;
    if args[1] == "minify" {
    	res = minjson::minimize_json(&strbuf);
    } else {
    	res = minjson::pretty_json(&strbuf, &minjson::PrettySetting{indent_width: 2});
    }

    print!("{}", res);
}
