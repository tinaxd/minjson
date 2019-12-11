use std::collections::HashMap;

pub fn minimize_json(json: &str) -> String {
	let mut result = String::new();
	let mut in_str_literal = false;
	let mut skip_char = false;
	for ch in json.chars() {
		if skip_char {
			skip_char = false;
			result.push(ch);
		}
		else if in_str_literal {
			if ch == '\\' {
				skip_char = true;
				result.push(ch);
			} else if ch == '\"' {
				in_str_literal = false;
				result.push(ch);
			} else {
				result.push(ch);
			}
		} else if ch.is_whitespace() {
			// Do nothing
		} else {
			if ch == '\"' {
				in_str_literal = true;
				result.push(ch);
			} else {
				result.push(ch);
			}
		}
	}
	result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsonContext {
	Object, Array, String, Number
}

pub fn pretty_json(json: &str, setting: &PrettySetting) -> String {
	let compressed = minimize_json(json);
	let dirty = &compressed;

	let mut curr_indentlv = 0;
	let mut result = String::new();
	let mut ctx: Vec<JsonContext> = vec![JsonContext::Object]; // Used as stack
	let mut skip_char = false;
	let mut last_comma = false;

	for ch in dirty.chars() {
		if skip_char {
			skip_char = false;
			result.push(ch);
			last_comma = false;
			continue;
		}

		if ctx[0] == JsonContext::String {
			if ch == '\\' {
				skip_char = true;
			} else if ch == '\"' {
				ctx.pop();
			}
			result.push(ch);
			last_comma = false;
			continue;
		}

		if ch == '\"' {
			result.push(ch);
			ctx.push(JsonContext::String);
			last_comma = false;
			continue;
		}

		if ch == ',' {
			result.push(ch);
			result.push('\n');
			for _ in 0..(curr_indentlv * setting.indent_width) {
				result.push(' ');
			}
			last_comma = true;
			continue;
		}

		if ch == '[' || ch == '{' {
			result.push(ch);
			result.push('\n');
			curr_indentlv += 1;
			for _ in 0..(curr_indentlv * setting.indent_width) {
				result.push(' ');
			}
			ctx.push(match ch {
				'[' => JsonContext::Array,
				'{' => JsonContext::Object,
				_ => unreachable!(),
			});
			last_comma = false;
			continue;
		}

		if ch == ']' || ch == '}' {
			curr_indentlv -= 1;
			if !last_comma {
				result.push('\n');
				for _ in 0..(curr_indentlv * setting.indent_width) {
					result.push(' ');
				}
			}
			result.push(ch);
			ctx.pop();
			continue;
		}

		if ch == ':' {
			result.push(ch);
			result.push(' ');
			continue;
		}

		result.push(ch);
	}
	result.push('\n');

	result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrettySetting {
	pub indent_width: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonElement {
	JsonNumber(f64),
	JsonString(String),
	JsonArray(Vec<JsonElement>),
	JsonObject(HashMap<String, JsonElement>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct JsonLexer<'a> {
	chars: &'a str,
	ptr: usize,
}

impl<'a> JsonLexer<'a> {
	fn new(json: &'a str) -> JsonLexer<'a> {
		JsonLexer {chars: json, ptr: 0}
	}

	fn ptr(&mut self) -> usize {
		self.ptr
	}

	fn next(&mut self) -> Option<char> {
		if self.ptr < self.chars.len() {
			let ch = self.chars.chars().nth(self.ptr);
			self.ptr += 1;
			ch
		} else {
			None
		}
	}

	fn back(&mut self) {
		self.ptr -= 1;
	}

	fn slice(&self, start: usize, end: Option<usize>) -> Option<&'a str> {
		let end = end.unwrap_or(self.chars.len());
		if start >= 0 && end <= self.chars.len() && start <= end {
			Some(&self.chars[start..end])
		} else {
			None
		}
	}

	fn reset(&mut self) {
		self.ptr = 0;
	}
}

fn parse_json(json: &mut JsonLexer) -> Result<JsonElement, String> {
	let mut json_elem: Option<Result<JsonElement, String>> = None;

	while let Some(ch) = json.next() {
		if ch.is_whitespace() {
			continue;
		}

		//println!("{}", ch);
		if ch == '{' {
			//println!("OBJECT");
			json_elem = Some(parse_json_object(json));
			break;
		} else if ch == '"' {
			//println!("STRING");
			json_elem = Some(parse_json_string(json));
			break;
		} else if ch == '[' {
			//println!("ARRAY");
			json_elem = Some(parse_json_array(json));
			break;
		} else {
			//println!("NUMBER");
			// TODO Number parsing
			json.back();
			json_elem = Some(parse_json_number(json));
			break;
		}
	}

	match json_elem {
		Some(j) => j,
		None => Err("cannot parse JSON".to_string())
	}
}

fn parse_json_object(json: &mut JsonLexer) -> Result<JsonElement, String> {
	let mut pairs = HashMap::new();

	let mut res = None;
	'top: while let (key, value) = parse_json_object_pair(json)? {

		//println!("objKEY: {:?}, objVALUE: {:?}", &key, &value);
		pairs.insert(key, value);

		while let Some(ch) = json.next() {
			//println!("kv ch {}", ch);
			if ch.is_whitespace() {
				continue;
			} else if ch == ',' {
				break; // Next kv pair
			} else if ch == '}' {
				// End of JSON object
				let obj = JsonElement::JsonObject(pairs);
				res = Some(Ok(obj));
				break 'top;
			} else {
				res = Some(Err(format!("Expected ',' or '}}', got {}", ch)));
				break 'top;
			}
		}
	}
	match res {
		Some(s) => s,
		None => Err(String::from("Reached EOF while parsing JSON object")),
	}
}

fn parse_json_object_pair(json: &mut JsonLexer) -> Result<(String, JsonElement), String> {
	let _ctx = vec![JsonContext::Object];

	// Parse key (string)
	let mut res = None;
	while let Some(ch) = json.next() {
		if ch.is_whitespace() {
			continue;
		} else if ch == '"' {
			break;
		} else {
			res = Some(Err(String::from(format!("Expected '\"', got {}", ch))));
			break;
		}
	}
	if res.is_some() {
		return res.unwrap();
	}
	let key = parse_json_string(json)?;
	// expect colon
	let mut res = None;
	while let Some(ch) = json.next() {
		if ch.is_whitespace() {
			continue;
		} else if ch == ':' {
			break;
		} else {
			res = Some(Err(String::from(format!("Expected ':', got {}", ch))));
			break;
		}
	}
	if res.is_some() {
		return res.unwrap();
	}
	// Parse value (any json element)
	let value = parse_json(json)?;

	let key = match key {
		JsonElement::JsonString(s) => s,
		_ => unreachable!()
	};

	Ok((key, value))
}

fn parse_json_string(json: &mut JsonLexer) -> Result<JsonElement, String> {
	let mut skip = false;
	let mut buffer = String::new();

	let mut res = None;
	while let Some(ch) = json.next() {
		if ch == '"' && !skip {
			// End of JSON string
			res = Some(Ok(JsonElement::JsonString(buffer.clone())));
			break;
		} else if ch == '\\' && !skip {
			skip = true;
			continue;
		} else {
			buffer.push(ch);
		}
	};

	//println!("end string");
	//println!("{}", json.ptr());
	match res {
		Some(s) => {
			//println!("{}", buffer);
			s
		},
		None => Err(format!("Reached EOF while parsing JSON string. string: {}", buffer)),
	}
}

fn parse_json_number(json: &mut JsonLexer) -> Result<JsonElement, String> {
	// TODO: support for frac exp, more precise parsing

	let mut first_char = None;
	while let Some(ch) = json.next() {
		if ch.is_whitespace() {
			continue;
		} else {
			first_char = Some(ch);
			break;
		}
	}

	if first_char.is_none() {
		return Err(String::from("Reached EOF while parsing JSON number"));
	}
	let first_char = first_char.unwrap();

	let is_minus = first_char == '-';

	let mut num: f64 = if first_char.is_ascii_digit() {
		first_char.to_digit(10).unwrap() as f64
	} else {
		0.0
	};

	let mut fp = 0;
	while let Some(ch) = json.next() {
		if ch.is_whitespace() {
			continue;
		} else if ch.is_ascii_digit() { // 0 - 9
			if fp == 0 {
				num *= 10.0;
				num += ch.to_digit(10).unwrap() as f64;
			} else {
				num += (ch.to_digit(10).unwrap() as f64) / (10 ^ fp) as f64;
			}
		} else if ch == '.' {
			fp = 1;
		} else {
			// End of number
			json.back();
			break;
		}
	}

	if is_minus {
		num *= -1.0;
	}

	Ok(JsonElement::JsonNumber(num))
}

fn parse_json_array(json: &mut JsonLexer) -> Result<JsonElement, String> {
	let mut elems = Vec::new();

	// (elem ,)*
	let mut res = None;
	'top: loop {
		while let Some(ch) = json.next() {
			if ch.is_whitespace() {
				continue;
			} else if ch == ']' {
				break 'top;
			} else {
				json.back();
				break;
			}
		}

		let elem = parse_json(json)?;
		//println!("ARRAY ELM {:?}", &elem);
		elems.push(elem);

		while let Some(ch) = json.next() {
			if ch.is_whitespace() {
				continue;
			} else if ch == ',' {
				break;
			} else if ch == ']' {
				break 'top;
			} else {
				res = Some(Err(format!("Expected ',' or ']', got {}", ch)));
				break 'top;
			}
		}
	}

	match res {
		Some(r) => r,
		None => Ok(JsonElement::JsonArray(elems)),
	}
}

pub fn build_json_graph(json: &str) -> Result<JsonElement, String> {
	let mut lexer = JsonLexer::new(json);
	parse_json(&mut lexer)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DiffType {
	Added, Deleted, Modified
}

impl std::fmt::Display for DiffType {
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		use DiffType::*;
		let tok = match self {
			Added => "+++",
			Deleted => "---",
			Modified => "***", 
		};
		write!(formatter, "{}", tok)
	}
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JsonDiff {
	pub diff_type: DiffType,
	pub from_desc: Option<String>,
	pub to_desc: Option<String>,
	pub base_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiffSetting {
	pub floatDiffThreashold: f64,
}

impl Default for DiffSetting {
	fn default() -> DiffSetting {
		DiffSetting {
			floatDiffThreashold: 10e-6
		}
	}
}

pub fn structure_diff(base_json: &str, compared_json: &str, settings: DiffSetting) -> Result<Vec<JsonDiff>, String> {
	let base_g = build_json_graph(base_json);
	if base_g.is_err() {
		return Err(base_g.unwrap_err().to_string());
	}
	let base_g = base_g.unwrap();
	let compared_g = build_json_graph(compared_json);
	if compared_g.is_err() {
		return Err(compared_g.unwrap_err().to_string());
	}
	let compared_g = compared_g.unwrap();

	Ok(element_diff(&base_g, &compared_g, "", settings))
}

fn element_diff(base_el: &JsonElement, compared_el: &JsonElement, base_path: &str, settings: DiffSetting) -> Vec<JsonDiff> {
	use JsonElement::*;
	use DiffType::*;

	let path_sec = "::";

	// Helper function
	let make_json_type_diff = |a: &JsonElement, b: &JsonElement| -> Vec<JsonDiff> {
		vec!(JsonDiff {
			diff_type: Modified,
			from_desc: Some(format!("{:?}", a)),
			to_desc: Some(format!("{:?}", b)),
			base_path: base_path.to_string(),
		})
	};

	match base_el {
		JsonNumber(base_num) => {
			match compared_el {
				JsonNumber(compared_num) => {
					if (base_num - compared_num).abs() < settings.floatDiffThreashold {
						Vec::new()
					} else {
						vec!(JsonDiff {
							diff_type: Modified,
							from_desc: Some(format!("{}", base_num)),
							to_desc: Some(format!("{}", compared_num)),
							base_path: base_path.to_string(),
						})
					}
				},
				_ => {  // Number vs (other than Number)
					make_json_type_diff(base_el, compared_el)
				}
			}
		},

		JsonString(base_str) => {
			match compared_el {
				JsonString(compared_str) => {
					if base_str == compared_str {
						Vec::new()
					} else {
						vec!(JsonDiff {
							diff_type: Modified,
							from_desc: Some(base_str.to_string()),
							to_desc: Some(compared_str.to_string()),
							base_path: base_path.to_string(),
						})
					}
				},
				_ => {
					make_json_type_diff(base_el, compared_el)
				}
			}
		},

		JsonArray(base_vec) => {
			match compared_el {
				JsonArray(compared_vec) => {
					let mut diffs = Vec::new();
					if base_vec.len() == compared_vec.len() {
						for (b_elm, c_elm) in base_vec.iter().zip(compared_vec.iter()) {
							diffs.extend_from_slice(&element_diff(&b_elm, &c_elm, base_path, settings));
						}
					} else if base_vec.len() < compared_vec.len() {
						let last = base_vec.len();
						for (i, b_elm) in base_vec.iter().enumerate() {
							diffs.extend_from_slice(&element_diff(&b_elm, &compared_vec[i], base_path, settings));
						}
						for new_elm in &compared_vec[last..] {
							diffs.push(JsonDiff {
								diff_type: Added,
								from_desc: None,
								to_desc: Some(format!("{:?}", new_elm)),
								base_path: base_path.to_string(),
							})
						}
					} else {
						let last = compared_vec.len();
						for (i, c_elm) in compared_vec.iter().enumerate() {
							diffs.extend_from_slice(&element_diff(&base_vec[i], &c_elm, base_path, settings));
						}
						for del_elm in &base_vec[last..] {
							diffs.push(JsonDiff {
								diff_type: Deleted,
								from_desc: Some(format!("{:?}", del_elm)),
								to_desc: None,
								base_path: base_path.to_string(),
							})
						}
					}
					diffs
				},

				_ => {
					make_json_type_diff(base_el, compared_el)
				}
			}
		},

		JsonObject(base_obj) => {
			match compared_el {
				JsonObject(compared_obj) => {
					let mut diffs = Vec::new();

					for (bk, bv) in base_obj.iter() {
						let new_bp = format!("{}{}{}", base_path, path_sec, bk);
						match compared_obj.get(bk) {
							Some(cv) => {
								diffs.extend_from_slice(&element_diff(&bv, &cv, &new_bp, settings));
							},
							None => {
								diffs.push(JsonDiff {
									diff_type: Deleted,
									from_desc: Some(format!("{:?}", bv)),
									to_desc: None,
									base_path: new_bp,
								})
							}
						}
					}

					for (ck, cv) in compared_obj.iter() {
						let new_bp = format!("{}{}{}", base_path, path_sec, ck);
						match base_obj.get(ck) {
							Some(_) => {},
							None => {
								diffs.push(JsonDiff {
									diff_type: Added,
									from_desc: None,
									to_desc: Some(format!("{:?}", cv)),
									base_path: new_bp,
								})
							}
						}
					}

					diffs
				},

				_ => {
					make_json_type_diff(base_el, compared_el)
				}
			}
		}
	}
}