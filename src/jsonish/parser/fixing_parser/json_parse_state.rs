use std::iter::Peekable;

use crate::jsonish::{value::Fixes, Value};
use anyhow::Result;

use super::json_collection::JsonCollection;

pub struct JsonParseState {
    pub collection_stack: Vec<(JsonCollection, Vec<Fixes>)>,

    // Technically we may find multiple values in a single string
    pub completed_values: Vec<(&'static str, Value, Vec<Fixes>)>,
}

impl JsonParseState {
    pub fn new() -> Self {
        JsonParseState {
            collection_stack: vec![],
            completed_values: vec![],
        }
    }

    pub fn complete_collection(&mut self) {
        let (collection, fixes) = match self.collection_stack.pop() {
            Some(collection) => collection,
            None => return,
        };

        let name = collection.name();

        let value: Value = match collection.into() {
            Some(value) => value,
            None => return,
        };

        if let Some((last, _fixes)) = self.collection_stack.last_mut() {
            match last {
                JsonCollection::Object(keys, values) => {
                    if keys.len() == values.len() {
                        match value {
                            Value::String(s) => keys.push(s),
                            Value::AnyOf(_, s) => keys.push(s),
                            _ => keys.push(value.to_string()),
                        }
                    } else {
                        values.push(value);
                    }
                }
                JsonCollection::Array(values) => {
                    values.push(value);
                }
                _ => {
                    // TODO: this should never happen as we should only be pushing objects and arrays
                    panic!(
                        "Unexpected value: {:?} in collection stack: {:?}",
                        value, last
                    );
                }
            }
        } else {
            self.completed_values.push((name, value, fixes));
        }
    }

    fn consume(&mut self, token: char) -> Result<usize> {
        let Some((last, _)) = self.collection_stack.last_mut() else {
            return Err(anyhow::anyhow!(
                "No collection to consume token: {:?}",
                token
            ));
        };
        match last {
            JsonCollection::QuotedString(s)
            | JsonCollection::TripleQuotedString(s)
            | JsonCollection::BlockComment(s)
            | JsonCollection::SingleQuotedString(s)
            | JsonCollection::BacktickString(s)
            | JsonCollection::TripleBacktickString { content: s, .. }
            | JsonCollection::UnquotedString(s)
            | JsonCollection::TrailingComment(s) => {
                // println!("Consuming: {s} + {:?}", token);
                s.push(token);
            }
            JsonCollection::Object(_, _) | JsonCollection::Array(_) => {
                panic!("Unexpected token: {:?} in: {:?}", token, last);
            }
        }
        Ok(0)
    }

    fn is_string_complete(&self) -> bool {
        let Some((JsonCollection::UnquotedString(v), _)) = self.collection_stack.last() else {
            return false;
        };

        // Check if the token is a valid json character
        match v.as_str() {
            "true" | "false" | "null" => true,
            _ => {
                // Check if the token parses as a number
                if v.parse::<f64>().is_ok() {
                    return true;
                }
                false
            }
        }
    }

