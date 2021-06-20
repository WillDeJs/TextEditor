use crate::document::SearchDirection;
use crate::filetype::FileType;
use crate::highlighting;
use crate::terminal::Color;
use std::cmp;
#[derive(Default, Debug)]
pub struct Row {
    string: String,
    highlighting: Vec<highlighting::Type>,
}


impl Row {
    pub fn new(string: String) -> Self {
        let highlighting = Vec::new();
        Self {
            string,
            highlighting,
        }
    }

    // pub fn render(&self, start: usize, end: usize) -> String {
    //     let mut result = String::new();
    //     let end = cmp::min(end, self.string.len());
    //     let start = cmp::min(start, end);
    //     for (index, character) in self.string.chars().skip(start).take(end - start).enumerate() {
    //        let highlight_type = self.highlighting.get(index).unwrap_or(&highlighting::Type::None);
    //        let colored_char = format!("{}{}{}", crossterm::SetFg(highlight_type.to_color()), character, crossterm::SetFg(Color::Reset));
    //        result.push_str(&colored_char[..]);
    //     }
    //     result
    // }
    pub fn render(&self, filetype: &FileType, start: usize, end: usize) -> String {
        let mut result = String::new();
        let tokens = highlighting::Token::tokenize(filetype, &self.string);
        for token in tokens {
            result.push_str(token.to_string().as_str());
        }
        // for (index, character) in self.string.chars().skip(start).take(end - start).enumerate() {
        //     let highlight_type = self.highlighting.get(index).unwrap_or(&highlighting::Type::None);
        //     let colored_char = format!("{}{}{}", crossterm::SetFg(highlight_type.to_color()), character, crossterm::SetFg(Color::Reset));
        //     result.push_str(&colored_char[..]);
        // }
        result
    }

    pub fn len(&self) -> usize {
        self.string.len()
    }

    pub fn delete(&mut self, at: usize) {
        let mut string = String::new();
        for (index, character) in self.string.chars().enumerate() {
            if index == at {
                continue;
            }
            string.push(character);
        }
        self.string = string;
    }
    pub fn insert(&mut self, c: char, at: usize) {
        if self.string.len() == at {
            self.string.push(c);
            return;
        }

        let mut string = String::new();
        for (i, character) in self.string.chars().enumerate() {
            if at == i {
                string.push(c);
            }
            string.push(character);
        }
        self.string = string;
    }
    pub fn split(&mut self, pos: usize) -> Self {
        let mut this_string = String::new();
        let mut new_string = String::new();
        for (i, c) in self.string.chars().enumerate() {
            if i >= pos {
                new_string.push(c);
            } else {
                this_string.push(c);
            }
        }
        self.string = this_string;
        Self {
            string: new_string,
            highlighting: Vec::new(),
        }
    }
    pub fn append(&mut self, other_row: &Row) {
        self.string.push_str(&other_row.string[..]);
    }

    pub fn text(&self) -> &String {
        &self.string
    }
    pub fn find(&self, query: &String, start: usize, direction: SearchDirection) -> Option<usize> {
        match direction {
            SearchDirection::Forward => {
                if start >= self.string.len() {
                    return None;
                }
                let new_string = self.string[start..].to_string();
                if let Some(location) = new_string.find(query) {
                    return Some(location + start + query.len());
                }
            }
            SearchDirection::Backward => {
                let start = start.saturating_sub(query.len());
                let new_string = self.string[..start].to_string();
                if let Some(location) = new_string.rfind(query) {
                    return Some(location + query.len());
                }
            }
        }
        None
    }

    //     pub fn highlight(&mut self, word: Option<&str>) {
    //         let mut prev_separator = true;
    //         let mut previous_highlight : highlighting::Type = highlighting::Type::None;
    //         let mut search_index = 0;
    //         let mut highlighting = Vec::new();
    //         let mut matches = Vec::<usize>::new();
    //         if let Some(word) = word {
    //             while let Some(index) = self.find(&word.to_string(), search_index, SearchDirection::Forward) {
    //                 matches.push(index);
    //                 search_index = search_index.saturating_sub(word.len());  // returns the last index inside of the word, we subract to get the first
    //             }
    //         }
    //         for (i, c) in self.string.chars().enumerate() {
    //             if let Some(word) = word {
    //                 if matches.contains(&i) {
    //                     for _ in  i.. i + word.len() {
    //                         highlighting.push(highlighting::Type::Match);
    //                     }
    //                     continue;
    //                 }

    //             }
    //             if c.is_ascii_digit() && (prev_separator || previous_highlight == highlighting::Type::Number) {
    //                 highlighting.push(highlighting::Type::Number);
    //                 previous_highlight = highlighting::Type::Number;
    //             } else {
    //                 highlighting.push(highlighting::Type::None);
    //                 previous_highlight = highlighting::Type::None;
    //             }
    //             prev_separator = c.is_ascii_punctuation() || c.is_ascii_whitespace();
    //         }
    //        self.highlighting = highlighting;
    //     }
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            highlighting: Vec::new(),
        }
    }
}
