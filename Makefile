all: program rainbow helloworld hackclub keyboard wozmon serial snake


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

keyboard:
	ca65 src/asm/keyboard/keyboard_demo.s -o build/asm/keyboard_demo.o
	ld65 \
		-C src/asm/keyboard/linker.cfg \
		build/asm/keyboard_demo.o \
		-o build/asm/keyboard_demo.bin
	rm build/asm/keyboard_demo.o

serial:
	ca65 src/asm/serial/serial_demo.s -o build/asm/serial_demo.o
	ld65 \
		-C src/asm/serial/linker.cfg \
		build/asm/serial_demo.o \
		-o build/asm/serial_demo.bin
	rm build/asm/serial_demo.o


snake:
	ca65 src/asm/snake/snake_demo.s -o build/asm/snake_demo.o
	ld65 \
		-C src/asm/snake/linker.cfg \
		build/asm/snake_demo.o \
		-o build/asm/snake_demo.bin
	rm build/asm/snake_demo.o

wozmon:
	ca65 src/asm/wozmon/wozmon.s -o build/asm/wozmon.o
	ld65 \
		-C src/asm/wozmon/linker.cfg \
		build/asm/wozmon.o \
		-o build/asm/wozmon.bin
	rm build/asm/wozmon.o

font:
	python3 tools/font_generator.py
	rm src/asm/font.inc
	mv font.inc src/asm/font.inc

aseprite:
	/Applications/Aseprite.app/Contents/MacOS/aseprite -b *.aseprite --save-as {path}/{title}.png
