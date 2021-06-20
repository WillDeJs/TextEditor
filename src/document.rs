use crate::editor::Position;
use crate::filetype::FileType;
use crate::row::Row;
use std::clone::Clone;
use std::fs;
use std::io::Write;
use std::usize;

#[derive(Default, Debug)]
pub struct Document {
    pub rows: Vec<Row>,
    pub filetype: FileType,
    pub filename: Option<String>,
    pub search_string: Option<String>,
    is_dirty: bool,
}
#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let mut rows = Vec::<Row>::new();
        let is_dirty = false;
        let filetype = FileType::from(&filename);
        let contents = fs::read_to_string(filename)?;
        let search_string = Option::None;
        contents.lines().for_each(|line| {
            let mut row = Row::from(line);
            row.highlight(&filetype, &search_string);
            rows.push(row);
        });

        let filename = Some(filename.to_string());
        Ok(Self {
            rows,
            is_dirty,
            search_string,
            filename,
            filetype,
        })
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }
    pub fn row_mut(&mut self, index: usize) -> Option<&mut Row> {
        self.rows.get_mut(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    pub fn delete(&mut self, pos: &Position) {
        let doc_len = self.len();
        if pos.y >= doc_len {
            return;
        }
        if pos.x == self.rows[pos.y].len() && pos.y + 1 < doc_len {
            let next_row = self.rows.remove(pos.y + 1);
            let this_row = &mut self.rows[pos.y];
            this_row.append(&next_row);
        } else {
            let row = &mut self.rows[pos.y];
            row.delete(pos.x);
        }
        self.is_dirty = true;
    }
    pub fn insert(&mut self, c: char, pos: &Position) {
        let doc_len = self.len();
        if pos.y > doc_len {
            return;
        }
        if c == '\n' {
            if pos.x == 0 {
                self.rows.insert(pos.y, Row::default());
            } else {
                let row = &mut self.rows[pos.y];
                row.highlight(&self.filetype, &self.search_string);
                let mut new_row = row.split(pos.x);
                new_row.highlight(&self.filetype, &self.search_string);
                self.rows.insert(pos.y + 1, new_row);
            }
        } else if pos.y == self.rows.len() {
            let mut row = Row::default();
            row.insert(c, 0);
            row.highlight(&self.filetype, &self.search_string);
            self.rows.push(row);
        } else {
            let row = &mut self.rows[pos.y];
            row.highlight(&self.filetype, &self.search_string);
            row.insert(c, pos.x);
        }
        self.is_dirty = true;
    }

    pub fn save(&mut self) -> std::result::Result<(), std::io::Error> {
        if self.is_dirty() {
            if let Some(filename) = &self.filename {
                let filepath = std::path::Path::new(&filename[..]);
                let mut file = if filepath.exists() {
                    fs::File::create(&filepath)?
                } else {
                    fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .open(&filepath)?
                };
                for row in &self.rows {
                    file.write_all(row.text().as_bytes())?;
                    file.write_all(b"\n")?;
                }
                self.is_dirty = false;
            }
        }
        Ok(())
    }

    pub fn find(
        &mut self,
        query: &String,
        at: Position,
        direction: SearchDirection,
    ) -> Option<Position> {
        let mut pos = at;
        let start;
        let end;
        if direction == SearchDirection::Forward {
            end = self.rows.len();
            start = pos.y;
            for y in start..end {
                let row = &mut self.rows[y];
                // moving to new line, restart x position
                if y > pos.y {
                    pos.x = 0;
                }
                if let Some(x) = row.find(query, pos.x, direction) {
                    self.search_string = Some(query.clone());
                    row.highlight(&self.filetype, &Option::Some(query.clone()));
                    return Some(Position { x, y });
                }
                // if let Some(row) = self.row_mut(y) {
                //     // moving to new line, restart x position
                //     if y > pos.y {
                //         pos.x = 0;
                //     }
                //     if let Some(x) = row.find(query, pos.x, direction) {
                //         // row.highlight(&self.filetype, &Option::Some(query.clone()));
                //         self.search_string = Some(query.clone());
                //         return Some(Position { x, y });
                //     }
                // }
            }
        } else {
            end = 0;
            start = pos.y;
            for y in (end..=start).rev() {
                let row = &mut self.rows[y];
                if y < pos.y {
                    pos.x = row.len().saturating_sub(1);
                }
                if let Some(x) = row.find(query, pos.x, direction) {
                    self.search_string = Some(query.clone());
                    row.highlight(&self.filetype, &Option::Some(query.clone()));
                    return Some(Position { x, y });
                }
            }
        }

        None
    }

    pub fn hightlight(&mut self) {
        for i in 0..self.rows.len() {
            &mut self.rows[i].highlight(&self.filetype, &self.search_string);
        }
    }
}
