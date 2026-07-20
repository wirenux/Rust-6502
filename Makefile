all:
	ca65 src/asm/program.s -o build/asm/program.o
	ld65 \
		-C src/asm/linker.cfg \
		build/asm/program.o \
		-o build/asm/program.bin
	rm build/asm/program.o

rainbow:
	ca65 src/asm/rainbow/rainbow_demo.s -o build/asm/rainbow_demo.o
	ld65 \
		-C src/asm/rainbow/linker.cfg \
		build/asm/rainbow_demo.o \
		-o build/asm/rainbow_demo.bin
	rm build/asm/rainbow_demo.o