//! The module that implements the `wasmtime run` command.

use anyhow::{anyhow, bail, Context as _, Result};
use clap::Parser;
use libc::c_void;
use once_cell::sync::Lazy;
use std::any::Any;
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;
use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
    process,
};
use wasmtime::{Engine, Func, Linker, Module, Store, Trap, Val, ValType, Caller};
use wasmtime_cli_flags::{CommonOptions, WasiModules};
use wasmtime_wasi::sync::{ambient_authority, Dir, TcpListener, WasiCtxBuilder};

#[cfg(feature = "wasi-nn")]
use wasmtime_wasi_nn::WasiNnCtx;

#[cfg(feature = "wasi-crypto")]
use wasmtime_wasi_crypto::WasiCryptoCtx;

fn parse_module(s: &OsStr) -> anyhow::Result<PathBuf> {
    // Do not accept wasmtime subcommand names as the module name
    match s.to_str() {
        Some("help") | Some("config") | Some("run") | Some("wast") | Some("compile") => {
            bail!("module name cannot be the same as a subcommand")
        }
        _ => Ok(s.into()),
    }
}

fn parse_env_var(s: &str) -> Result<(String, String)> {
    let parts: Vec<_> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        bail!("must be of the form `key=value`");
    }
    Ok((parts[0].to_owned(), parts[1].to_owned()))
}

fn parse_map_dirs(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.split("::").collect();
    if parts.len() != 2 {
        bail!("must contain exactly one double colon ('::')");
    }
    Ok((parts[0].into(), parts[1].into()))
}

fn parse_dur(s: &str) -> Result<Duration> {
    // assume an integer without a unit specified is a number of seconds ...
    if let Ok(val) = s.parse() {
        return Ok(Duration::from_secs(val));
    }
    // ... otherwise try to parse it with units such as `3s` or `300ms`
    let dur = humantime::parse_duration(s)?;
    Ok(dur)
}

fn parse_preloads(s: &str) -> Result<(String, PathBuf)> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        bail!("must contain exactly one equals character ('=')");
    }
    Ok((parts[0].into(), parts[1].into()))
}

static AFTER_HELP: Lazy<String> = Lazy::new(|| crate::FLAG_EXPLANATIONS.to_string());

/// Runs a WebAssembly module
#[derive(Parser)]
#[structopt(name = "run", trailing_var_arg = true, after_help = AFTER_HELP.as_str())]
pub struct RunCommand {
    #[clap(flatten)]
    common: CommonOptions,

    /// Allow unknown exports when running commands.
    #[clap(long = "allow-unknown-exports")]
    allow_unknown_exports: bool,

    /// Allow the main module to import unknown functions, using an
    /// implementation that immediately traps, when running commands.
    #[clap(long = "trap-unknown-imports")]
    trap_unknown_imports: bool,

    /// Allow executing precompiled WebAssembly modules as `*.cwasm` files.
    ///
    /// Note that this option is not safe to pass if the module being passed in
    /// is arbitrary user input. Only `wasmtime`-precompiled modules generated
    /// via the `wasmtime compile` command or equivalent should be passed as an
    /// argument with this option specified.
    #[clap(long = "allow-precompiled")]
    allow_precompiled: bool,

    /// Inherit environment variables and file descriptors following the
    /// systemd listen fd specification (UNIX only)
    #[clap(long = "listenfd")]
    listenfd: bool,

