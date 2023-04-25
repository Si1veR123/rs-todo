use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskCategory {
    pub name: String,
    pub child: Vec<TaskItem>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub name: String,
    pub done: bool
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TaskItem {
    TaskCategory(TaskCategory),
    Task(Task)
}

impl TaskItem {
    // definitely not the most efficient way to do these, but simple and works

    pub fn all_text_lines(&self, line_buffer: &mut Vec<String>, indentation: usize) {
        let mut this_line_string = "    ".repeat(indentation);
        
        match self {
            TaskItem::Task(t) => {
                this_line_string.push_str(t.name.as_str());
                this_line_string.push(' ');
                this_line_string.push( if t.done { '☑' } else { '☐' } );
                line_buffer.push(this_line_string);
            },
            TaskItem::TaskCategory(c) => {
                this_line_string.push_str(c.name.as_str());
                line_buffer.push(this_line_string);

                for child in &c.child {
                    child.all_text_lines(line_buffer, indentation + 1);
                }
            }
        }
    }

    pub fn item_at_line_number(&mut self, line_number: usize) -> Option<&mut TaskItem> {
        self.item_search(0, line_number)
    }

    pub fn parent_of_line_number(&mut self, line_number: usize) -> Option<(&mut TaskItem, usize)> {
        self.parent_search(0, line_number).map(|line_no| {
            (self.item_search(0, line_no.0).unwrap(), line_no.1)
        })
    }

    fn parent_search(&self, current_line_number: usize, find_line_number: usize) -> Option<(usize, usize)> {
        // (parent line number, child index in parent's list)
        match self {
            TaskItem::TaskCategory(c) => {
                let mut lines_skipped = 1;
                for (i, child) in c.child.iter().enumerate() {
                    if lines_skipped + current_line_number == find_line_number {
                        return Some((current_line_number, i))
                    }

                    if let TaskItem::TaskCategory(_) = child {
                        if let Some(i) = child.parent_search(current_line_number + lines_skipped, find_line_number) {
                            return Some(i)
                        }
                        lines_skipped += child.line_length() - 1;
                    }

                    lines_skipped += 1;
                }
                None
            },
            TaskItem::Task(_) => return None,
        }
    }

    fn item_search(&mut self, current_line_number: usize, find_line_number: usize) -> Option<&mut TaskItem> {
        if current_line_number == find_line_number {
            return Some(self);
        }
        match self {
            TaskItem::TaskCategory(c) => {
                let mut lines_skipped = 1;
                for child in c.child.iter_mut() {
                    if current_line_number + lines_skipped == find_line_number {
                        return Some(child)
                    }

                    let children_len = if let TaskItem::TaskCategory(_) = child {
                        Some(child.line_length() - 1)
                    } else {
                        None
                    };

                    if let Some(i) = children_len {
                        let search_child = child.item_search(current_line_number + lines_skipped, find_line_number);
                        if search_child.is_some() {
                            return search_child
                        }
                        lines_skipped += i;
                    }

                    lines_skipped += 1;
                }
                return None
            },
            TaskItem::Task(_) => return None,
        }
    }

    pub fn line_length(&self) -> usize {
        // number of lines shown on todo list of this taskitem and children (if any)
        match self {
            TaskItem::Task(_) => 1,
            TaskItem::TaskCategory(c) => 1usize + c.child.iter().map(|x| x.line_length()).sum::<usize>()
        }
    }

    pub fn interact(&mut self) {
        match self {
            TaskItem::TaskCategory(_) => return,
            TaskItem::Task(t) => t.done = !t.done,
        }
    }
}
