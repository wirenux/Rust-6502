all: program rainbow helloworld hackclub


program:
	ca65 src/asm/program.s -o build/asm/program.o
	ld65 \
		-C src/asm/linker.cfg \
		build/asm/program.o \
		-o build/asm/program.bin
	rm build/asm/program.o

hackclub:
	ca65 src/asm/hackclub/hackclub_logo_demo.s -o build/asm/hackclub_logo_demo.o
	ld65 \
		-C src/asm/hackclub/linker.cfg \
		build/asm/hackclub_logo_demo.o \
		-o build/asm/hackclub_logo_demo.bin
	rm build/asm/hackclub_logo_demo.o

rainbow:
	ca65 src/asm/rainbow/rainbow_demo.s -o build/asm/rainbow_demo.o
	ld65 \
		-C src/asm/rainbow/linker.cfg \
		build/asm/rainbow_demo.o \
		-o build/asm/rainbow_demo.bin
	rm build/asm/rainbow_demo.o

helloworld:
	ca65 src/asm/helloworld/helloworld_demo.s -o build/asm/helloworld_demo.o
	ld65 \
		-C src/asm/helloworld/linker.cfg \
		build/asm/helloworld_demo.o \
		-o build/asm/helloworld_demo.bin
	rm build/asm/helloworld_demo.o