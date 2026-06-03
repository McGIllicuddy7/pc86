#include "utils.h"
#include <string.h>
#include <stdarg.h>
#include <stdio.h>
arena_t * new_arena(void){
    const size_t BYTE_COUNT = 4096*4096;
    arena_t * out = malloc(sizeof(arena_t) +(64-64%sizeof(arena_t))+BYTE_COUNT);
    out->bytes = (unsigned char *)out+(sizeof(arena_t) +(64-64%sizeof(arena_t)));
    out->last_ptr = 0;
    out->bump_offset = 0;
    out->length = BYTE_COUNT;
    out->next = 0;
    return out;
}
void delete_arena(arena_t * arena){
    if(arena->next){
        delete_arena(arena->next);
    }
    free(arena);
}
void * arena_alloc(arena_t * arena, size_t count){
    void * out;
    size_t min_count;
    if(count%16 != 0){
        count += 16-count%16;
    }
    if (arena->bump_offset+count>= arena->length){
        if (arena->next){
            return arena_alloc(arena->next, count);
        }else{
            min_count = arena->length;
            while(min_count<count){
                min_count *= 2;
            }         
            arena->next = new_arena_sized(min_count);
            return arena_alloc(arena->next, count);
        }
    }
    out = arena->bytes+arena->bump_offset;
    arena->bump_offset += count;
    arena->last_ptr = out;
    return out;
}
void * arena_realloc(arena_t * arena,void * ptr, size_t base_count, size_t new_count){
    if(!ptr){
        return arena_alloc(arena,new_count);
    }
    unsigned char * byte_ptr = ptr;
    if(arena->last_ptr == ptr){
        size_t offset = ((size_t)byte_ptr-(size_t)arena->bytes);
        if(offset+new_count< arena->length){
            arena->bump_offset = offset;
            return arena_alloc(arena, new_count);
        }
    }
    void *out = arena_alloc(arena, new_count);
    memcpy(out, ptr,base_count);
    return out;
}
arena_t * new_arena_sized(size_t size){
    const size_t BYTE_COUNT = size;
    arena_t * out = malloc(sizeof(arena_t) +(64-64%sizeof(arena_t))+BYTE_COUNT);
    out->bytes = (unsigned char *)out+(sizeof(arena_t) +(64-64%sizeof(arena_t)));
    out->last_ptr = 0;
    out->bump_offset = 0;
    out->length = BYTE_COUNT;
    out->next = 0;
    return out;
}

string_t arena_fmt(arena_t * arena, const char * format, ...){
    va_list args;
    va_start (args, format);
    size_t count = vsnprintf(0, 0, format, args);
    va_end(args);
    char * buffer = arena_alloc(arena, count+1);
    va_start (args, format);
    vsnprintf(buffer, count+1, format,args);
    printf("%zu\n", count);
    va_end(args);
    return (string_t){.ptr = buffer, .len = count};
}
