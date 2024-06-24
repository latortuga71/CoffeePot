# coffeepot

# RISC-V MUSL Toolchain setup
* cd ~ && git clone https://github.com/riscv-collab/riscv-gnu-toolchain.git
* sudo apt-get install autoconf automake autotools-dev curl python3 python3-pip libmpc-dev libmpfr-dev libgmp-dev gawk build-essential bison flex texinfo gperf libtool patchutils bc zlib1g-dev libexpat-dev ninja-build git cmake libglib2.0-dev libslirp-dev
* export PATH=$PATH:/home/latortuga0x71/riscv_musl/bin
* mkdir ~/riscv_musl/
* ./configure --prefix=/home/latortuga0x71/riscv_musl --with-arch=rv64imca --with-abi=lp64
* make musl
* apt-get install qemu-user-static

# GCC TOOLCHAIN INSTALL (for gdb)
* mkdir ~/riscv_gcc/
* make linux
* wget https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download/2024.04.12/riscv64-elf-ubuntu-20.04-gcc-nightly-2024.04.12-nightly.tar.gz
* tar -zxvf
* gdb is in bin
* 
# DEBUG
* qemu-riscv64-static -g 1234 /home/latortuga0x71/CoffeePot/tests/test_binaries/simple_snapshot_test
* ./riscv64-unknown-elf-gdb /home/latortuga0x71/CoffeePot/tests/test_binaries/simple_snapshot_test

# TODO





# Example
* make
* ./bin/coffeepot ./tests/test_binaries/simple_server_example
* nc 0.0.0.0 4444
* Will snapshot user data sent and fuzz until crash
