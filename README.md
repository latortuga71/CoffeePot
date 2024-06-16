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
* export PATH=$PATH:/home/latortuga0x71/riscv_gcc/bin
* mkdir ~/riscv_gcc/
* ./configure --prefix=/home/latortuga0x71/riscv_gcc --with-arch=rv64gc
* make linux

# TODO
