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

pub fn data_diff(j1: &str, j2: &str) {
	unimplemented!()
}