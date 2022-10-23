f=$1

clang --target=wasm32-wasi \
--sysroot ${WASI_SYSROOT} \
-emit-llvm -S -o ${f}.ll ${f}

llc -filetype obj ${f}.ll -o ${f}.o

wasm-ld ${f}.o \
${WASI_SYSROOT}/lib/wasm32-wasi/crt1-command.o \
-L${WASI_SYSROOT}/lib/wasm32-wasi \
-L/opt/homebrew/Cellar/llvm@13/13.0.1_2/lib/clang/13.0.1/lib/wasi/ \
-lc \
-lclang_rt.builtins-wasm32 \
-o ${f}.wasm \
--no-entry \
--allow-undefined \
--export-table

rm *.ll *.o


