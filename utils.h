#pragma once 
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
typedef struct {
    char * ptr;
    size_t len;
}string_t;

#define make_array_type(T, Name)\
    typedef struct{\
        T* items;\
        size_t len;\
    }Name##_slice_t;\
    typedef struct {\
        T* items;\
        size_t len;\
        size_t cap;\
        arena_t * arena;\
    }Name##_list_t;\


typedef struct allocation_t{
    void * start;
    size_t len;
    struct allocation_t * next;
} allocation_t;

typedef struct arena_t{
    unsigned char * bytes;
    unsigned char * last_ptr;
    size_t bump_offset;
    size_t length;
    struct arena_t * next;
}arena_t;

arena_t * new_arena(void);
arena_t * new_arena_sized(size_t size);
void delete_arena(arena_t * arena);
void * arena_alloc(arena_t * arena, size_t count);
void * arena_realloc(arena_t * arena,void * ptr, size_t base_count, size_t new_count);

#define STR(st) (string_t){.ptr = (st), .len = strlen((st))}
#define STR_FMT "%.*s"
#define STR_ARG(st) (int)(st).len, (st).ptr

#define new_list(arena, T) (T){.arena = arena, .items = 0,.len = 0, .cap =0}

#define list_push(list, value)\
    if((list).len<(list).cap){\
        (list).items[(list).len] = value;\
        (list).len+= 1;\
    }else{\
        if((list).cap== 0){\
            (list).items = arena_alloc((list).arena, sizeof(*(list).items)*8);\
            (list).cap = 8;\
            (list).items[0] = value;\
            (list).len+= 1;\
        }else{\
            (list).items = arena_realloc((list).arena, (list).items, (sizeof(*(list).items))*(list).cap,sizeof(*(list).items)*(list).cap*2);\
            (list).cap *= 2;\
            (list).items[(list).len] = value;\
            (list).len+=1;\
        }\
    }\

#define list_remove(list, at)\
    if((list).len>at){\
        memmove(&(list).items[at], &(list).items[at+1], (list).len-at);\
        (list).len -= 1;\
    }\

#define list_insert(list, at, value)\
    if((list).length>=at){\
        memmove(&(list).items[at+1], &(list).items[at], (list).len-at);\
        (list).items[at] = value;\
        (list).len += 1;\
    }\

#define list_resize(list, new_size)\
    {(list).items = arena_realloc((list).arena, list.items, (sizeof(*(list).items))*(list).cap,sizeof(*(list).items)*new_size); (list).cap =new_size; }
make_array_type(uint8_t, u8)
make_array_type(uint16_t, u16)
make_array_type(uint32_t, u32)
make_array_type(uint64_t, u64)

make_array_type(int8_t, i8)
make_array_type(int16_t, i16)
make_array_type(int32_t, i32)
make_array_type(int64_t, i64)
make_array_type(float, float)
make_array_type(double, double)
make_array_type(bool, bool)
make_array_type(string_t, string)

string_t arena_fmt(arena_t * arena, const char * format, ...);
string_list_t split_string_by(arena_t * arena, string_t s, const char delimator);
u8_list_t read_file_to_bytes(arena_t * arena, char * file_name);
string_t read_file_to_string(arena_t * arena, char * file_name);

typedef enum {
    TOKEN_TYPE_LITERAL, 
    TOKEN_TYPE_IDENTIFIER,
    TOKEN_TYPE_OPERATOR,
    TOKEN_TYPE_SEPERATOR, 
    TOKEN_TYPE_OPEN_DELIM, 
    TOKEN_TYPE_CLOSE_DELIM,
    TOKEN_TYPE_TOKEN_OPERATOR,
}token_type_t;

typedef struct {
    token_type_t token_type;
    int32_t line;
    string_t source_file;
    string_t contents;
}token_t;
#define SLICE(T,LIST...) {.items = (T[]){LIST}, .len = sizeof((T[]){LIST})}

make_array_type(token_t, token);
make_array_type(char,char);
token_list_t tokenize_string(arena_t *arena,string_t file_name, string_t contents, string_list_t operators);

typedef char_list_t string_builder_t;
string_t string_builder_take(string_builder_t s);