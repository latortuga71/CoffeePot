#include "corpus.h"


static int get_file_count(const char* corpus_dir){
    int files = 0;
    DIR* d;
    struct dirent *dir;
    d = opendir(corpus_dir);
    if (d == NULL){
        assert("failed to open directory to read corpus" == 0);
    }
    while ( (dir = readdir(d)) != NULL){
        if (dir->d_type == DT_REG){
            files++;
        }
    }
    closedir(d);
    return files;
}

Corpus* new_corpus(const char* corpus_dir){
    printf("corpus dir %s\n",corpus_dir);
    Corpus* corpus = (Corpus*)calloc(1,sizeof(Corpus));
    corpus->count = get_file_count(corpus_dir);
    corpus->capacity = corpus->count * 2;
    corpus->cases = (FuzzCase*)calloc(corpus->capacity,sizeof(FuzzCase));
    int i = 0;
    DIR* d;
    struct dirent *dir;
    d = opendir(corpus_dir);
    if (d == NULL){
        assert("failed to open directory to read corpus" == 0);
    }
    while ( (dir = readdir(d)) != NULL){
        if (dir->d_type == DT_REG){
            int sz = snprintf(NULL,0,"%s/%s",corpus_dir,dir->d_name) + 1;
            char* path_buffer = (char*)calloc(1,sz);
            snprintf(path_buffer,sz,"%s/%s",corpus_dir,dir->d_name);
            FILE* f = fopen(path_buffer,"r");
            if (f == NULL){
                assert("failed to open corpus file" == 0);
            }
            free(path_buffer);
            fseek(f,0,SEEK_END);
            long file_sz = ftell(f);
            rewind(f);
            char* file_buffer = (char*)calloc(file_sz,sizeof(char));
            size_t read  = fread(file_buffer,1,file_sz,f);
            fclose(f);
            assert(read == file_sz);
            file_buffer[file_sz] = '\0';
            corpus->cases[i].data = (uint8_t*)file_buffer;
            corpus->cases[i].size = file_sz;
            i++;
        }
    }
    closedir(d);
    return corpus;
}

void add_to_corpus(Corpus* corpus, FuzzCase* fzz_case){
    if (corpus->capacity + 1 > corpus->count){
        printf("REALLOC CORPUS TODO\n");
        exit(1);
    }
    corpus->cases[corpus->count].data = fzz_case->data;
    corpus->cases[corpus->count].size = fzz_case->size;
    corpus->count++;
}
