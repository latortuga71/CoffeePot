#include "mutate.h"


FuzzCase* MutateBuffer(FuzzCase* original){
    FuzzCase* mutated = (FuzzCase*)calloc(1,sizeof(FuzzCase));
    mutated->size = original->size;
    mutated->data = (uint8_t*)calloc(mutated->size,sizeof(uint8_t));
    memcpy(mutated->data,original->data,original->size);
    int mutationsPerCycle = 5 * mutated->size / 100;
    for (int i = 0; i < mutationsPerCycle; i++){
        int rand_bit = rand() % 8 + 0;
        int rand_byte = rand() % mutated->size -1 ;
        mutated->data[rand_byte] = mutated->data[rand_byte] ^ (1<< rand_bit);
    }
    return mutated;
}