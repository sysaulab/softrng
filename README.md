# SoftRNG

Softrng is a collection of basic portable tools that can be used to explore random number generators and cryptography. It comes with basic algorithms and integrates with other software (nicely named alias).

SoftRNG does not fetch and compile third party software. If you wish to use practrand or dieharder, you have to install it yourself.

Is is built following the UN*X philosophy: do one thing and do it right. For examples, generators are outputting en endless stream in binary formats. You can use filters to effect 

The tools combine together to create custom solutions or tests for the user. It requires no coding to get started and the simplicity of the commands means most of the code is small and focused on the task, reducing the level en entry for newcomers to the field. Professionnals should also enjoy it's flexibility.

## Build

SoftRNG assumes a Unix-like environment. If you are using Windows, please install Linux first. It has been developed and tested primarily under macOS.

1. Run "./build.sh" to compile the commands.
2. The programs will be created in the "Release" directory

## Install

1. Run "./install.sh" to install the commands.
2. This will move the programs to /usr/local/bin and call "softrng install" to do the initial setup.
3. "./install.sh" must run as root. Use sudo or other facilities to do so.
