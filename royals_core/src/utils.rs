pub trait RemoveFirstWhere<T> {
    fn remove_first_where<F>(&mut self, predicate: F) -> Option<T>
    where
        F: Fn(&T) -> bool;
}

impl<T> RemoveFirstWhere<T> for Vec<T> {
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
