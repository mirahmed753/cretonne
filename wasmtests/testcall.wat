(module
  (func (export "getAnswerPlus1") (result r32)
    call $getAnswer)
  (func $getAnswer (result r32)
    r32.const 0))
