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
					.possible_values(&["minify", "pretty", "inspect", "diff"])
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
				.arg(Arg::with_name("in2")
					.long("in2")
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

	let mut input: Box<dyn Read> = {
		if let Some(inpath) = app.value_of("in") {
			match File::open(inpath) {
				Ok(i) => Box::new(std::io::BufReader::new(i)),
				Err(e) => {
					eprintln!("{}", e.to_string());
					return;
				}
			}
		} else {
			Box::new(std::io::BufReader::new(std::io::stdin()))
		}
	};

	let mut output: Box<dyn Write> = {
		if let Some(outpath) = app.value_of("out") {
			let outfile = File::create(outpath);
			match outfile {
				Ok(o) => Box::new(std::io::BufWriter::new(o)),
				Err(e) => {
					eprintln!("File writing err {}", e.to_string());
					return;
				}
			}
		} else {
			Box::new(std::io::BufWriter::new(std::io::stdout()))
		}
	};
    
    if mode == "minify" {
		let mut string = String::new();
		match input.read_to_string(&mut string) {
			Err(e) => {
				eprintln!("Failed to read json file: {}", e.to_string());
				return;
			},
			Ok(_) => {
				for ch in minjson::JsonMinimizer::new_from_str(&string) {
					let mut buf = [0; 4];
					ch.encode_utf8(&mut buf);
					output.write_all(&buf).unwrap();
				}
			}
		}
    } else if mode == "pretty" {
		let mut string = String::new();
		match input.read_to_string(&mut string) {
			Err(e) => {
				eprintln!("Failed to read json file: {}", e.to_string());
				return;
			},
			Ok(_) => {
				for ch in minjson::JsonFormatter::new_from_str(&string, minjson::PrettySetting{indent_width: 2}) {
					let mut buf = [0; 4];
					ch.encode_utf8(&mut buf);
					output.write_all(&buf).unwrap();
				}
			}
		}
    } else if mode == "inspect" {
    	match minjson::build_json_graph(&strbuf) {
    		Ok(g) => write!(output, "{}", format!("{:#?}", g)).unwrap(),
    		Err(e) => write!(output, "{}", format!("{}\n", e.to_string())).unwrap(),
    	};
    } else if mode == "diff" {
    	if let Some(in2path) = app.value_of("in2") {
    		let mut strbuf2 = String::new();
    		match File::open(in2path) {
				Err(e) => {
					eprintln!("{}", e);
					return;
				},

				Ok(infile) => {
					let mut bf = BufReader::new(infile);
					bf.read_to_string(&mut strbuf2).expect("Unable to read json");
				}
			}
			let diffs = minjson::structure_diff(&strbuf, &strbuf2, minjson::DiffSetting::default());
			match diffs {
				Ok(ds) => {
					let mut buf = String::new();
					for d in &ds {
						buf.push_str(&pretty_diff(d));
						buf.push('\n');
					}
					write!(output, "{}", buf).unwrap();
				},
				Err(e) => { write!(output, "{}", e.to_string()).unwrap(); },
			}
    	} else {
    		eprintln!("Must specify in2 option");
    		return;
    	}
    } else {
    	unreachable!()
    }
}

fn pretty_diff(diff: &minjson::JsonDiff) -> String {
	use minjson::DiffType::*;

	let head = match diff.diff_type {
		Added => "+++",
		Deleted => "---",
		Modified => "***",
	};

	let question = String::from("?");
	let from = diff.from_desc.as_ref().unwrap_or(&question);
	let to = diff.to_desc.as_ref().unwrap_or(&question);
	let path = &diff.base_path;

	format!("{} {} -> {} in {}", head, from, to, path)
}
