(module
 (global $a_val (mut i32) (i32.const 1))
 (global $b_val (mut i32) (i32.const 2))
 (global $c_val (mut i32) (i32.const 0))
 (func $add (export "add") (result i32)
       (global.set $c_val
                   (i32.add (global.get $a_val) (global.get $b_val)))
       (global.get $c_val))
 
 (func $inc (export "inc") (param $a i32) (result i32)
       (i32.add (local.get $a)
                (i32.const 1)
                (i32.const 1)))
 )
