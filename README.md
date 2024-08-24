# SoftRNG

Softrng is a collection of basic portable tools that can be used to explore random number generators and cryptography. It comes with basic algorithms and integrates with other software (nicely named alias).

SoftRNG does not fetch and compile third party software. If you wish to use practrand or dieharder, you will have to install it yourself.

Is is built following the UN*X philosophy: do one thing only and do it right. 

The tools combine together to create custom solutions or tests for the user. It requires no coding to get started and the simplicity of the commands means most of the code is small and focused on the task, reducing the level en entry for newcomers to the field. Professionnals should also enjoy it's flexibility.

# Installation

1. Run "./build.sh" to compile the commands.
2. Run "cat ./install.sh" to look at the installation script. It needs to run with elevated privileges, it is your responsibility to ensure it is not doing anything malicious before running it.
3. Run "./install.sh" as root.
