(module
  (type (;0;) (func (param i32)))
  (type (;1;) (func (param i32 i32) (result i32)))
  (type (;2;) (func (result i32)))
  (type (;3;) (func))
  (import "env" "test6_func_2" (func $test6_func_2 (type 0)))
  (import "wasi_snapshot_preview1" "proc_exit" (func $__imported_wasi_snapshot_preview1_proc_exit (type 0)))
  (func $add_def (type 1) (param i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    local.get 0
    i32.store offset=12
    local.get 2
    local.get 1
    i32.store offset=8
    local.get 2
    i32.load offset=12
    local.get 2
    i32.load offset=8
    i32.add)
  (func $__original_main (type 2) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 0
    global.set $__stack_pointer
    local.get 0
    i32.const 0
    i32.store offset=12
    i32.const 1
    call $test6_func_2
    local.get 0
    i32.const 16
    i32.add
    global.set $__stack_pointer
    i32.const 0)
  (func $_start (type 3)
    (local i32)
    block  ;; label = @1
      call $__original_main
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      call $exit
      unreachable
    end)
  (func $__wasi_proc_exit (type 0) (param i32)
    local.get 0
    call $__imported_wasi_snapshot_preview1_proc_exit
    unreachable)
  (func $_Exit (type 0) (param i32)
    local.get 0
    call $__wasi_proc_exit
    unreachable)
  (func $dummy (type 3))
  (func $__wasm_call_dtors (type 3)
    call $dummy
    call $dummy)
  (func $exit (type 0) (param i32)
    call $dummy
    call $dummy
    local.get 0
    call $_Exit
    unreachable)
  (func $_start.command_export (type 3)
    call $_start
    call $__wasm_call_dtors)
  (table (;0;) 2 2 funcref)
  (memory (;0;) 2)
  (global $__stack_pointer (mut i32) (i32.const 66560))
  (export "memory" (memory 0))
  (export "__indirect_function_table" (table 0))
  (export "_start" (func $_start.command_export))
  (elem (;0;) (i32.const 1) func $add_def))
