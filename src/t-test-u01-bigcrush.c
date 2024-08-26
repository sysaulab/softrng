#include "TestU01.h"
#include <stdio.h>


unsigned int getstdin (void)
{
    unsigned int buffer;
    fread(&buffer, sizeof( unsigned int), 1, stdin);
    return buffer;
}

int main()
{
    // Create TestU01 PRNG object for our generator
    unif01_Gen* gen = unif01_CreateExternGenBits("SoftRNG unknown", getstdin);

    // Run the tests.
    bbattery_BigCrush(gen);

    // Clean up.
    unif01_DeleteExternGenBits(gen);

    return 0;
}