use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum List<T>
where
    T: Clone,
{
    Cons(T, Rc<List<T>>),
    Nil,
}

impl<T> List<T>
where
    T: Clone,
{
    // Create a new empty list
    pub fn new() -> Self {
        List::Nil
    }

    // Create a new list with a single element
    pub fn singleton(element: T) -> Self {
        List::Cons(element, Rc::new(List::Nil))
    }

    // Add an element to the front of the list
    pub fn cons(&self, element: T) -> Self {
        List::Cons(element, Rc::new(self.clone()))
    }

    // Get the head (first element) of the list
    pub fn head(&self) -> Option<&T> {
        match self {
            List::Cons(head, _) => Some(head),
            List::Nil => None,
        }
    }

    // Get the tail (rest) of the list
    pub fn tail(&self) -> Option<Rc<List<T>>> {
        match self {
            List::Cons(_, tail) => Some(Rc::clone(tail)),
            List::Nil => None,
        }
    }

    // Check if the list is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, List::Nil)
    }

    // Get the length of the list
    pub fn len(&self) -> usize {
        match self {
            List::Nil => 0,
            List::Cons(_, tail) => 1 + tail.len(),
        }
    }
}
