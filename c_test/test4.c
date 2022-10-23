/*
本用例用于测试入参结构体内存在零长度数组的情况
    暂时还没什么好办法
*/
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char *name;
    int age;
    void *buf[0];
} ChangeLen;

extern void test4_func(ChangeLen *c);

int main() {
    ChangeLen* c = malloc(sizeof(ChangeLen) + sizeof(char) * 7);
    c->name = "Tom";
    c->age = 12;
    for (int i = 0; i < 4; i++){
        c->buf[i] = 'i';
    }
    //c->buf = "Jack";
    test4_func(c);
}