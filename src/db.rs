use std::slice::Iter;

use crate::monitors::{self, Monitor};

pub struct Db(Vec<Monitor>);

impl Db {
    pub fn new() -> Db {
        Db(monitors::get_monitors())
    }

    pub fn iter(&self) -> Iter<Monitor> {
        self.0.iter()
    }

    pub fn get(&self, id: i32) -> Option<&Monitor> {
        self.0.iter().find(|m| m.id() == id)
    }

    pub fn refresh(&mut self) {
        self.0 = monitors::get_monitors();
    }
}
