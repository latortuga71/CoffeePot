#include "corpus.h"
#include <math.h>


static int get_file_count(const char* corpus_dir){
    int files = -1;
    DIR* d;
    struct dirent *dir;
    d = opendir(corpus_dir);
    if (d == NULL){
        assert("failed to open directory to read corpus" == 0);
    }
    while ( (dir = readdir(d)) != NULL){
        if (dir->d_type == DT_REG){
            //printf("-> %s :::\n",dir->d_name);
            files++;
        }
    }
    closedir(d);
    return files;
}

Corpus* new_corpus(const char* corpus_dir){
    //printf("corpus dir %s\n",corpus_dir);
    Corpus* corpus = (Corpus*)calloc(1,sizeof(Corpus));
    corpus->count = get_file_count(corpus_dir);
    corpus->capacity = 100; // keep corpus size at 100 for now since we cant realloc lol
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

            fseek(f,0,SEEK_END);
            long file_sz = ftell(f);
            rewind(f);
            char* file_buffer = (char*)calloc(file_sz,sizeof(uint8_t));
            size_t read  = fread(file_buffer,1,file_sz,f);
            fclose(f);
            assert(read == file_sz);
            file_buffer[file_sz] = '\0';
            corpus->cases[i].data = (uint8_t*)file_buffer;
            corpus->cases[i].size = file_sz;
            i++;
            //printf("%s %d Added to corpus\n",path_buffer,file_sz);
            free(path_buffer);
        }
    }
    //printf("%d Added to corpus\n",i);
    closedir(d);
    return corpus;
}

void add_to_corpus(Corpus* corpus, FuzzCase* fzz_case){
    if ((corpus->count + 1) > corpus->capacity){
        assert("TODO FIX RELLOCATION RANDOM CRASH" == 0);
        //printf("REALLOC capacity %d\n",corpus->capacity);
        //printf("REALLOC count %d\n",corpus->count);
        corpus->capacity = (int)(pow(corpus->capacity,2) + 0.5);
        void* tmp = realloc(corpus->cases,sizeof(FuzzCase) * corpus->capacity);
        if (tmp == NULL) {
            printf("REALLOC FAILED\n");
            exit(-1);
        }
        corpus->cases = (FuzzCase*)tmp;
    }
    // Write New Corpus To Out Dir So We just have track of what was added
    char path_name[250];
    snprintf(path_name,250,"./out/corpus/id_%d.bin",corpus->count);
    FILE* f =fopen(path_name,"w");
    fwrite(fzz_case->data,sizeof(uint8_t),fzz_case->size,f);
    fclose(f);
    corpus->cases[corpus->count].data = (uint8_t*)calloc(fzz_case->size,sizeof(uint8_t));
    memcpy(corpus->cases[corpus->count].data,fzz_case->data,fzz_case->size);
    corpus->cases[corpus->count].size = fzz_case->size;
    corpus->count++;
    //printf("corpus increase -> 0x%x 0x%x 0x%x 0x%x 0x%x\n",fzz_case->data[0],fzz_case->data[1],fzz_case->data[2],fzz_case->data[3],fzz_case->data[4]);
    //printf("corpus increase -> %s\n",fzz_case->data);
    //printf("corpus increase -> %c\n",fzz_case->data[11]);
}
