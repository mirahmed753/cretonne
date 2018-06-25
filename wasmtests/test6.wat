(module
  (global $x (mut r32) (r32.const 0))
  (memory 1)
  (func $main (local r32)
    (r32.store (r32.const 0) (get_global $x))
  )
  (start $main)
)
