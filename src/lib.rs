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