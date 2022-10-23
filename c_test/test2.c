/* 
本用例用于测试以下情况（假设extern的函数名为func）：
    func的入参为结构体指针，且该结构体指针有一成员类型为 void*;
    该成员在func的实现侧里会被转为有实际意义的另一个结构体指针
*/
#include <stdio.h>
#include <stdlib.h>

typedef struct {
    void* trans;
} TransTrans;

typedef struct {
    char *no_trans;
    char *name;
    int age;
} Trans;

extern void test2_func(TransTrans* tt);

int main() {
    TransTrans* tt = malloc(sizeof(TransTrans));
    tt->trans = malloc(sizeof(Trans));
    Trans* t = tt->trans;
    t->no_trans = "NoTrans";
    t->name = "Tom";
    t->age = 13;
    test2_func(tt);
}