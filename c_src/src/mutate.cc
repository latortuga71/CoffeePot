#include "mutate.h"


void MutateBuffer(FuzzCase* original,FuzzCase* copy){
    //printf("original 0x%llx copy 0x%llx\n",original->data,copy->data);
    //printf("original size %d copy size %d \n",original->size,copy->size);
    memcpy(copy->data,original->data,copy->size);
    int mutationsPerCycle = 8 * copy->size / 100;
    for (int i = 0; i < mutationsPerCycle; i++){
        int rand_strat = rand() % (uint64_t)(5+0);
        int rand_bit = rand() % uint64_t(7 + 0);
        int rand_byte_insert = rand() % uint64_t(255+0);
        int rand_byte = rand() % uint64_t(copy->size - 1);
        switch (rand_strat){
            case 0: {
                copy->data[rand_byte] ^= (1  << rand_bit);
                break;
            }
            case 1: {
                copy->data[rand_byte] ^=  rand_byte_insert;
                break;
            }
            case 2: {
                copy->data[rand_byte] =  rand_byte_insert;
                break;
            }
            case 3: {
                copy->data[rand_byte] =  0x0;
                break;
            }
            default: {
                break;
            }
        }
    }
}