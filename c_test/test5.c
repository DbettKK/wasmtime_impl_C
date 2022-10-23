/*
本用例用于测试以下结构体作为参数的情况
typedef struct {
    DoubleList *prev, *next;
} DoubleList;
如果指针延续很长，需要一直重建下去才行
*/
#include <stdio.h>
#include <stdlib.h>

struct DoubleList {
    struct DoubleList *prev, *next;
    int val;
};
typedef struct DoubleList DoubleList;

extern void test5_func(DoubleList *d);
// void test5_func(DoubleList *d) {
//     printf("%d\n", d->val);
//     printf("%d\n", d->prev->val);
//     printf("%d\n", d->prev->prev->val);
//     printf("%d\n", d->next->val);
//     printf("%d\n", d->next->next->val);
// }

int main() {
    DoubleList *d = malloc(sizeof(DoubleList));
    d->val = 1;
    d->prev = malloc(sizeof(DoubleList));
    d->prev->val = 0;
    d->prev->prev = malloc(sizeof(DoubleList));
    d->prev->prev->val = -1;
    d->next = malloc(sizeof(DoubleList));
    d->next->val = 2;
    d->next->next = malloc(sizeof(DoubleList));
    d->next->next->val = 3;
    test5_func(d);
}