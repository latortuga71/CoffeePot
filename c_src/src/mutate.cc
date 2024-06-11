#include "mutate.h"


void MutateBuffer(FuzzCase* original,FuzzCase* copy){
    memcpy(copy->data,original->data,copy->size);
    int mutationsPerCycle = 5 * copy->size / 100;
    for (int i = 0; i < mutationsPerCycle; i++){
        int rand_bit = rand() % uint64_t(7 + 0);
        int rand_byte = rand() % uint64_t(copy->size - 1);
        copy->data[rand_byte] = original->data[rand_byte] ^ (1 << rand_bit);
    }
}