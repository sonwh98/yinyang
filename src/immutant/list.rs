use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum List<T>
where
    T: Clone,
{
    Cons(T, Rc<List<T>>),
    Nil,
}

pub struct ListIter<'a, T: Clone> {
    current: Option<&'a List<T>>,
}

impl<'a, T: Clone> Iterator for ListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        match current {
            List::Nil => {
                self.current = None;
                None
            }
            List::Cons(head, tail) => {
                self.current = Some(&*tail);
                Some(head)
            }
        }
    }
}

impl<T: Clone> List<T>
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

    pub fn append(&self, element: T) -> Self {
        match self {
            List::Nil => List::singleton(element),
            List::Cons(head, tail) => List::Cons(head.clone(), Rc::new(tail.append(element))),
        }
    }

    // Get the head (first element) of the list
    pub fn head(&self) -> Option<&T> {
        match self {
            List::Cons(head, _) => Some(head),
            List::Nil => None,
        }
    }

    pub fn first(&self) -> Option<&T> {
        self.head()
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

    pub fn from_vec(v: Vec<T>) -> Self {
        let mut list = List::Nil;
        for item in v.into_iter().rev() {
            list = List::Cons(item, Rc::new(list));
        }
        list
    }

    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::new();
        let mut current = self;
        while let List::Cons(head, tail) = current {
            result.push(head.clone());
            current = tail;
        }
        result
    }

    /// Returns an iterator over references to the elements of the list
    pub fn iter(&self) -> ListIter<'_, T> {
        ListIter {
            current: Some(self),
        }
    }

    pub fn rest(&self) -> Self {
        match self {
            List::Cons(_, tail) => (**tail).clone(),
            List::Nil => List::Nil,
        }
    }

    pub fn reverse(&self) -> Self {
        let mut current = self;
        let mut reversed = List::Nil;

        while let List::Cons(head, tail) = current {
            reversed = List::Cons(head.clone(), Rc::new(reversed));
            current = tail;
        }

        reversed
    }
}
