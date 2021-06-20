test.

/ characters
                else if c == '\'' && !found_string  && previoius_char != '\\'{
                    if found_char {
                        buffer.push(c);
                        tokens.push(Token::from(highlighting_options, buffer));
                        buffer = String::new();
                        found_char = false;
                    } else {
                        found_char = true;
                        buffer.push(c);
                    }
                }