    fn should_close_unescaped_string(
        &mut self,
        mut next: Peekable<impl Iterator<Item = (usize, char)>>,
    ) -> Option<usize> {
        let pos = if self.collection_stack.len() >= 2 {
            self.collection_stack
                .get(self.collection_stack.len() - 2)
                .map(|(c, _)| match c {
                    JsonCollection::Object(keys, values) => {
                        if keys.len() == values.len() {
                            2
                        } else {
                            3
                        }
                    }
                    JsonCollection::Array(_) => 4,
                    _ => 1,
                })
                .unwrap()
        } else {
            0
        };
        match pos {
            0 => {
                // in nothing, so perhaps the first '{' or '[' is the start of a new object or array
                let mut counter = 0;
                for (idx, c) in next.by_ref() {
                    counter = idx;
                    match c {
                        // If at some point we find a valid json character, we'll close the string
                        '{' | '[' => return Some(idx),
                        x => {
                            let _ = self.consume(x);
                        }
                    }
                }
                Some(counter)
            }
            1 => None,
            2 => {
                // in object key
                let mut counter = 0;
                for (idx, c) in next.by_ref() {
                    counter = idx;
                    match c {
                        ':' => return Some(idx),
                        x => {
                            let _ = self.consume(x);
                        }
                    }
                }
                Some(counter)
            }
            3 => {
                // in object value
                let mut counter = 0;
                while let Some((idx, c)) = next.next() {
                    counter = idx;
                    match c {
                        ',' => {
                            // Check if we have just numeric values in the string so far.
                            let Some((JsonCollection::UnquotedString(current_value), _)) =
                                self.collection_stack.last()
                            else {
                                return Some(idx);
                            };

                            // current value could be a numeric looking things.
                            let is_numeric = current_value.trim().parse::<f64>().is_ok();
                            let is_bool = current_value.trim().eq_ignore_ascii_case("true")
                                || current_value.trim().eq_ignore_ascii_case("false");
                            let is_null = current_value.trim().eq_ignore_ascii_case("null");
                            let is_possible_value = is_numeric || is_bool || is_null;

                            if let Some((_, next_c)) = next.peek() {
                                match next_c {
                                    '\n' => {
                                        log::debug!("Closing due to: newline after comma");
                                        return Some(idx);
                                    }
                                    ' ' => {
                                        log::debug!("Testing for comment after space + comma");
                                        if is_possible_value {
                                            return Some(idx);
                                        }
                                        // If after the space we have "//" or "/*" or the beginning of a key, we'll close the string
                                        let mut buffer = ",".to_string();
                                        let mut anything_but_whitespace = false;
                                        while let Some((_, next_next_c)) = next.next() {
                                            anything_but_whitespace = anything_but_whitespace
                                                || !next_next_c.is_whitespace();
                                            buffer.push(next_next_c);
                                            match next_next_c {
                                                ' ' => {}
                                                '\n' => {
                                                    if anything_but_whitespace {
                                                    } else {
                                                        // Likely end of the key as the LLM generated a ", " token by mistake instead of a ","
                                                        // so drop the comma
                                                        log::debug!("Closing due to: newline after comma + space");
                                                        return Some(idx);
                                                    }
                                                }
                                                '/' => match next.peek() {
                                                    Some((_, '/')) => {
                                                        // This is likely a comment
                                                        return Some(idx);
                                                    }
                                                    Some((_, '*')) => {
                                                        // This is likely a comment
                                                        return Some(idx);
                                                    }
                                                    _ => {
                                                        // let _ = self.consume(c);
                                                    }
                                                },
                                                '"' => {
                                                    // This is likely a new key
                                                    log::debug!("Closing due to: new key after space + comma");
                                                    return Some(idx);
                                                }
                                                _x => {
                                                    break;
                                                }
                                            }
                                        }
                                        for c in buffer.chars() {
                                            let _ = self.consume(c);
                                        }
                                    }
                                    _ => {
                                        let _ = self.consume(c);
                                    }
                                }
                            } else {
                                // Don't include the comma
                                return Some(idx);
                            }
                        }
                        '}' => return Some(idx),
                        x => {
                            let _ = self.consume(x);
                        }
                    }
                }
                Some(counter)
            }
            4 => {
                // in array
                let mut counter = 0;
                for (idx, c) in next {
                    counter = idx;
                    match c {
                        ',' => return Some(idx),
                        ']' => return Some(idx),
                        x => {
                            let _ = self.consume(x);
                        }
                    }
                }
                counter += 1; // Indicate that we called next() one time after the final `Some`.
                Some(counter)
            }
            _ => unreachable!("Invalid position"),
        }
    }