    /// Grant access to the given TCP listen socket
    #[clap(
        long = "tcplisten",
        number_of_values = 1,
        value_name = "SOCKET ADDRESS"
    )]
    tcplisten: Vec<String>,

    /// Grant access to the given host directory
    #[clap(long = "dir", number_of_values = 1, value_name = "DIRECTORY")]
    dirs: Vec<String>,

    /// Pass an environment variable to the program
    #[clap(long = "env", number_of_values = 1, value_name = "NAME=VAL", parse(try_from_str = parse_env_var))]
    vars: Vec<(String, String)>,

    /// The name of the function to run
    #[clap(long, value_name = "FUNCTION")]
    invoke: Option<String>,

    /// Grant access to a guest directory mapped as a host directory
    #[clap(long = "mapdir", number_of_values = 1, value_name = "GUEST_DIR::HOST_DIR", parse(try_from_str = parse_map_dirs))]
    map_dirs: Vec<(String, String)>,

    /// The path of the WebAssembly module to run
    #[clap(
        required = true,
        value_name = "MODULE",
        parse(try_from_os_str = parse_module),
    )]
    module: PathBuf,

    /// Load the given WebAssembly module before the main module
    #[clap(
        long = "preload",
        number_of_values = 1,
        value_name = "NAME=MODULE_PATH",
        parse(try_from_str = parse_preloads)
    )]
    preloads: Vec<(String, PathBuf)>,

    /// Maximum execution time of wasm code before timing out (1, 2s, 100ms, etc)
    #[clap(
        long = "wasm-timeout",
        value_name = "TIME",
        parse(try_from_str = parse_dur),
    )]
    wasm_timeout: Option<Duration>,

    // NOTE: this must come last for trailing varargs
    /// The arguments to pass to the module
    #[clap(value_name = "ARGS")]
    module_args: Vec<String>,
}

impl RunCommand {
    /// Executes the command.
    pub fn execute(&self) -> Result<()> {
        self.common.init_logging();

        let mut config = self.common.config(None)?;
        if self.wasm_timeout.is_some() {
            config.epoch_interruption(true);
        }
        let engine = Engine::new(&config)?;
        let mut store = Store::new(&engine, Host::default());

        // If fuel has been configured, we want to add the configured
        // fuel amount to this store.
        if let Some(fuel) = self.common.fuel {
            store.add_fuel(fuel)?;
        }

        let preopen_sockets = self.compute_preopen_sockets()?;

        // Make wasi available by default.
        let preopen_dirs = self.compute_preopen_dirs()?;
        let argv = self.compute_argv();

        let mut linker = Linker::new(&engine);
        linker.allow_unknown_exports(self.allow_unknown_exports);

        // linker.func_wrap("env", "test_func", wrap_test_func)?;
        

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

        populate_with_wasi(
            &mut store,
            &mut linker,
            preopen_dirs,
            &argv,
            &self.vars,
            &self.common.wasi_modules.unwrap_or(WasiModules::default()),
            self.listenfd,
            preopen_sockets,
        )?;

        // Load the preload wasm modules.
        for (name, path) in self.preloads.iter() {
            // Read the wasm module binary either as `*.wat` or a raw binary
            let module = self.load_module(&engine, path)?;

            // Add the module's functions to the linker.
            linker.module(&mut store, name, &module).context(format!(
                "failed to process preload `{}` at `{}`",
                name,
                path.display()
            ))?;
        }

        // Load the main wasm module.
        match self
            .load_main_module(&mut store, &mut linker)
            .with_context(|| format!("failed to run main module `{}`", self.module.display()))
        {
            Ok(()) => (),
            Err(e) => {
                // If the program exited because of a non-zero exit status, print
                // a message and exit.
                if let Some(trap) = e.downcast_ref::<Trap>() {
                    // Print the error message in the usual way.
                    if let Some(status) = trap.i32_exit_status() {
                        // On Windows, exit status 3 indicates an abort (see below),
                        // so return 1 indicating a non-zero status to avoid ambiguity.
                        if cfg!(windows) && status >= 3 {
                            process::exit(1);
                        }
                        process::exit(status);
                    }

                    eprintln!("Error: {:?}", e);

                    // If the program exited because of a trap, return an error code
                    // to the outside environment indicating a more severe problem
                    // than a simple failure.
                    if cfg!(unix) {
                        // On Unix, return the error code of an abort.
                        process::exit(128 + libc::SIGABRT);
                    } else if cfg!(windows) {
                        // On Windows, return 3.
                        // https://docs.microsoft.com/en-us/cpp/c-runtime-library/reference/abort?view=vs-2019
                        process::exit(3);
                    }
                }
                return Err(e);
            }
        }

        Ok(())
    }

