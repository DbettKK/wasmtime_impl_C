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

// ========= test2 ============ //

typedef struct {
    void* trans;
} TransTrans;

typedef struct {
    char *no_trans;
    char *name;
    int age;
} Trans;

void func2(void *trans) {
    Trans* t = trans;
    printf("host name: %s\n", t->name);
    printf("host age: %d\n", t->age);
}

void test2_func(TransTrans *tt) {
    func2(tt->trans);
}

// ========= test3 ============ //
typedef struct {
    char *name;
    int age;
} Son;

typedef struct  {
    Son son;
} Father;

void test3_func(Father f) {
    printf("name: %s\n", f.son.name);
    printf("age: %d\n", f.son.age);
}

// ======= test4 ======= // 
typedef struct {
    char *name;
    int age;
    void *buf[0];
} ChangeLen;

void test4_func(ChangeLen *c) {
    printf("name: %s\n", c->name);
    printf("age: %d\n", c->age);
    for (int i = 0; i < 4; i++) {
        char m = c->buf[i];
        printf("buffer: %c\n", m);
    }
}

// ==== test5 ==== //
struct DoubleList {
    struct DoubleList *prev, *next;
    int val;
};
typedef struct DoubleList DoubleList;

void test5_func(DoubleList *d) {
    printf("%d\n", d->val);
    printf("%d\n", d->prev->val);
    printf("%d\n", d->next->val);
    printf("%d\n", d->prev->prev->val);
    printf("%d\n", d->next->next->val);
}

// ==== test6 ==== //
typedef struct {
    int i[32];
    char *name;
} Name;

typedef struct {
    int (*add)(int, int, void*);
    void* add_closre;
    void* (*my_malloc)(size_t, void*);
    void* my_malloc_closure;
} FuncPointer;

typedef struct {
    Name name;
    FuncPointer fp;
} FuncParam;

void test6_func(void *op) {
    FuncParam *fp = op;
    printf("%d\n", fp->fp.add(1, 2, fp->fp.add_closre));
    int *p = fp->fp.my_malloc(6, fp->fp.my_malloc_closure);
    printf("%d\n", p[1]);
}

typedef int (*callback)(int, int, void*);

void test6_func_2(callback c, void* closure) {
    printf("%d\n", c(1, 5, closure));
}