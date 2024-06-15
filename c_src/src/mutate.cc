#include "mutate.h"


void MutateBuffer(FuzzCase* original,FuzzCase* copy){
    //printf("original 0x%llx copy 0x%llx\n",original->data,copy->data);
    //printf("original size %d copy size %d \n",original->size,copy->size);
    memcpy(copy->data,original->data,copy->size);
    int mutationsPerCycle = 12 * copy->size / 100;
    for (int i = 0; i < mutationsPerCycle; i++){
        int rand_strat = rand() % (uint64_t)(2+0);
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
            default: {
                break;
            }
        }
    }
    // GRAMMER CONFIRM
    /*
    copy->data[0] = 'A';
    copy->data[1] = 'B';
    copy->data[2] = 'C';
    copy->data[3] = 'D';
    copy->data[4] = 'E';
    copy->data[5] = 'F';
    copy->data[6] = 'G';
    copy->data[7] = 'H';
    copy->data[8] = 'I';
    copy->data[9] = 'J';
    copy->data[10] = 'K';
    copy->data[11] = 'L';
    */
    copy->data[copy->size+1] = '\0';
}