    fn should_close_string(
        &mut self,
        mut next: Peekable<impl Iterator<Item = (usize, char)>>,
        closing_char: char,
    ) -> bool {
        let (has_some_object, in_object_key, in_object_value, in_array) =
            if self.collection_stack.len() >= 2 {
                self.collection_stack
                    .get(self.collection_stack.len() - 2)
                    .map(|(c, _)| match c {
                        JsonCollection::Object(keys, values) => {
                            if keys.len() == values.len() {
                                (true, false, false)
                            } else {
                                (false, true, true)
                            }
                        }
                        JsonCollection::Array(_) => (false, false, true),
                        _ => (false, false, false),
                    })
                    .map(|(a, b, c)| (true, a, b, c))
                    .unwrap()
            } else {
                (false, false, false, false)
            };

            let closing_char_count = if closing_char == '"' {
                // count the number of quotes in the string
                let (last, _) = self.collection_stack.last().unwrap();
                match last {
                    JsonCollection::QuotedString(s, ..) => {
                        let mut count = 0;
                        // Iterate with indices so we can look backwards
                        for (i, c) in s.char_indices() {
                            if c == '"' {
                                // Count consecutive backslashes immediately preceding this quote
                                let mut backslash_count = 0;
                                let mut j = i;
                                while j > 0 {
                                    j -= 1;
                                    if s.as_bytes()[j] == b'\\' {
                                        backslash_count += 1;
                                    } else {
                                        break;
                                    }
                                }
                                // Only count this quote if the number of backslashes is even
                                if backslash_count % 2 == 0 {
                                    count += 1;
                                }
                            }
                        }
                        count
                    }
                    _ => 0,
                }
            } else {
                0
            };

        if let Some((idx, next_char)) = next.peek() {
            let _idx = *idx;
            match next_char {
                ':' | '}' if in_object_key => {
                    // We're ready to close the key
                    log::debug!("Closing due to: key");
                    true
                }
                ',' if in_object_value || in_array => {
                    if closing_char_count % 2 == 0 {
                        // We're ready to close the value
                        log::debug!("Closing due to: value",);
                        true
                    } else {
                        // We're not ready to close the value
                        false
                    }
                }
                '}' if in_object_value => {
                    // We're ready to close the value
                    log::debug!("Closing due to: value",);
                    true
                }
                ']' if in_array => {
                    // We're ready to close the value
                    log::debug!("Closing due to: array");
                    true
                }
                ' ' | '\t' | '\n' => {
                    // look ahead and see if we can find a closing bracket or comma
                    while let Some((_, c)) = next.next() {
                        match c {
                            ' ' | '\t' | '\n' => {}
                            '}' if in_object_key || in_object_value => return true,
                            ':' if in_object_key => return true,
                            ',' if in_object_value => return true,
                            ',' | ']' if in_array => return true,
                            '/' => {
                                // Could be a comment
                                match next.peek() {
                                    Some((_, '/')) => {
                                        // We're ready to close the comment
                                        return true;
                                    }
                                    Some((_, '*')) => {
                                        // We're ready to close the comment
                                        return true;
                                    }
                                    _ => return false,
                                }
                            }
                            _ => return false,
                        }
                    }
                    // If we faile, terminate the string
                    true
                }
                x if closing_char == *x => {
                    // We'll close the string the next time around.
                    false
                }
                '{' | '"' | '\'' | '[' => {
                    if !has_some_object {
                        // We're in a string
                        true
                    } else {
                        false
                    }
                }
                _ => {
                    // Almost every other character should not close the string
                    false
                }
            }
        } else {
            true
        }
    }

