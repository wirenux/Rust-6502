# PS/2 Keyboard

`rust6502` emulates a **PS/2** keyboard interface, mapped into memory so the CPU can poll scancodes the way it would talk to a real **PS/2** controller.

Keystrokes are captured from the host terminal (via `crossterm`) and translated into **PS/2 Scan Code Set 2** bytes, which are queued up for the CPU to read one at a time.

## Memory Mapping

| Adress | Name | Access | Description |
|--------|------|--------|-------------|
| `$BFF0`| `KBD_DATA` | Read | Peek at the next scancode byte in the queue (does **not** remove it) |
| `$BFF1`| `KBD_STATUS` | Read | Bit 0 (`$01`): a byte is waiting. Bit 1 (`$02`): keyboard capture is active |
| `$BFF2`| `KBD_ACK` | Write | Pop the byte that was just reed at `$BFF0` off the queue (value written is ignored) |

Because `$BFF0` only **peeks** at the front of the queue, reading it never removes the byte on its own,so you must explicitly acknowledge it via `$BFF2` once you're done with it, or you'll keep reading the same byte forever.

The standard loop is:

1. Read `$BFF1` and check Bit 0, if it's `0`, no byte is waiting yet, so keep polling.
2. Once Bit 0 is `1`, read `$BFF0` to get the scancode byte, and do whatever processing you need with it.
3. Write any value to `$BFF2`, this pops the byte you just read off the queue and exposes the next one (if any) at `$BFF0`.

## Keyboard Capture

The TUI's [Screen](./Screen.md) panel **must keyboard focus** (click-to-capture) in order to turned keystrokes into scancodes, this is what Bit 1 of `$BFF1` reflects. If capture is inactive no new bytes will be queued even though the host terminal is still receiving input elsewhere in the UI.

## Scan Code Set 2 Encoding

Each keystroke is encoded as a **tap**: a make code followed by a break code, matching how a real PS/2 keyboard reports a press-and-release.

* Regular key: `<code> F0 <code>`
* Extended key (arrows, Home/End, Insert/Delete, Page Up/Down): `E0 <code> E0 <code>`
* Shifted character (e.g. an uppercase letter or a symbol like `!`): the shift make/break codes wrap the key's own make/break sequence, `12 <code> F0 <code> F0 12` where `12` is the Left Shift scancode.

There is no separate "key down" vs "key held" state, evey keystroke from the terminal is send as one complete tap sequence, then queued in full.

### Supported Keys

* Letters `a-z`/`A-Z`, digits `0-9`, and standar punctuation/symbol characters (shifted variants are handled automatically)
* `Enter`, `Backspace`, `Tab`, `Esc`, `Space`
* Arrow keys, `Insert`, `Delete`, `Home`, `End`, `Page Up`, `Page Down` (all extended scancodes)
* `F1-F12`

Any key not in this list is ignored, nothing is queued.

## Example

```asm
.segment "CODE"

KBD_DATA     = $BFF0
KBD_STATUS   = $BFF1
KBD_ACK      = $BFF2

RESET:
    cld
    ldx #$FF
    txs

wait_key:
    lda KBD_STATUS
    and #$01   ; bit 0: data ready
    beq wait_key

    lda KBD_DATA
    ; ... do somethings with the byte loaded in the A register ...

    sta KBD_ACK
    jmp wait_key
```

> [!NOTE]
> A single physical keystroke produces *multiple* bytes.

