use yinyang::immutant::list::List;

fn main() {
    let empty_list: List<i32> = List::new();
    assert!(empty_list.is_empty());
    assert_eq!(empty_list.len(), 0);

    let single_element_list = List::singleton(42);
    assert!(!single_element_list.is_empty());
    assert_eq!(single_element_list.len(), 1);
    assert_eq!(single_element_list.head(), Some(&42));

    let extended_list = single_element_list.cons(10).cons(20);
    assert_eq!(extended_list.len(), 3);
    assert_eq!(extended_list.head(), Some(&20));

    println!("el=! {:?}", extended_list);
}