    pub fn process_token(
        &mut self,
        token: char,
        mut next: Peekable<impl Iterator<Item = (usize, char)>>,
    ) -> Result<usize> {
        // println!("Processing: {:?}..{:?}", token, next.peek());
        match self.collection_stack.last() {
            Some((last, _)) => match last {
                JsonCollection::Object(_, _) => {
                    match token {
                        '}' => {
                            // We're ready to close the object
                            self.complete_collection();
                            Ok(0)
                        }
                        // We can safely ignore these tokens
                        ',' | ':' => Ok(0),
                        // look for a new key or value
                        _ => self.find_any_starting_value(token, next),
                    }
                }
                JsonCollection::Array(_) => {
                    // We could be expecting:
                    // - A value
                    // - a comma
                    // - a closing bracket
                    match token {
                        ']' => {
                            // We're ready to close the array
                            self.complete_collection();
                            Ok(0)
                        }
                        // Skip these tokens
                        ',' => Ok(0),
                        _ => self.find_any_starting_value(token, next),
                    }
                }
                JsonCollection::TripleQuotedString(_) => {
                    // We should be expecting:
                    if token == '"' {
                        // TODO: this logic is busted. peekable.peek() does not
                        // advance the iterator (this is easily verified with
                        // a unit test), but to fix this we need to do a bit of
                        // refactoring, so for now we'll live with it.
                        let is_triple_quoted = match next.peek() {
                            Some((_, '"')) => matches!(next.peek(), Some((_, '"')) | None),
                            None => true,
                            _ => false,
                        };

                        if is_triple_quoted {
                            self.complete_collection();
                            Ok(3)
                        } else {
                            self.consume(token)
                        }
                    } else {
                        self.consume(token)
                    }
                }
                JsonCollection::QuotedString(_) => {
                    // We could be expecting:
                    // - A closing quote
                    // - A character
                    match token {
                        '"' => {
                            // It's possible that the LLM messed up the escaping
                            // We'll try to fix it.
                            if self.should_close_string(next, '"') {
                                self.complete_collection();
                                Ok(0)
                            } else {
                                self.consume(token)
                            }
                        }
                        '\\' => {
                            // Capture escaped characters
                            match next.peek() {
                                Some((_, 'n')) => {
                                    self.consume('\n')?;
                                    Ok(1)
                                }
                                Some((_, 't')) => {
                                    self.consume('\t')?;
                                    Ok(1)
                                }
                                Some((_, 'r')) => {
                                    self.consume('\r')?;
                                    Ok(1)
                                }
                                Some((_, 'b')) => {
                                    self.consume('\x08')?;
                                    Ok(1)
                                }
                                Some((_, 'f')) => {
                                    self.consume('\x0C')?;
                                    Ok(1)
                                }
                                Some((_, '\\')) => {
                                    self.consume('\\')?;
                                    Ok(1)
                                }
                                Some((_, '"')) => {
                                    self.consume('"')?;
                                    Ok(1)
                                }
                                Some((_, 'u')) => {
                                    // We'll consume the 'u' and the next 4 characters
                                    let mut buffer = String::new();
                                    buffer.push(token);
                                    for _ in 0..4 {
                                        if let Some((_, c)) = next.next() {
                                            buffer.push(c);
                                        } else {
                                            break;
                                        }
                                    }
                                    for c in buffer.chars() {
                                        let _ = self.consume(c);
                                    }
                                    Ok(5)
                                }
                                _ => self.consume(token),
                            }
                        }
                        _ => self.consume(token),
                    }
                }
                JsonCollection::TripleBacktickString { .. } => {
                    // We could be expecting:
                    // - A closing backtick
                    // - A character
                    if token == '`' {
                        // TODO: this logic is busted. peekable.peek() does not
                        // advance the iterator (this is easily verified with
                        // a unit test), but to fix this we need to do a bit of
                        // refactoring, so for now we'll live with it.
                        let is_triple_quoted = match next.peek() {
                            Some((_, '`')) => matches!(next.peek(), Some((_, '`')) | None),
                            None => true,
                            _ => false,
                        };

                        if is_triple_quoted {
                            self.complete_collection();
                            Ok(3)
                        } else {
                            self.consume(token)
                        }
                    } else {
                        self.consume(token)
                    }
                }
                JsonCollection::BacktickString(_) => {
                    // We could be expecting:
                    // - A closing backtick
                    // - A character
                    match token {
                        '`' => {
                            if self.should_close_string(next, '`') {
                                self.complete_collection();
                                Ok(0)
                            } else {
                                self.consume(token)
                            }
                        }
                        _ => self.consume(token),
                    }
                }
                JsonCollection::SingleQuotedString(_) => {
                    // We could be expecting:
                    // - A closing quote
                    // - A character
                    // - A space
                    match token {
                        '\'' => {
                            // It's possible that the LLM messed up the escaping
                            // We'll try to fix it.
                            if self.should_close_string(next, '\'') {
                                self.complete_collection();
                                Ok(0)
                            } else {
                                self.consume(token)
                            }
                        }
                        _ => self.consume(token),
                    }
                }
                JsonCollection::UnquotedString(_) => {
                    // We could be expecting:
                    // - A terminating json character (comma, colon, bracket, space, newline)
                    // - A character
                    let res = self.consume(token);
                    if let Some(count) = self.should_close_unescaped_string(next) {
                        self.complete_collection();
                        Ok(count)
                    } else {
                        res
                    }
                }
                JsonCollection::TrailingComment(_) => {
                    // We could be expecting:
                    // - A newline
                    // - A character
                    match token {
                        '\n' => {
                            // We're ready to close the comment
                            self.complete_collection();
                            Ok(0)
                        }
                        _ => self.consume(token),
                    }
                }
                JsonCollection::BlockComment(_) => {
                    // We could be expecting:
                    // - A closing comment
                    // - A character
                    match token {
                        '*' => {
                            // We could be closing the comment
                            match next.peek() {
                                Some((_, '/')) => {
                                    // We're ready to close the comment
                                    self.complete_collection();
                                    Ok(1)
                                }
                                _ => Ok(0),
                            }
                        }
                        _ => self.consume(token),
                    }
                }
            },
            None => {
                // We could be expecting:
                // - A value
                // - Any leading whitespace
                let preview = next.peekable();
                self.find_any_starting_value(token, preview)
            }
        }
    }

