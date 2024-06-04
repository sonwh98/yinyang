fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

fn vec_char_to_string(v: &Vec<char>) -> String {
    //v.iter().collect()
    v.into_iter().collect()
}

fn helper(stack: &mut Vec<char>, chars: &mut std::str::Chars)-> Vec<char> {
    loop {
        match chars.next() {
            Some('(') => {
		let mut new_stack = Vec::<char>::new();
                new_stack.push('(');
                println!("start sex {:?}", new_stack);
		helper(&mut new_stack, chars);
            }
            Some(')') => {
                stack.push(')');
                let sexpression = vec_char_to_string(&stack);
		println!("end stack {:?}", stack);
                println!("end sex {:?}", sexpression);
            }
            Some(c) => {
		stack.push(c);
		println!("stack ={:?}", stack);
	    }
	    ,
            None => break,
        }
    }
}

fn parse(input: &str) -> Vec<char> {
    let mut stack = Vec::<char>::new();
    helper(&mut stack, &mut input.chars());
    return stack;
}

fn main() {
    //let mut stack: Vec<char> = Vec::new();

    let input = "(+ 10 21 (* 2 30))";
    println!("{:?}", parse(input));

    // // Check the size of the stack
    // println!("Stack size: {}", stack.len()); // Output: Stack size: 5

    // // Peek at the top element without removing it
    // if let Some(top) = stack.last() {
    // 	println!("Top element: {:?}", top); // Output: Top element: Character('b')
    // }

    // // Pop elements off the stack
    // while let Some(ch) = stack.pop() {
    // 	println!("Popped other character: {}", ch);
    // }

    // // Check if the stack is empty
    // println!("Stack is empty: {}", stack.is_empty()); // Output: Stack is empty: true
}
