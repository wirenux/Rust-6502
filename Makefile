all:
	ca65 src/asm/program.s -o build/asm/program.o
	ld65 \
		-C src/asm/linker.cfg \
		build/asm/program.o \
		-o build/asm/program.bin
	rm build/asm/program.o

rainbow:
	ca65 src/asm/rainbow/rainbow.s -o build/asm/rainbow.o
	ld65 \
		-C src/asm/rainbow/linker.cfg \
		build/asm/rainbow.o \
		-o build/asm/rainbow.bin
	rm build/asm/rainbow.o