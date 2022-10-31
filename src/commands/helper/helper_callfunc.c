#include "helper.h"

typedef struct {
    char *name;
    int age;
} State; 

typedef struct {
    int name;
    int age;
} WasmState; 

typedef struct {
    char* (*modify_get_name)(State *s, char* md_name);
    void (*modfify_age)(State *s, int md_age);
} FuncPointer;

typedef struct {
    FuncPointer fp;
    State s;
} Modify;

typedef struct {
    int modify_get_name;
    int modfify_age;
} WasmFuncPointer;

typedef struct {
    WasmFuncPointer fp;
    WasmState s;
} WasmModify;

// == call func pointer == //
typedef void (*modify_age)(int func_offset, int state, int new_age, char* closure);
typedef int (*modify_get_name)(int func_offset, int state, int new_name, char* closure);
modify_age ma;
modify_get_name mn;
void* modify_age_closure;
void* modify_name_closure;
void register_modify_age(modify_age func, void* closure) {
    ma = func;
    modify_age_closure = closure;
}
void register_modify_name(modify_get_name func, void* closure) {
    mn = func;
    modify_name_closure = closure;
}

int modify(int op, int md_name) {
    WasmModify *md = transfer_i32_to_ptr(op);
    WasmFuncPointer fp = md->fp;
    WasmState ws = md->s;
    ma(fp.modfify_age, transfer_ptr_to_i32(&md->s), 31, modify_age_closure);
    int ret_name = mn(fp.modify_get_name, transfer_ptr_to_i32(&md->s), md_name, modify_name_closure);
    return ret_name;
}

