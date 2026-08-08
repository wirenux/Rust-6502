# Apple I Documentation

`rust6502` can run a modified version of [Wozmon](https://en.wikipedia.org/wiki/Apple_I), the monitor Steve Wozniak wrote for the Apple I. This let you Read, Write and Run raw 6502 program without compling it before.

## What did I modified ?

I have added a function to handle backspace.

I also changed address for the [Display](Serial.md) and the [Keyboard](./Keyboard.md) to fit this emulator

## Loading the Wozmon

To load the [Wozmon](https://en.wikipedia.org/wiki/Apple_I) program you can do it via the TUI :

```bash
rust6502
```

> The start address will automatically be loaded, so you just need to press Start

Or you can run it using command arguments :

```bash
rust6502 wozmon.bin 0xFE80
```

Once running, Wozmon prints its `\` prompt and waits for input via the [Serial Monitor](Serial.md).

## Wozmon Command

Wozmon understand 4 commands :

| Syntax | Action |
|-|-|
| `XXXX` | Read the byte present at address `XXXX` |
| `XXXX.YYYY` | Read a range of byte from address `XXXX` to address `YYYY` |
| `XXXX: BB BB BB BB` | Write bytes `BB BB BB BB` to address `XXXX` |
| `XXXXR` | Run the program starting at address `XXXX` |

> [!NOTE]
> `XXXX` is an Hexadecimal address
> `BB` is an Hexadecimal value

## Using Wozmon over the [Serial Monitor](Serial.md)

Because Wozmon reads and writes a byte at a time, it work nicelly over the [Serial Monitor](Serial.md).

To use it you can use `netcat` with this command:

```bash
nc 127.0.0.1 8080
```

You should see the `\` prompt !

## Demo program

The Apple I manual provide a short demo program to enter by hand to prove that the monitor works.

*This demo is a modified version because the address are differents*

Here is the program you have to enter by hand :

This write the program in RAM at `0x0000`:
```
0: A9 0 AA 20 86 FF E8 8A 4C 2 0
```

This read the 1st and last byte to be sure we enter it correctly:

```
0.A
```

Then this is to run the program :

```
0R
```

You should see:

```
123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~
```