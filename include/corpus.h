
#ifndef CORPUS_HEADER

#define CORPUS_HEADER 

#include <stdint.h>
#include <dirent.h>
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct cases_t {
    uint8_t* data;
    size_t size;
} FuzzCase;

typedef struct corpus_t {
    int count;
    int capacity;
    FuzzCase* cases;
} Corpus;


Corpus* new_corpus(const char* corpus_dir);
void add_to_corpus(Corpus*,FuzzCase*);


#endif