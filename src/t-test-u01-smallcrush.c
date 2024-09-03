#include "TestU01.h"
#include <stdio.h>


unsigned int getstdin (void)
{
    unsigned int buffer;
    fread(&buffer, sizeof( unsigned int), 1, stdin);
    return buffer;
}

int main(void)
{
    // Create TestU01 PRNG object for our generator
    unif01_Gen* gen = unif01_CreateExternGenBits("SoftRNG stdin", getstdin);

    // Run the tests.
    bbattery_SmallCrush(gen);

    // Clean up.
    unif01_DeleteExternGenBits(gen);

    return 0;
}


