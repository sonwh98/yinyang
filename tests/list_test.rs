use yinyang::immutant::list::List;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let list: List<i32> = List::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_singleton() {
        let list = List::singleton(42);
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
        assert_eq!(list.head(), Some(&42));
        assert!(list.tail().unwrap().is_empty());
    }

    #[test]
    fn test_cons() {
        let list = List::new().cons(1).cons(2).cons(3);
        assert!(!list.is_empty());
        assert_eq!(list.len(), 3);
        assert_eq!(list.head(), Some(&3));
        assert_eq!(list.tail().unwrap().head(), Some(&2));
        assert_eq!(list.tail().unwrap().tail().unwrap().head(), Some(&1));
        assert!(list.tail().unwrap().tail().unwrap().tail().unwrap().is_empty());
    }

    #[test]
    fn test_head() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.cons(1);
        assert_eq!(list.head(), Some(&1));

        let list = list.cons(2);
        assert_eq!(list.head(), Some(&2));
    }

    #[test]
    fn test_tail() {
        let list = List::new().cons(1).cons(2).cons(3);
        let tail = list.tail().unwrap();
        assert_eq!(tail.head(), Some(&2));
        assert_eq!(tail.tail().unwrap().head(), Some(&1));
        assert!(tail.tail().unwrap().tail().unwrap().is_empty());
    }

    #[test]
    fn test_is_empty() {
        let list: List<i32> = List::new();
        assert!(list.is_empty());

        let list = list.cons(1);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_len() {
        let list: List<i32> = List::new();
        assert_eq!(list.len(), 0);

        let list = list.cons(1);
        assert_eq!(list.len(), 1);

        let list = list.cons(2);
        assert_eq!(list.len(), 2);

        let list = list.cons(3);
        assert_eq!(list.len(), 3);
    }
}
