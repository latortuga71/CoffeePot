#include "hash.h"

// DBJ2?
uint64_t hashstring(unsigned char* str){
    uint64_t hash = 5281;
    int c;
    while (c = *str++){
        hash = ( (hash << 5) + hash) + c;
    }
    return hash;
}