    fn compute_preopen_dirs(&self) -> Result<Vec<(String, Dir)>> {
        let mut preopen_dirs = Vec::new();

        for dir in self.dirs.iter() {
            preopen_dirs.push((
                dir.clone(),
                Dir::open_ambient_dir(dir, ambient_authority())
                    .with_context(|| format!("failed to open directory '{}'", dir))?,
            ));
        }

        for (guest, host) in self.map_dirs.iter() {
            preopen_dirs.push((
                guest.clone(),
                Dir::open_ambient_dir(host, ambient_authority())
                    .with_context(|| format!("failed to open directory '{}'", host))?,
            ));
        }

        Ok(preopen_dirs)
    }

    fn compute_preopen_sockets(&self) -> Result<Vec<TcpListener>> {
        let mut listeners = vec![];

        for address in &self.tcplisten {
            let stdlistener = std::net::TcpListener::bind(address)
                .with_context(|| format!("failed to bind to address '{}'", address))?;

            let _ = stdlistener.set_nonblocking(true)?;

            listeners.push(TcpListener::from_std(stdlistener))
        }
        Ok(listeners)
    }

    fn compute_argv(&self) -> Vec<String> {
        let mut result = Vec::new();

        // Add argv[0], which is the program name. Only include the base name of the
        // main wasm module, to avoid leaking path information.
        result.push(
            self.module
                .components()
                .next_back()
                .map(Component::as_os_str)
                .and_then(OsStr::to_str)
                .unwrap_or("")
                .to_owned(),
        );

        // Add the remaining arguments.
        for arg in self.module_args.iter() {
            result.push(arg.clone());
        }

        result
    }

    fn load_main_module(&self, store: &mut Store<Host>, linker: &mut Linker<Host>) -> Result<()> {
        if let Some(timeout) = self.wasm_timeout {
            store.set_epoch_deadline(1);
            let engine = store.engine().clone();
            thread::spawn(move || {
                thread::sleep(timeout);
                engine.increment_epoch();
            });
        }

        // Read the wasm module binary either as `*.wat` or a raw binary.
        let module = self.load_module(linker.engine(), &self.module)?;

        //let instance = linker.instantiate(&mut *store, &module)?;
        //let my_realloc = instance.get_typed_func::<(u32, u32), u32, _>(&mut *store, "realloc").unwrap();

        // The main module might be allowed to have unknown imports, which
        // should be defined as traps:
        if self.trap_unknown_imports {
            linker.define_unknown_imports_as_traps(&module)?;
        }
        // Use "" as a default module name.
        linker
            .module(&mut *store, "", &module)
            .context(format!("failed to instantiate {:?}", self.module))?;

        // If a function to invoke was given, invoke it.
        if let Some(name) = self.invoke.as_ref() {
            self.invoke_export(store, linker, name)
        } else {
            let func = linker.get_default(&mut *store, "")?;
            self.invoke_func(store, func, None)
        }
    }

    fn invoke_export(
        &self,
        store: &mut Store<Host>,
        linker: &Linker<Host>,
        name: &str,
    ) -> Result<()> {
        let func = match linker
            .get(&mut *store, "", name)
            .ok_or_else(|| anyhow!("no export named `{}` found", name))?
            .into_func()
        {
            Some(func) => func,
            None => bail!("export of `{}` wasn't a function", name),
        };
        self.invoke_func(store, func, Some(name))
    }

