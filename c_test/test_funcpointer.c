#include "pub.h"

typedef struct {
    char *name;
    int (*add)(int, int);
} FuncPointer;

extern void modify_fp(FuncPointer *p);

int add_def(int a, int b) { return a + b; }

int main() {
    FuncPointer *fp = malloc(sizeof(FuncPointer));
    fp->name = "Old fp";
    fp->add = add_def;
    printf("old name: %s\n", fp->name);
    printf("old add: 1 + 2 = %d\n", fp->add(1, 2));

    modify_fp(fp);

    printf("new name: %s\n", fp->name);
    printf("new add: 1 + 2 = %d\n", fp->add(1, 2));
}