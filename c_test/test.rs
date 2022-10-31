/*
linker.func_wrap("env", "test_func", |mut caller: wasmtime::Caller<'_, Host>, _ptr: u32, size: u32| {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
            //let my_realloc = caller.get_export("realloc").unwrap().into_func().unwrap().typed::<(u32, u32), u32, _>(&caller).unwrap();
            let linear_memory = memory.data(&caller).as_ptr();
            unsafe {
                register_my_realloc(wasmtime_realloc, &mut WasmtimeCtx {caller: &mut caller} as *mut WasmtimeCtx);
                register_my_malloc(wasmtime_malloc, &mut WasmtimeCtx {caller: &mut caller} as *mut WasmtimeCtx);  
                let ptr: *const u32 = linear_memory.add(_ptr as usize).cast();
                let ret_ptr: *const u8 = test_func(ptr, size).cast();
                ret_ptr.offset_from(linear_memory) as u32
            }   
        })?;

        linker.func_wrap("env", "test2_func", |mut caller: wasmtime::Caller<'_, Host>, _ptr: u32| {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
            //let my_realloc = caller.get_export("realloc").unwrap().into_func().unwrap().typed::<(u32, u32), u32, _>(&caller).unwrap();
            let linear_memory = memory.data(&caller).as_ptr();
            unsafe {
                let ptr: *const RustTransTrans = linear_memory.add(_ptr as usize).cast();
                let trans: *const RustTrans = linear_memory.add((*ptr).trans as usize).cast();
                let name: *const u8 = linear_memory.add((*trans).name as usize).cast();
                let trans = &Trans{name, age: (*trans).age} as *const Trans;
                let ptr = &TransTrans{trans: trans.cast()} as *const TransTrans;
                test2_func(ptr.cast())
            }   
        })?;

        linker.func_wrap("env", "test3_func", |mut caller: wasmtime::Caller<'_, Host>, _father: u32| {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
            //let my_realloc = caller.get_export("realloc").unwrap().into_func().unwrap().typed::<(u32, u32), u32, _>(&caller).unwrap();
            let linear_memory = memory.data_mut(&mut caller).as_mut_ptr();
            unsafe {
                let father: *mut RustFather = linear_memory.add(_father as usize).cast();
                let son = (*father).son;
                let name: *mut u8 = linear_memory.add(son.name as usize).cast();
                let son: Son = Son {name, age: son.age};
                let father: Father = Father { son };
                test3_func(father)
            }   
        })?;

        linker.func_wrap("env", "test5_func", wrap_test5_func)?;
        linker.func_wrap("env", "test6_func", wrap_test6_func)?;
        linker.func_wrap("env", "test6_func_2", wrap_test6_func_2)?;


#[repr(C)]
struct JSMallocState {
    malloc_count: u32,
    malloc_size: u32,
    malloc_limit: u32,
    opaque: *const libc::c_void
}

#[repr(C)]
struct WasmtimeCtx<'a> {
    pub caller: *mut wasmtime::Caller<'a, Host>
}

#[link(name = "my-helpers")]
extern "C" {
    fn test_func(ptr: *const u32, size: u32) -> *const u32;
    fn test2_func(ptr: *const libc::c_void);
    fn test3_func(f: Father);
    fn test5_func(d: *const DoubleList);
    fn test6_func(op: *mut c_void);
    fn test6_func_2(callback: MyCallback, closure: *mut c_void);
    fn register_my_realloc(re: extern fn(*const libc::c_void, u32, *mut WasmtimeCtx) -> *const libc::c_void, ctx: *mut WasmtimeCtx);
    fn register_my_malloc(re: extern fn(u32, *mut WasmtimeCtx) -> *const libc::c_void, ctx: *mut WasmtimeCtx);
}

extern fn wasmtime_realloc(ptr: *const libc::c_void, size: u32, ctx: *mut WasmtimeCtx) -> *const libc::c_void {
    unsafe {
        let caller = (*ctx).caller;
        let memory = (*caller).get_export("memory").unwrap().into_memory().unwrap();
        let my_realloc = (*caller).get_export("realloc").unwrap().into_func().unwrap().typed::<(u32, u32), u32, _>(&*caller).unwrap();

        let linear_memory = memory.data(&*caller).as_ptr();
        let ptr: *const u8 = ptr.cast();
        let ptr_offset = ptr.offset_from(linear_memory) as u32;
        let res = my_realloc.call(&mut *caller, (ptr_offset, size)).unwrap();
        linear_memory.add(res as usize).cast()
    }
}

extern fn wasmtime_malloc(size: u32, ctx: *mut WasmtimeCtx) -> *const libc::c_void {
    unsafe {
        let caller = (*ctx).caller;
        let memory = (*caller).get_export("memory").unwrap().into_memory().unwrap();
        let my_malloc = (*(*ctx).caller).get_export("malloc").unwrap().into_func().unwrap().typed::<u32, u32, _>(&*caller).unwrap();

        let linear_memory = memory.data(&*caller).as_ptr();
        let res = my_malloc.call(&mut *caller, size).unwrap();
        linear_memory.add(res as usize).cast()
    }
}


// fn wrap_test_func(mut caller: wasmtime::Caller<'_, Host>, _ptr: u32, size: u32) {
//     let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
//     let linear_memory: &[u8] = memory.data(&caller);
    
//     unsafe {
//         let ptr: *const u32 = linear_memory.as_ptr().add(_ptr as usize).cast();
//         test_func(ptr, size)
//     }
// }
// ==== test2 ==== // 

#[repr(C)]
struct TransTrans {
    trans: *const libc::c_void,
}

struct RustTransTrans {
    trans: i32,
}

#[repr(C)]
struct Trans {
    name: *const u8,
    age: i32,
}

struct RustTrans {
    name: i32,
    age: i32,
}

// ==== test3 ==== // 
#[derive(Clone, Copy)]
struct RustSon {
    pub name: i32,
    pub age: i32
}
#[derive(Clone, Copy)]
struct RustFather {
    pub son: RustSon,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Son{
    pub name: *const u8,
    pub age: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Father {
    pub son: Son,
}

struct RustDoubleList {
    prev: i32,
    next: i32,
    val: i32,
}
#[repr(C)]
struct DoubleList {
    prev: *mut DoubleList,
    next: *mut DoubleList,
    val: i32,
}

fn wrap_test5_func(mut caller: Caller<'_, Host>, _d: i32) {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    //let my_realloc = caller.get_export("realloc").unwrap().into_func().unwrap().typed::<(u32, u32), u32, _>(&caller).unwrap();
    let linear_memory = memory.data_mut(&mut caller).as_mut_ptr();
    unsafe {
        let d: *mut RustDoubleList = linear_memory.add(_d as usize).cast();
        let prev: *mut DoubleList = linear_memory.add((*d).prev as usize).cast();
        let next: *mut DoubleList = linear_memory.add((*d).next as usize).cast();
        let d: *const DoubleList = &DoubleList {prev, next, val: (*d).val};
        test5_func(d)
    }   
}

// ==== test6 ==== //
#[repr(C)]
#[derive(Clone, Copy)]
struct Name {
    i: [i32; 32],
    name: *mut u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RustName {
    i: [i32; 32],
    name: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RustFuncPointer {
    add: i32,
    my_malloc: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FuncPointer {
    add: unsafe extern "C" fn(i32, i32, *mut c_void) -> i32,
    add_closure: *mut c_void,
    my_malloc: unsafe extern "C" fn(u32, *mut c_void) -> *mut c_void,
    my_malloc_closure: *mut c_void,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct RustFuncParam {
    name: u32,
    fp: RustFuncPointer,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct FuncParam {
    name: Name,
    fp: FuncPointer,
}

fn wrap_test6_func(mut caller: Caller<'_, Host>, _fp: i32) {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let table = caller.get_export("__indirect_function_table").unwrap().into_table().unwrap();
    let linear_memory = memory.data_mut(&mut caller).as_mut_ptr();
    unsafe {
        let fp: *mut RustFuncParam = linear_memory.add(_fp as usize).cast();
        println!("{},{}", (*fp).fp.add, (*fp).fp.my_malloc);
        let name: *mut Name = linear_memory.add((*fp).name as usize).cast();
        let n: *mut u8 = linear_memory.add((*name).name as usize).cast();
        let add_func = table.get(&mut caller, (*fp).fp.add as u32).unwrap()
                .funcref().unwrap().unwrap()
                .typed::<(i32, i32), i32, _>(&caller).unwrap();
        let my_malloc_func = table.get(&mut caller, (*fp).fp.my_malloc as u32).unwrap()
                .funcref().unwrap().unwrap()
                .typed::<u32, u32, _>(&caller).unwrap();
        let mut add_closure = |a, b| {
            add_func.call(&mut caller, (a, b)).unwrap()
        };
        let add = get_trampoline(&add_closure);
        let add_closure = &mut add_closure as *mut _ as *mut c_void;
        let mut my_malloc_closure = |size: u32| -> *mut c_void {
            let ret = my_malloc_func.call(&mut caller, size).unwrap();
            linear_memory.add(ret as usize).cast()
        };
        let fp = &mut FuncParam {
            name: Name {i:(*name).i, name: n},
            fp: FuncPointer { 
                add,
                add_closure,
                my_malloc: get_trampoline2(&my_malloc_closure), 
                my_malloc_closure: &mut my_malloc_closure as *mut _ as *mut c_void, }} as *mut FuncParam;
        test6_func(fp as *mut c_void)
    }   
}


fn wrap_test6_func_2(mut caller: Caller<'_, Host>, _add: u32) {
    let table = caller.get_export("__indirect_function_table").unwrap().into_table().unwrap();
    let add_func = table.get(&mut caller, _add).unwrap()
                .funcref().unwrap().unwrap()
                .typed::<(i32, i32), i32, _>(&caller).unwrap();
    
    unsafe {
        let mut closure = |a, b| {
            add_func.call(&mut caller, (a, b)).unwrap()
        };
        test6_func_2(get_trampoline(&closure), &mut closure as *mut _ as *mut c_void)
    }   
}

type MyCallback = unsafe extern "C" fn(i32, i32, *mut c_void) -> i32;
type MallocCallback = unsafe extern "C" fn(u32, *mut c_void) -> *mut c_void;

unsafe extern "C" fn trampoline<F>(a: i32, b: i32, user_data: *mut c_void) -> i32
where F: FnMut(i32, i32) -> i32
{
    let user_data = &mut *(user_data as *mut F);
    user_data(a, b)
}

unsafe extern "C" fn trampoline2<F>(size: u32, user_data: *mut c_void) -> *mut c_void
where F: FnMut(u32) -> *mut c_void
{
    let user_data = &mut *(user_data as *mut F);
    user_data(size)
}

fn get_trampoline<F>(_closure: &F) -> MyCallback
where F: FnMut(i32, i32) -> i32,
{
    trampoline::<F>
}
fn get_trampoline2<F>(_closure: &F) -> MallocCallback
where F: FnMut(u32) -> *mut c_void
{
    trampoline2::<F>
}
*/