    fn invoke_func(&self, store: &mut Store<Host>, func: Func, name: Option<&str>) -> Result<()> {
        let ty = func.ty(&store);
        if ty.params().len() > 0 {
            eprintln!(
                "warning: using `--invoke` with a function that takes arguments \
                 is experimental and may break in the future"
            );
        }
        let mut args = self.module_args.iter();
        let mut values = Vec::new();
        for ty in ty.params() {
            let val = match args.next() {
                Some(s) => s,
                None => {
                    if let Some(name) = name {
                        bail!("not enough arguments for `{}`", name)
                    } else {
                        bail!("not enough arguments for command default")
                    }
                }
            };
            values.push(match ty {
                // TODO: integer parsing here should handle hexadecimal notation
                // like `0x0...`, but the Rust standard library currently only
                // parses base-10 representations.
                ValType::I32 => Val::I32(val.parse()?),
                ValType::I64 => Val::I64(val.parse()?),
                ValType::F32 => Val::F32(val.parse()?),
                ValType::F64 => Val::F64(val.parse()?),
                t => bail!("unsupported argument type {:?}", t),
            });
        }

        // Invoke the function and then afterwards print all the results that came
        // out, if there are any.
        let mut results = vec![Val::null(); ty.results().len()];
        func.call(store, &values, &mut results).with_context(|| {
            if let Some(name) = name {
                format!("failed to invoke `{}`", name)
            } else {
                format!("failed to invoke command default")
            }
        })?;
        if !results.is_empty() {
            eprintln!(
                "warning: using `--invoke` with a function that returns values \
                 is experimental and may break in the future"
            );
        }

        for result in results {
            match result {
                Val::I32(i) => println!("{}", i),
                Val::I64(i) => println!("{}", i),
                Val::F32(f) => println!("{}", f32::from_bits(f)),
                Val::F64(f) => println!("{}", f64::from_bits(f)),
                Val::ExternRef(_) => println!("<externref>"),
                Val::FuncRef(_) => println!("<funcref>"),
                Val::V128(i) => println!("{}", i),
            }
        }

        Ok(())
    }

    fn load_module(&self, engine: &Engine, path: &Path) -> Result<Module> {
        // Peek at the first few bytes of the file to figure out if this is
        // something we can pass off to `deserialize_file` which is fastest if
        // we don't actually read the whole file into memory. Note that this
        // behavior is disabled by default, though, because it's not safe to
        // pass arbitrary user input to this command with `--allow-precompiled`
        let mut file =
            File::open(path).with_context(|| format!("failed to open: {}", path.display()))?;
        let mut magic = [0; 4];
        if let Ok(()) = file.read_exact(&mut magic) {
            if &magic == b"\x7fELF" {
                if self.allow_precompiled {
                    return unsafe { Module::deserialize_file(engine, path) };
                }
                bail!(
                    "cannot load precompiled module `{}` unless --allow-precompiled is passed",
                    path.display()
                )
            }
        }

        Module::from_file(engine, path)
    }
}

#[derive(Default)]
struct Host {
    wasi: Option<wasmtime_wasi::WasiCtx>,
    #[cfg(feature = "wasi-nn")]
    wasi_nn: Option<WasiNnCtx>,
    #[cfg(feature = "wasi-crypto")]
    wasi_crypto: Option<WasiCryptoCtx>,
}

/// Populates the given `Linker` with WASI APIs.
fn populate_with_wasi(
    store: &mut Store<Host>,
    linker: &mut Linker<Host>,
    preopen_dirs: Vec<(String, Dir)>,
    argv: &[String],
    vars: &[(String, String)],
    wasi_modules: &WasiModules,
    listenfd: bool,
    mut tcplisten: Vec<TcpListener>,
) -> Result<()> {
    if wasi_modules.wasi_common {
        wasmtime_wasi::add_to_linker(linker, |host| host.wasi.as_mut().unwrap())?;

        let mut builder = WasiCtxBuilder::new();
        builder = builder.inherit_stdio().args(argv)?.envs(vars)?;

        let mut num_fd: usize = 3;

        if listenfd {
            let (n, b) = ctx_set_listenfd(num_fd, builder)?;
            num_fd = n;
            builder = b;
        }

        for listener in tcplisten.drain(..) {
            builder = builder.preopened_socket(num_fd as _, listener)?;
            num_fd += 1;
        }

        for (name, dir) in preopen_dirs.into_iter() {
            builder = builder.preopened_dir(dir, name)?;
        }

        store.data_mut().wasi = Some(builder.build());
    }

    if wasi_modules.wasi_nn {
        #[cfg(not(feature = "wasi-nn"))]
        {
            bail!("Cannot enable wasi-nn when the binary is not compiled with this feature.");
        }
        #[cfg(feature = "wasi-nn")]
        {
            wasmtime_wasi_nn::add_to_linker(linker, |host| host.wasi_nn.as_mut().unwrap())?;
            store.data_mut().wasi_nn = Some(WasiNnCtx::new()?);
        }
    }

    if wasi_modules.wasi_crypto {
        #[cfg(not(feature = "wasi-crypto"))]
        {
            bail!("Cannot enable wasi-crypto when the binary is not compiled with this feature.");
        }
        #[cfg(feature = "wasi-crypto")]
        {
            wasmtime_wasi_crypto::add_to_linker(linker, |host| host.wasi_crypto.as_mut().unwrap())?;
            store.data_mut().wasi_crypto = Some(WasiCryptoCtx::new());
        }
    }

    Ok(())
}