    // Returns the number of increments to skip after processing the token
    fn find_any_starting_value(
        &mut self,
        token: char,
        mut next: Peekable<impl Iterator<Item = (usize, char)>>,
    ) -> Result<usize> {
        match token {
            '{' => {
                self.collection_stack
                    .push((JsonCollection::Object(vec![], vec![]), Default::default()));
            }
            '[' => {
                self.collection_stack
                    .push((JsonCollection::Array(vec![]), Default::default()));
            }
            '"' => {
                // Peek if next 2 characters are also quotes
                let is_triple_quoted = {
                    next.next_if(|&(_, c)| c == '"')
                        .and_then(|_| next.next_if(|&(_, c)| c == '"'))
                        .is_some()
                };

                if is_triple_quoted {
                    self.collection_stack.push((
                        JsonCollection::TripleQuotedString(String::new()),
                        Default::default(),
                    ));
                    return Ok(2);
                } else {
                    self.collection_stack.push((
                        JsonCollection::QuotedString(String::new()),
                        Default::default(),
                    ))
                }
            }
            '\'' => {
                self.collection_stack.push((
                    JsonCollection::SingleQuotedString(String::new()),
                    Default::default(),
                ));
            }
            '`' => {
                // Peek if next 2 characters are also quotes
                let is_triple_quoted = {
                    next.next_if(|&(_, c)| c == '`')
                        .and_then(|_| next.next_if(|&(_, c)| c == '`'))
                        .is_some()
                };

                if is_triple_quoted {
                    self.collection_stack.push((
                        JsonCollection::TripleBacktickString {
                            lang: None,
                            path: None,
                            content: String::new(),
                        },
                        Default::default(),
                    ));
                    return Ok(2);
                } else {
                    self.collection_stack.push((
                        JsonCollection::BacktickString(String::new()),
                        Default::default(),
                    ))
                }
            }
            '/' => {
                // Could be a comment
                match next.peek() {
                    Some((_, '/')) => {
                        self.collection_stack.push((
                            JsonCollection::TrailingComment(String::new()),
                            Default::default(),
                        ));
                        return Ok(1);
                    }
                    Some((_, '*')) => {
                        self.collection_stack.push((
                            JsonCollection::BlockComment(String::new()),
                            Default::default(),
                        ));
                        return Ok(1);
                    }
                    _ => {
                        // if we're in an object, this could be the beginning of a string
                        // say a path?
                        if matches!(
                            self.collection_stack.last(),
                            Some((JsonCollection::Object(_, _), _))
                        ) {
                            self.collection_stack.push((
                                JsonCollection::UnquotedString(token.into()),
                                Default::default(),
                            ));
                            return Ok(0);
                        }
                    }
                }
            }
            x if x.is_whitespace() => {}
            x => {
                self.collection_stack
                    .push((JsonCollection::UnquotedString(x.into()), Default::default()));
                if let Some(count) = self.should_close_unescaped_string(next) {
                    self.complete_collection();
                    return Ok(count);
                }
            }
        };

        Ok(0)
    }
}
