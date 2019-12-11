extern crate minjson;
extern crate clap;

use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::io::BufReader;

use std::io;

use clap::{App, Arg};

fn main() -> () {
	let app =App::new("minjson")
				.version("0.2")
				.about("JSON tools")
				.author("tinaxd")
				.arg(Arg::with_name("mode")
					.short("m")
					.long("mode")
					.required(true)
					.takes_value(true)
					.possible_values(&["minify", "pretty", "inspect"])
					.value_name("MODE")
					)
				.arg(Arg::with_name("out")
					.long("out")
					.takes_value(true)
					.value_name("FILEPATH")
					)
				.arg(Arg::with_name("in")
					.long("in")
					.takes_value(true)
					.value_name("FILEPATH"))
				.get_matches();

	let mode = app.value_of("mode").unwrap();

	let mut strbuf = String::new();
	if let Some(inpath) = app.value_of("in") {
		match File::open(inpath) {
			Err(e) => {
				eprintln!("{}", e);
				return;
			},

			Ok(infile) => {
				let mut bf = BufReader::new(infile);
				bf.read_to_string(&mut strbuf).expect("Unable to read json");
			}
		}
	} else {
		io::stdin().read_to_string(&mut strbuf).expect("Unable to read json");
	}
    
    let res: String;
    if mode == "minify" {
    	res = minjson::minimize_json(&strbuf);
    } else if mode == "pretty" {
    	res = minjson::pretty_json(&strbuf, &minjson::PrettySetting{indent_width: 2});
    } else if mode == "inspect" {
    	match minjson::build_json_graph(&strbuf) {
    		Ok(g) => res = format!("{:#?}", g),
    		Err(e) => res = format!("{}\n", e.to_string()),
    	}
    } else {
    	unreachable!()
    }

    if let Some(outpath) = app.value_of("out") {
		let outfile = File::create(outpath);
		if outfile.is_err() {
			eprintln!("File writing err {}", outfile.unwrap_err().to_string());
			return;
		} else {
    		let mut outfile = outfile.unwrap();
    		if let Err(e) = write!(outfile, "{}", res) {
    			eprintln!("{}", e);
    			return;
    		}
		}
    } else {
    	// stdout output
    	print!("{}", res);
    }
}
