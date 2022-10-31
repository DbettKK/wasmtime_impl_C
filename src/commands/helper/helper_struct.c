#include "helper.h"

typedef struct {
    char *name;
    int age;
} Stu;

typedef struct {
    char *name;
    int age;
} Tea;

typedef struct {
    int name;
    int age;
} WasmStu;

typedef struct {
    WasmStu st;
    int te;
} WasmClass;

void check_struct(int c) {
    WasmClass *wc = transfer_i32_to_ptr(c);
    //printf("st: %d\n", wc->st);
    WasmStu s = wc->st;
    printf("s->name %s\n", transfer_i32_to_ptr(s.name));
    //printf("%s\n", transfer_i32_to_ptr(s->name));
    WasmStu *te = transfer_i32_to_ptr(c + 8);
    printf("te: %s\n", transfer_i32_to_ptr(te->name));
}