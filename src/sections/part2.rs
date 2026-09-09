struct Buffer {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
    scroll: usize,
}

impl Buffer {
    fn new(content: &str) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };
        Self { lines, cursor_row: 0, cursor_col: 0, scroll: 0 }
    }
    fn content(&self) -> String { self.lines.join("\n") }
    fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cursor_row];
        if self.cursor_col > line.len() { self.cursor_col = line.len(); }
        line.insert(self.cursor_col, c);
        self.cursor_col += c.len_utf8();
    }
    fn insert_newline(&mut self) {
        let rest = self.lines[self.cursor_row].split_off(self.cursor_col);
        self.cursor_row += 1;
        self.lines.insert(self.cursor_row, rest);
        self.cursor_col = 0;
    }
    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_row];
            let mut idx = self.cursor_col - 1;
            while idx > 0 && !line.is_char_boundary(idx) { idx -= 1; }
            line.remove(idx);
            self.cursor_col = idx;
        } else if self.cursor_row > 0 {
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current);
        }
    }
    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            let line = &self.lines[self.cursor_row];
            let mut idx = self.cursor_col - 1;
            while idx > 0 && !line.is_char_boundary(idx) { idx -= 1; }
            self.cursor_col = idx;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }
    fn move_right(&mut self) {
        let line = &self.lines[self.cursor_row];
        if self.cursor_col < line.len() {
            let mut idx = self.cursor_col + 1;
            while idx < line.len() && !line.is_char_boundary(idx) { idx += 1; }
            self.cursor_col = idx;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }
    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        }
    }
    fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        }
    }
    fn ensure_visible(&mut self, height: usize) {
        if height == 0 { return; }
        if self.cursor_row < self.scroll { self.scroll = self.cursor_row; }
        else if self.cursor_row >= self.scroll + height { self.scroll = self.cursor_row - height + 1; }
    }
    fn scroll_by(&mut self, delta: i32, height: usize) {
        if delta < 0 { self.scroll = self.scroll.saturating_sub((-delta) as usize); }
        else {
            let max_scroll = self.lines.len().saturating_sub(height);
            self.scroll = (self.scroll + delta as usize).min(max_scroll);
        }
    }
}
