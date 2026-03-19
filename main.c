#include <stdlib.h>
#include <stdio.h>

typedef struct{
    void * ptr;
    size_t len;
}Slice;
typedef struct{
    char* ptr;
    size_t len;
}Str;
extern void sys_test();
extern void sys_put_str(Str c);
#define STR(ST) (Str){.ptr = ST, .len = sizeof(ST)-1}
int _prog_start(Slice args){
    sys_put_str(STR("hi toast i love you"));
    return 0;
}