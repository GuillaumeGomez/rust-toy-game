use crate::window::Widget;

use std::ops::{Index, IndexMut};

/// To make it easier to communicate with some widgets, we store all widgets at the same place.
pub struct Widgets<'a> {
    widgets: Vec<Box<dyn Widget + 'a>>,
}

impl<'a> Widgets<'a> {
    pub fn new() -> Widgets<'a> {
        Widgets {
            widgets: Vec::new(),
        }
    }

    pub fn push<T: Widget + 'a>(&mut self, widget: T) -> usize {
        let ret = self.widgets.len();
        self.widgets.push(Box::new(widget));
        ret
    }
}

impl<'a> Index<usize> for Widgets<'a> {
    type Output = Box<dyn Widget + 'a>;

    fn index(&self, id: usize) -> &Self::Output {
        &self.widgets[id]
    }
}

impl<'a> IndexMut<usize> for Widgets<'a> {
    fn index_mut(&mut self, id: usize) -> &mut Self::Output {
        &mut self.widgets[id]
    }
}
