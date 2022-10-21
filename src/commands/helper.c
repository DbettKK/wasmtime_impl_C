#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct WasmtimeCtx WasmtimeCtx;

typedef void* (*my_realloc)(void*, size_t, WasmtimeCtx*);
typedef void* (*my_malloc)(size_t, WasmtimeCtx*);

my_realloc my;
my_malloc my_m;
WasmtimeCtx* glob_ctx;

void register_my_realloc(my_realloc re, WasmtimeCtx * ctx) {
    my = re;
    glob_ctx = ctx;
}

void register_my_malloc(my_malloc re, WasmtimeCtx * ctx) {
    my_m = re;
    glob_ctx = ctx;
}

void* realloc_wasm(void *ptr, size_t size) {
    return my(ptr, size, glob_ctx);
}

void* malloc_wasm(size_t size) {
    return my_m(size, glob_ctx);
}

#define realloc(ptr, size) realloc_wasm(ptr, size)
#define malloc(size) malloc_wasm(size)

int* test_func(int *ptr, int size) {
    printf("host: old_ptr: %p\n", ptr);
    int *new_ptr;
    //new_ptr = realloc(ptr, size);
    new_ptr = malloc(size);
    printf("host: new_ptr: %p\n", new_ptr);
    ptr = new_ptr;
    printf("%d\n", ptr[1]);
    return ptr;
}