#include "helper.h"

typedef struct {
    char *name;
    int (*add)(int, int);
} FuncPointer;

typedef struct {
    int name;
    int add;
} MyFuncPointer;

void modify_fp(int fp) {
    MyFuncPointer *p = transfer_i32_to_ptr(fp);
    //printf("%d\n", p->name);
    char *name = transfer_i32_to_ptr(p->name);
    //printf("host name: %s\n", name);
    char *new_name = malloc(sizeof(char) * 4);
    new_name[0] = 'T';
    new_name[1] = 'i';
    new_name[2] = 'm';
    int new_name_offset = transfer_ptr_to_i32(new_name);
    p->name = new_name_offset;
}