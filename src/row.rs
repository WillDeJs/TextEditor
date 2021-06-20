use crate::document::SearchDirection;
use crate::filetype::FileType;
use crate::highlighting;
use crate::terminal::Color;

#[derive(Default, Debug)]
pub struct Row {
    string: String,
    highlighting: Vec<highlighting::Type>,
    shading: Vec<highlighting::Type>,
}

impl Row {
    pub fn new(string: String) -> Self {
        let highlighting = Vec::new();
        let shading = Vec::new();
        Self {
            string,
            highlighting,
            shading,
        }
    }

    pub fn render(&self, start: usize, end: usize) -> String {
        let mut result = String::new();

        for (index, character) in self
            .string
            .chars()
            .skip(start)
            .take(end - start)
            .enumerate()
        {
            let highlight_type = self
                .highlighting
                .get(index + start)
                .unwrap_or(&highlighting::Type::None);
            let mut colored_char = format!(
                "{}{}{}",
                crossterm::SetFg(highlight_type.to_color()),
                character,
                crossterm::SetFg(Color::Reset)
            );
            let shading_type = self
                .shading
                .get(index + start)
                .unwrap_or(&highlighting::Type::None);
            if shading_type != &highlighting::Type::None {
                colored_char = format!(
                    "{}{}{}",
                    crossterm::SetBg(shading_type.to_color()),
                    colored_char,
                    crossterm::SetBg(Color::Reset)
                );
            }
            result.push_str(&colored_char[..]);
        }
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
            shading: Vec::new(),
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
    pub fn highlight(&mut self, filetype: &FileType, search_word: &Option<String>) {
        let tokens = highlighting::Token::tokenize(filetype, &self.string);
        let mut highlighting = Vec::new();
        let mut shading = Vec::new();
        for token in tokens {
            for _ in token.value.chars() {
                highlighting.push(token.token_type.clone());
                shading.push(highlighting::Type::None);
            }
        }
        let mut search_index = 0;
        if let Some(word) = search_word {
            // println!("word={}", word);
            // std::thread::sleep_ms(500);
            while let Some(index) =
                self.find(&word.to_string(), search_index, SearchDirection::Forward)
            {
                // search_index = search_index.saturating_sub(word.len()); // returns the last index inside of the word, we subract to get the first
                for i in index.saturating_sub(word.len())..index {
                    shading[i] = highlighting::Type::Match;
                }
                search_index = index;
            }
        }
        self.highlighting = highlighting;
        self.shading = shading;
    }
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            highlighting: Vec::new(),
            shading: Vec::new(),
        }
    }
}
