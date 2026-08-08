# Serial Monitor

**`rust6502`** exposes a memory-mapped serial port that bridges the emulated CPU to a TCP socket, so you can talk to the running program from any terminal with `netcat` (or a similar tool).

## How this works

When the emulator starts, it opens a TCP listener on `127.0.0.1:8080` on a new CPU thread. Any client that connects to that port becomes the "serial monitor": bytes it sends are queued up for the CPU to read, and bytes the CPU writes are queued up to be sent back to it.

## Memory Mapping

|  Adress | Name | Access | Description |
|---------|-|-|-|
| `$BFE0` | Serial Data | Read / Write | Read: pops the next received byte. Write: sends a byte out to the connected client. |
| `$BFE1` | Serial Status | Read | Bit 7 (`$80`) is set when a received byte is waiting to be read.

Only one client can be connected at a time.

## Byte Handling Rules

The serial thread applies a couple of small transformation so the port behaves nicely with a plain terminal:

* Incoming (Terminal -> CPU): any `\n` (LF) is converted to `\r` (CR) before being queued, and the byte is uppercased. This matches how the [Apple I](./AppleI.md) / [Wozmon](https://en.wikipedia.org/wiki/Apple_I) expects input.
* Outgoind (CPU -> Terminal): any `\r` (CR) written by the CPU is expanded to `\r\n` (CRLF) so it displays correctly on the client's terminal.

## Connecting with `netcat`

With the emulator running, you can connect from another terminal using this command:

```bash
nc 127.0.0.1 8080
```

Anythings you type is sent to the CPU as it reads `$BFE0`; anythings the CPU writes to `$BFE0` appears in your terminal.

## Example: Polling the Serial Port in Assembly

The following code waits for a byte to arrive and send `'A'` back in loop:

```asm
.segment "CODE"

SERIAL_DATA = $BFE0
SERIAL_STATUS = $BFE1

RESET:
    cld
    ldx #$FF
    txs

wait_byte:
    lda SERIAL_STATUS
    bpl wait_byte

    lda #'A'
    sta SERIAL_DATA
    jmp wait_byte

.segment "VECTORS"
  .word $0000
  .word RESET
  .word $0000
```

## Notes

* The port number (`8080`) is fixed. So if you have any other program running on that port you should stop this program.