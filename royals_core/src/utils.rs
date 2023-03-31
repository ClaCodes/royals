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
        self.iter()
            .position(predicate)
            .map(|index| self.remove(index))
    }
}

pub trait SliceExtensions<T> {
    fn single_element(&self) -> Option<&T>;
}

impl<T> SliceExtensions<T> for [T] {
    fn single_element(&self) -> Option<&T> {
        match self.len() {
            1 => self.iter().next(),
            _ => None,
        }
    }
}
