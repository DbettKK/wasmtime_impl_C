clang --target=wasm32-wasi \
--sysroot ${WASI_SYSROOT} \
-emit-llvm -S -o test.ll test.c

llc -filetype obj test.ll -o test.o

wasm-ld test.o \
${WASI_SYSROOT}/lib/wasm32-wasi/crt1-command.o \
-L${WASI_SYSROOT}/lib/wasm32-wasi \
-L/opt/homebrew/Cellar/llvm@13/13.0.1_2/lib/clang/13.0.1/lib/wasi/ \
-lc \
-lclang_rt.builtins-wasm32 \
-o test.wasm \
--no-entry \
--allow-undefined


