pub trait VecExtensions<T> {
    fn remove_first_where<F>(&mut self, predicate: F) -> Option<T>
    where
        F: Fn(&T) -> bool;
}

impl<T> VecExtensions<T> for Vec<T> {
    fn remove_first_where<F>(&mut self, predicate: F) -> Option<T>
    where
        F: Fn(&T) -> bool,
    {
        match self.iter().position(predicate) {
            Some(index) => Some(self.remove(index)),
            None => None,
        }
    }
}

pub trait SliceExtensions<T> {
    fn single_element(&self) -> Option<&T>;
}

impl<T> SliceExtensions<T> for [T] {
    fn single_element(&self) -> Option<&T> {
        match self.len() {
            1 => self.into_iter().next(),
            _ => None,
        }
    }
}
