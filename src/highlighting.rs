use crate::filetype::FileType;
use crate::filetype::HighlightingOptions;
use crate::terminal::Color;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    None,
    Number,
    Match,
    String,
    Character,
    Comment,
    MultilineComment,
    PrimaryKeywords,
    SecondaryKeywords,
    WhiteSpace,
    Punctuation,
}

impl Type {
    pub fn to_color(&self) -> Color {
        match self {
            Type::Number => {
                return Color::Rgb {
                    r: 255,
                    g: 0,
                    b: 145,
                }
            }
            Type::Match => return Color::DarkYellow,
            Type::String | Type::Character => return Color::DarkGreen,
            Type::Comment => return Color::DarkGrey,
            Type::PrimaryKeywords => return Color::DarkCyan,
            Type::SecondaryKeywords => return Color::DarkRed,
            Type::Punctuation => return Color::Magenta,
            _ => Color::White,
        }
    }
}

pub struct Token {
    pub value: String,
    pub token_type: Type,
}

impl Token {
    ///
    /// Parse the given string as a Token and assign it a type
    /// To be used in highlighting
    ///
    pub fn from(highlighting_options: &HighlightingOptions, string: String) -> Self {
        let primary_keywords = highlighting_options.primary_keywords();
        let secundary_keywords = highlighting_options.secondary_keywords();
        let new_string = string.trim().to_string();

        if new_string.is_empty() {
            return Self {
                value: string,
                token_type: Type::WhiteSpace,
            };
        }
        // Numbers
        else if string.trim().to_string().parse::<usize>().is_ok()
            && highlighting_options.numbers()
        {
            return Self {
                value: string,
                token_type: Type::Number,
            };
        }
        // String
        else if new_string.ends_with("\"")
            && new_string.starts_with("\"")
            && highlighting_options.strings()
        {
            return Self {
                value: string,
                token_type: Type::String,
            };
        }
        // Character
        else if string.ends_with("'")
            && string.starts_with("'")
            && highlighting_options.characters()
        {
            return Self {
                value: string,
                token_type: Type::Character,
            };
        }
        // Comment
        else if new_string.starts_with("//") && highlighting_options.comments() {
            return Self {
                value: string,
                token_type: Type::Comment,
            };
        }
        // primary keywords
        else if primary_keywords.contains(&new_string) {
            return Self {
                value: string,
                token_type: Type::PrimaryKeywords,
            };
        }
        // Secundary keywords
        else if secundary_keywords.contains(&new_string) {
            return Self {
                value: string,
                token_type: Type::SecondaryKeywords,
            };
        }
        // Punctuation
        else if new_string.len() == 1 {
            let character = new_string.chars().nth(0).unwrap_or('\0');
            if character.is_ascii_punctuation() && highlighting_options.punctuation() {
                return Self {
                    value: string,
                    token_type: Type::Punctuation,
                };
            }
        }
        // Default case
        return Self {
            value: string,
            token_type: Type::None,
        };
    }

    ///
    /// Extract a set of Tokens from the given string based on the given hightlingOptions (include lists of primary and secundary words)
    ///     Returns a vector with all the tokens
    ///     This separation includes all strings, all charactesr, all white space, all punctuation as tokens.
    ///
    pub fn tokenize(filetype: &FileType, string: &String) -> Vec<Token> {
        let mut tokens = Vec::<Token>::new();
        let mut buffer = String::new();
        let mut previous_char = '\0';
        let mut found_string = false;
        let mut found_char = false;
        let highlighting_options = filetype.highlighting_options();
        for (i, c) in string.chars().enumerate() {
            if c.is_ascii_punctuation() || c.is_ascii_whitespace() {
                // this is a comment
                if previous_char == '/' && c == '/' {
                    buffer = string[(i - 1)..].to_string();
                    tokens.push(Token::from(highlighting_options, buffer));
                    break;
                }
                // parse strings
                if c == '\"' {
                    if found_string && previous_char != '\\' {
                        found_string = false;
                        buffer.push(c);
                        tokens.push(Token::from(highlighting_options, buffer));
                        buffer = String::new();
                    } else {
                        found_string = true;
                        buffer.push(c);
                    }
                }
                // characters
                else if c == '\'' && !found_string {
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
                // any other punctuation should be considered end of a token and start of a new.
                else if c != '/' {
                    tokens.push(Token::from(highlighting_options, buffer));
                    buffer = String::new();
                    buffer.push(c);
                    tokens.push(Token::from(highlighting_options, buffer));
                    buffer = String::new();
                }
            }
            // End of line
            else if c == '\n' || c == '\r' || i == string.len() - 1 {
                buffer.push(c);
                tokens.push(Token::from(highlighting_options, buffer));
                buffer = String::new();
            } else {
                buffer.push(c);
            }
            previous_char = c;
        }
        tokens
    }

    /// Convert token to string, this also adds the proper coloring to the token
    /// For matches, the foreground color is set.
    pub fn to_string(&self) -> String {
        if self.token_type == Type::Match {
            format!(
                "{}{}{}",
                crossterm::SetBg(self.token_type.to_color()),
                self.value,
                crossterm::SetBg(Color::Reset)
            )
        } else {
            format!(
                "{}{}{}",
                crossterm::SetFg(self.token_type.to_color()),
                self.value,
                crossterm::SetFg(Color::Reset)
            )
        }
    }
}