#[cfg(not(unix))]
fn ctx_set_listenfd(num_fd: usize, builder: WasiCtxBuilder) -> Result<(usize, WasiCtxBuilder)> {
    Ok((num_fd, builder))
}

#[cfg(unix)]
fn ctx_set_listenfd(num_fd: usize, builder: WasiCtxBuilder) -> Result<(usize, WasiCtxBuilder)> {
    use listenfd::ListenFd;

    let mut builder = builder;
    let mut num_fd = num_fd;

    for env in ["LISTEN_FDS", "LISTEN_FDNAMES"] {
        if let Ok(val) = std::env::var(env) {
            builder = builder.env(env, &val)?;
        }
    }

    let mut listenfd = ListenFd::from_env();

    for i in 0..listenfd.len() {
        if let Some(stdlistener) = listenfd.take_tcp_listener(i)? {
            let _ = stdlistener.set_nonblocking(true)?;
            let listener = TcpListener::from_std(stdlistener);
            builder = builder.preopened_socket((3 + i) as _, listener)?;
            num_fd = 3 + i;
        }
    }

    Ok((num_fd, builder))
}

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
    fn test6_func(fp: FuncParam);
    fn test6_func_2(add: extern "C" fn(i32, i32) -> i32);
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
struct RustFuncPointer {
    add: i32,
    my_malloc: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FuncPointer {
    add: unsafe extern "C" fn(i32, i32) -> i32,
    my_malloc: unsafe extern "C" fn(u32) -> *const c_void,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct RustFuncParam {
    fp: RustFuncPointer,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct FuncParam {
    fp: FuncPointer,
}

fn wrap_test6_func(mut caller: Caller<'_, Host>, _fp: i32) {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    //let my_realloc = caller.get_export("realloc").unwrap().into_func().unwrap().typed::<(u32, u32), u32, _>(&caller).unwrap();
    let linear_memory = memory.data_mut(&mut caller).as_mut_ptr();
    unsafe {
        let fp: *mut RustFuncParam = linear_memory.add(_fp as usize).cast();
        let fpointer: RustFuncPointer = (*fp).fp;
        let add: *mut extern "C" fn(i32, i32) -> i32 = linear_memory.add(fpointer.add as usize).cast();
        println!("call add: {}", (*add)(1, 2));
        let my_malloc: *mut extern "C" fn(u32) -> *const c_void = linear_memory.add(fpointer.my_malloc as usize).cast();
        let fp = FuncParam {fp: FuncPointer { add: *add, my_malloc: *my_malloc }};
        test6_func(fp)
    }   
}

fn wrap_test6_func_2(mut caller: Caller<'_, Host>, _add: i32) {
    // this can not run
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let linear_memory = memory.data_mut(&mut caller).as_mut_ptr();
    let table = caller.get_export("__indirect_function_table").unwrap().into_table().unwrap();
    let func = table.get(&mut caller, 1).unwrap()
                .funcref().unwrap().unwrap()
                .typed::<(i32, i32), i32, _>(&caller).unwrap();
    let res = func.call(&mut caller, (1, 2)).unwrap();
    println!("res: {}", res);
    unsafe {
        let add: *mut extern "C" fn(i32, i32) -> i32 = linear_memory.add(_add as usize).cast();
        test6_func_2(*add)
    }   
}



