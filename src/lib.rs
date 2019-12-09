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
	Object, Array, String,
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