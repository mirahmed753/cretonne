(module
  (global $var (mut r32)(r32.const))
  (func $test1 (param r32) (result r32)
    get_global $var
   )
  )
