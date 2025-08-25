use std::rc::Rc;
use std::fmt;

const BRANCH_FACTOR: usize = 32;
const BITS: usize = 5; // log2(32)

#[derive(Debug, Clone)]
pub struct PersistentVector<T> {
    count: usize,
    shift: usize,
    root: Rc<Node<T>>,
    tail: Rc<Vec<T>>,
}

#[derive(Debug, Clone)]
enum Node<T> {
    Internal(Vec<Rc<Node<T>>>),
    Leaf(Vec<T>),
}

impl<T: Clone> PersistentVector<T> {
    pub fn new() -> Self {
        PersistentVector {
            count: 0,
            shift: BITS,
            root: Rc::new(Node::Internal(Vec::new())),
            tail: Rc::new(Vec::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn conj(&self, item: T) -> Self {
        if self.tail.len() < BRANCH_FACTOR {
            // Room in tail, just add to it
            let mut new_tail = (*self.tail).clone();
            new_tail.push(item);
            
            PersistentVector {
                count: self.count + 1,
                shift: self.shift,
                root: self.root.clone(),
                tail: Rc::new(new_tail),
            }
        } else {
            // Tail is full, need to push it into the tree
            let new_root = if (self.count >> BITS) > (1 << self.shift) {
                // Need to create a new root
                let new_shift = self.shift + BITS;
                Rc::new(Node::Internal(vec![
                    self.root.clone(),
                    self.push_tail(new_shift, &self.root, &self.tail),
                ]))
            } else {
                self.push_tail(self.shift, &self.root, &self.tail)
            };

            PersistentVector {
                count: self.count + 1,
                shift: if (self.count >> BITS) > (1 << self.shift) { self.shift + BITS } else { self.shift },
                root: new_root,
                tail: Rc::new(vec![item]),
            }
        }
    }

    fn push_tail(&self, level: usize, parent: &Rc<Node<T>>, tail: &Rc<Vec<T>>) -> Rc<Node<T>> {
        let subidx = ((self.count - 1) >> level) & 0x1f;
        
        match parent.as_ref() {
            Node::Internal(children) => {
                if level == BITS {
                    // We're at the level just above leaves
                    let mut new_children = children.clone();
                    new_children.push(Rc::new(Node::Leaf((**tail).clone())));
                    Rc::new(Node::Internal(new_children))
                } else {
                    let mut new_children = children.clone();
                    if subidx < children.len() {
                        new_children[subidx] = self.push_tail(level - BITS, &children[subidx], tail);
                    } else {
                        new_children.push(self.push_tail(level - BITS, &Rc::new(Node::Internal(Vec::new())), tail));
                    }
                    Rc::new(Node::Internal(new_children))
                }
            }
            Node::Leaf(_) => {
                panic!("Cannot push tail to leaf node");
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.count {
            return None;
        }

        if index >= self.tail_offset() {
            // Item is in tail
            let tail_index = index & 0x1f;
            return self.tail.get(tail_index);
        }

        // Item is in tree
        self.array_for(index).and_then(|arr| arr.get(index & 0x1f))
    }

    fn tail_offset(&self) -> usize {
        if self.count < BRANCH_FACTOR {
            0
        } else {
            ((self.count - 1) >> BITS) << BITS
        }
    }

    fn array_for(&self, index: usize) -> Option<&Vec<T>> {
        if index >= self.count {
            return None;
        }

        if index >= self.tail_offset() {
            return Some(&self.tail);
        }

        let mut node = &self.root;
        let mut level = self.shift;

        loop {
            match node.as_ref() {
                Node::Internal(children) => {
                    let subidx = (index >> level) & 0x1f;
                    if let Some(child) = children.get(subidx) {
                        node = child;
                        if level == BITS {
                            break;
                        }
                        level -= BITS;
                    } else {
                        return None;
                    }
                }
                Node::Leaf(arr) => {
                    return Some(arr);
                }
            }
        }

        match node.as_ref() {
            Node::Leaf(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn pop(&self) -> Option<Self> {
        if self.count == 0 {
            return None;
        }

        if self.count == 1 {
            return Some(PersistentVector::new());
        }

        if self.tail.len() > 1 {
            // Remove from tail
            let mut new_tail = (*self.tail).clone();
            new_tail.pop();
            
            Some(PersistentVector {
                count: self.count - 1,
                shift: self.shift,
                root: self.root.clone(),
                tail: Rc::new(new_tail),
            })
        } else {
            // Need to get new tail from tree
            let new_tail = self.array_for(self.count - 2).unwrap();
            let new_root = self.pop_tail(self.shift, &self.root);
            
            Some(PersistentVector {
                count: self.count - 1,
                shift: self.shift,
                root: new_root.unwrap_or_else(|| Rc::new(Node::Internal(Vec::new()))),
                tail: Rc::new(new_tail.clone()),
            })
        }
    }

    fn pop_tail(&self, level: usize, node: &Rc<Node<T>>) -> Option<Rc<Node<T>>> {
        let subidx = ((self.count - 2) >> level) & 0x1f;
        
        match node.as_ref() {
            Node::Internal(children) => {
                if level > BITS {
                    let new_child = self.pop_tail(level - BITS, &children[subidx]);
                    if new_child.is_some() || subidx > 0 {
                        let mut new_children = children.clone();
                        if let Some(child) = new_child {
                            new_children[subidx] = child;
                        } else {
                            new_children.truncate(subidx);
                        }
                        Some(Rc::new(Node::Internal(new_children)))
                    } else {
                        None
                    }
                } else if subidx > 0 {
                    let mut new_children = children.clone();
                    new_children.truncate(subidx);
                    Some(Rc::new(Node::Internal(new_children)))
                } else {
                    None
                }
            }
            Node::Leaf(_) => None,
        }
    }

    pub fn iter(&self) -> PersistentVectorIter<T> {
        PersistentVectorIter {
            vector: self,
            index: 0,
        }
    }
}

impl<T: Clone> Default for PersistentVector<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Display + Clone> fmt::Display for PersistentVector<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, "]")
    }
}

pub struct PersistentVectorIter<'a, T> {
    vector: &'a PersistentVector<T>,
    index: usize,
}

impl<'a, T: Clone> Iterator for PersistentVectorIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vector.len() {
            let item = self.vector.get(self.index);
            self.index += 1;
            item
        } else {
            None
        }
    }
}

// Example usage and tests
fn main() {
    let v1 = PersistentVector::new();
    let v2 = v1.conj(1).conj(2).conj(3);
    let v3 = v2.conj(4).conj(5);
    
    println!("v1: {}", v1);
    println!("v2: {}", v2);
    println!("v3: {}", v3);
    
    println!("v2[1] = {:?}", v2.get(1));
    println!("v3[4] = {:?}", v3.get(4));
    
    let v4 = v3.pop().unwrap();
    println!("v4 (v3 popped): {}", v4);
    
    // Test with larger vector
    let mut large_v = PersistentVector::new();
    for i in 0..100 {
        large_v = large_v.conj(i);
    }
    println!("Large vector length: {}", large_v.len());
    println!("Element at index 50: {:?}", large_v.get(50));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_vector() {
        let v: PersistentVector<i32> = PersistentVector::new();
        assert_eq!(v.len(), 0);
        assert!(v.is_empty());
        assert_eq!(v.get(0), None);
    }

    #[test]
    fn test_conj() {
        let v1 = PersistentVector::new();
        let v2 = v1.conj(1);
        let v3 = v2.conj(2);
        
        assert_eq!(v1.len(), 0);
        assert_eq!(v2.len(), 1);
        assert_eq!(v3.len(), 2);
        
        assert_eq!(v2.get(0), Some(&1));
        assert_eq!(v3.get(0), Some(&1));
        assert_eq!(v3.get(1), Some(&2));
    }

    #[test]
    fn test_large_vector() {
        let mut v = PersistentVector::new();
        for i in 0..1000 {
            v = v.conj(i);
        }
        
        assert_eq!(v.len(), 1000);
        for i in 0..1000 {
            assert_eq!(v.get(i), Some(&i));
        }
    }

    #[test]
    fn test_pop() {
        let v1 = PersistentVector::new().conj(1).conj(2).conj(3);
        let v2 = v1.pop().unwrap();
        let v3 = v2.pop().unwrap();
        
        assert_eq!(v1.len(), 3);
        assert_eq!(v2.len(), 2);
        assert_eq!(v3.len(), 1);
        
        assert_eq!(v1.get(2), Some(&3));
        assert_eq!(v2.get(1), Some(&2));
        assert_eq!(v3.get(0), Some(&1));
    }

    #[test]
    fn test_structural_sharing() {
        let v1 = PersistentVector::new().conj(1).conj(2);
        let v2 = v1.conj(3);
        let v3 = v1.conj(4);
        
        // v2 and v3 should share structure with v1
        assert_eq!(v1.len(), 2);
        assert_eq!(v2.len(), 3);
        assert_eq!(v3.len(), 3);
        
        assert_eq!(v2.get(2), Some(&3));
        assert_eq!(v3.get(2), Some(&4));
        
        // v1 should be unchanged
        assert_eq!(v1.get(0), Some(&1));
        assert_eq!(v1.get(1), Some(&2));
    }
}
