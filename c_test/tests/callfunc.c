#include "../pub.h"

typedef struct {
    char *name;
    int age;
} State; 

typedef struct {
    char* (*modify_get_name)(State *s, char* md_name);
    void (*modfify_age)(State *s, int md_age);
} FuncPointer;

typedef struct {
    FuncPointer fp;
    State s;
} Modify;

extern char* modify(void *op, char* md_name);
// char* modify(void* op, char* md_name) {
//     Modify* md = op;
//     md->fp.modfify_age(&md->s, 31);
//     return md->fp.modify_get_name(&md->s, md_name);
// }

char* md_name_def(State *s, char* md_name) {
    s->name = md_name;
    return md_name;
}

void md_age_def(State *s, int md_age) {
    s->age = md_age;
}

int main() {
    Modify *md = malloc(sizeof(Modify));
    State s = { .name = "Tom", .age = 13 };
    FuncPointer fp = { .modify_get_name = md_name_def, .modfify_age = md_age_def };
    md->s = s;
    md->fp = fp;

    char* new_name = "Cherry";
    char* ret_name = modify(md, new_name);

    printf("new_name: %s\n", md->s.name);
    printf("ret_name: %s\n", ret_name);
    printf("new_age: %d\n", md->s.age);
}