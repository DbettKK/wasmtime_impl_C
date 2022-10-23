/*
本用例用于测试以下情况(假设 extern 的函数名为 func ): 
    func 的入参为结构体变量(非指针), 且该结构体有一成员也同样
    为结构体变量(非指针)
结论:
    对于 WASM 而言，入参不论为结构体变量 or 指针, 都会变为指针
    所以在 Wasmtime 里处理的时候同样通过转为指针再进行处理
注意:
    因为该结构体的成员也为结构体变量(非指针), 所以在上述处理
    得到该结构体 father 的时候, 其成员 son 可以直接通过 father.son 
    得到, 而不必且不应该将 father.son 看做 u32, 然后再将其转为指针
    但由于 son 的成员中有一指针变量 name, 所以需要将 father.son.name
    看作 u32, 而后通过 linear_mem 转为指针再重新赋值
补充:
    如果结构体变量/指针作为返回值的话, 其 WASM 形式对应的函数会将返回值
    置为空, 同时增加一个 i32 类型的 param, 该 param 即需要返回的结构体
    所对应的指针
总结:
    当结构体作为入参时, 不论是指针类型还是非指针类型, 其都会被看做是指针
    但在结构体内部的成员则是什么类型还是什么类型, 不会发生变量->指针的转变
*/

#include <stdio.h>
typedef struct {
    char *name;
    int age;
} Son;

typedef struct  {
    Son son;
} Father;

extern void test3_func(Father f);

int main() {
    Son son = {
        .age = 12, .name = "Tome"
    };
    Father f = {.son = son};
    //printf("%s\n", f.son.name);
    //printf("%d\n", f.son.age);
    test3_func(f);
    return 0;
}