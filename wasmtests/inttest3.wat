(module
  (global $var (mut i32)(i32.const 0))
  (func $test1 (param i32) (result i32)
    get_global $var
   )
  )
