/*
本用例用于测试函数指针作为结构体成员且结构体作为入参的情况
注意：函数指针作为参数时，WASM 表示仍然为 i32
*/
#include <stdio.h>
#include <stdlib.h>

typedef struct {
    int (*add)(int, int);
    void* (*my_malloc)(size_t);
} FuncPointer;

typedef struct {
    FuncPointer fp;
} FuncParam;

int add_def(int a, int b) { return a + b; }
void *my_malloc_def(size_t size) { return malloc(size); }

extern void test6_func(FuncParam fp);
// void test6_func(FuncParam fp) {
//     printf("%d\n", fp.fp.add(1, 2));
//     int *p = fp.fp.my_malloc(6);
//     printf("%d\n", p[1]);
// }

extern void test6_func_2(int (*add)(int a, int b));

int main() {
    // FuncParam fp;
    // fp.fp.add = add_def;
    // fp.fp.my_malloc = my_malloc_def;
    // test6_func(fp);
    test6_func_2(add_def);
    return 0;
}