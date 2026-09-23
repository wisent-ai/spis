use super::*;

impl<'a> StrictParser<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        StrictParser {
            text: text.as_bytes(),
            pos: 0,
            depth: 0,
        }
    }

    pub(crate) fn err(&self, what: &str) -> String {
        format!("{what} at byte {}", self.pos)
    }

    pub(crate) fn skip_ws(&mut self) {
        while self.pos < self.text.len()
            && matches!(self.text[self.pos], b' ' | b'\t' | b'\n' | b'\r')
        {
            self.pos += 1;
        }
    }

    pub(crate) fn peek(&mut self) -> Option<u8> {
        self.skip_ws();
        self.text.get(self.pos).copied()
    }

    pub(crate) fn expect(&mut self, byte: u8, what: &str) -> Result<(), String> {
        match self.peek() {
            Some(b) if b == byte => {
                self.pos += 1;
                Ok(())
            }
            _ => Err(self.err(what)),
        }
    }

    pub(crate) fn parse_value(&mut self) -> Result<Value, String> {
        self.depth += 1;
        if self.depth > 200 {
            return Err("JSON nesting deeper than 200 levels".to_string());
        }
        let value = match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => self.parse_string().map(Value::String),
            Some(b't') => self.parse_literal("true", Value::Bool(true)),
            Some(b'f') => self.parse_literal("false", Value::Bool(false)),
            Some(b'n') => self.parse_literal("null", Value::Null),
            Some(_) => self.parse_number(),
            None => Err(self.err("unexpected end of input")),
        };
        self.depth -= 1;
        value
    }

    pub(crate) fn parse_literal(&mut self, word: &str, value: Value) -> Result<Value, String> {
        if self.text[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Ok(value)
        } else {
            Err(self.err("invalid literal"))
        }
    }

    pub(crate) fn parse_number(&mut self) -> Result<Value, String> {
        let start = self.pos;
        while self.pos < self.text.len()
            && matches!(
                self.text[self.pos],
                b'-' | b'+' | b'.' | b'0'..=b'9' | b'e' | b'E'
            )
        {
            self.pos += 1;
        }
        let token = std::str::from_utf8(&self.text[start..self.pos])
            .map_err(|_| self.err("invalid number"))?;
        if token.is_empty() {
            return Err(self.err("invalid value"));
        }
        if let Ok(i) = token.parse::<i64>() {
            return Ok(Value::Number(i.into()));
        }
        token
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .ok_or_else(|| self.err("invalid number"))
    }

    pub(crate) fn parse_string(&mut self) -> Result<String, String> {
        self.expect(b'"', "expected string")?;
        let mut out = String::new();
        loop {
            let Some(&byte) = self.text.get(self.pos) else {
                return Err(self.err("unterminated string"));
            };
            match byte {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    let esc = *self
                        .text
                        .get(self.pos)
                        .ok_or_else(|| self.err("unterminated escape"))?;
                    self.pos += 1;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let cp = self.parse_hex4()?;
                            if (0xD800..0xDC00).contains(&cp) {
                                // High surrogate: require a following \uXXXX low surrogate.
                                if self.text.get(self.pos) == Some(&b'\\')
                                    && self.text.get(self.pos + 1) == Some(&b'u')
                                {
                                    self.pos += 2;
                                    let low = self.parse_hex4()?;
                                    if !(0xDC00..0xE000).contains(&low) {
                                        return Err(self.err("invalid low surrogate"));
                                    }
                                    let combined = 0x10000 + ((cp - 0xD800) << 10) + (low - 0xDC00);
                                    out.push(
                                        char::from_u32(combined)
                                            .ok_or_else(|| self.err("invalid surrogate pair"))?,
                                    );
                                } else {
                                    return Err(self.err("lone high surrogate"));
                                }
                            } else if (0xDC00..0xE000).contains(&cp) {
                                return Err(self.err("lone low surrogate"));
                            } else {
                                out.push(
                                    char::from_u32(cp)
                                        .ok_or_else(|| self.err("invalid \\u escape"))?,
                                );
                            }
                        }
                        _ => return Err(self.err("invalid escape")),
                    }
                }
                0x00..=0x1F => return Err(self.err("control character in string")),
                _ => {
                    // Copy one UTF-8 encoded scalar.
                    let remaining = &self.text[self.pos..];
                    let s = std::str::from_utf8(remaining)
                        .map_err(|_| self.err("invalid UTF-8 in string"))?;
                    let ch = s.chars().next().ok_or_else(|| self.err("empty string"))?;
                    out.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
        }
    }

    pub(crate) fn parse_hex4(&mut self) -> Result<u32, String> {
        let slice = self
            .text
            .get(self.pos..self.pos + 4)
            .ok_or_else(|| self.err("truncated \\u escape"))?;
        let hex = std::str::from_utf8(slice).map_err(|_| self.err("bad \\u escape"))?;
        let cp = u32::from_str_radix(hex, 16).map_err(|_| self.err("bad \\u escape"))?;
        self.pos += 4;
        Ok(cp)
    }

    pub(crate) fn parse_object(&mut self) -> Result<Value, String> {
        self.expect(b'{', "expected object")?;
        let mut map = Map::new();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Value::Object(map));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.expect(b':', "expected ':' after key")?;
            let value = self.parse_value()?;
            if map.contains_key(&key) {
                return Err(format!("duplicate key {key:?}"));
            }
            map.insert(key, value);
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Value::Object(map));
                }
                _ => return Err(self.err("expected ',' or '}'")),
            }
        }
    }

    pub(crate) fn parse_array(&mut self) -> Result<Value, String> {
        self.expect(b'[', "expected array")?;
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Value::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Value::Array(items));
                }
                _ => return Err(self.err("expected ',' or ']'")),
            }
        }
    }
}

pub(crate) fn strict_json(text: &str, what: &str) -> Result<Value> {
    let mut parser = StrictParser::new(text);
    let value = parser
        .parse_value()
        .map_err(|detail| anyhow!("{what}: not readable JSON: {detail}"))?;
    parser.skip_ws();
    if parser.pos != text.len() {
        bail!("{what}: trailing characters after JSON document");
    }
    Ok(value)
}

// ---------------------------------------------------------------------------
// Row helpers

pub(crate) fn pick<'a>(row: &'a Value, names: &[&str]) -> Option<&'a Value> {
    for name in names {
        if let Some(value) = row.get(*name) {
            if !value.is_null() {
                return Some(value);
            }
        }
    }
    None
}

pub(crate) fn action_rows(payload: &Value, what: &str) -> Result<Vec<Value>> {
    let rows_from = |list: &Value| -> Option<Vec<Value>> {
        list.as_array().map(|items| {
            items
                .iter()
                .filter(|item| item.is_object())
                .cloned()
                .collect()
        })
    };
    if let Some(rows) = rows_from(payload) {
        return Ok(rows);
    }
    if let Some(map) = payload.as_object() {
        for key in ["actions", "jobs", "captures", "items", "results"] {
            if let Some(value) = map.get(key) {
                if let Some(rows) = rows_from(value) {
                    return Ok(rows);
                }
            }
        }
    }
    bail!("{what}: no per-action list in the response")
}

pub(crate) fn action_id(row: &Value) -> Option<String> {
    pick(row, &["action_id", "actionId", "id", "job_id", "jobId"]).map(|v| v.to_string())
}
