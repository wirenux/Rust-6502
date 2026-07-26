# Screen & Video managements Documentation

Rust-6502 features a memory-mapped virtual screen that update in real-time as your program write data to specific RAM addresses.

## How this work ?

The 32x32 screen is rendered using the Unicode half block character `▀` and by changing his foreground color and background color to optain 2 pixel with there own color.

## Memory Mapping

* **Memory Footprint**: `$0200` to `$05FF` (1024 bytes)
* **Screen Resolution**: 32 x 32 pixels (1024 total pixels)

The screen is divided in 4 due to the 8-bit limitation. So each quarter contain 8 lines and 256 pixels.

* 1st Quarter: `$0200-$02FF`
* 2nd Quarter: `$0300-$03FF`
* 4rd Quarter: `$0400-$04FF`
* 4th Quarter: `$0500-$05FF`

Every single byte in this memory range corresponds directly to a single pixel on the screen.

### Pixel Coordinate Formula

To calculate the exact memory address for a specific pixel coordinate $(X, Y)$ where both $X$ and $Y$ range from `0` to `31`:

$$\text{Address} = \$0200 + (Y \times 32) + X$$

* $Y \times 32$: Moves down by whole rows (each row is 32 bytes wide).
* $+ X$: Moves horizontally across the row to the target column.

---

## Color Mapping

The value is storred at a pixel's memory address acts as an index into the emulator's color palette (ranging from `0` to `15`).

For example, writting `2` to a memory address colors that pixel <span style="color: rgb(255, 0, 0); font-size: 1.2em;">■</span> **Red**.

For a complete breakdown of all available color index values, refer to the [Color List](./Palette.md).

## Example: Drawing a Pixel in Assembly

The following 6502 assembly snippet demonstrates how to draw a <span style="color: rgb(255, 0, 0); font-size: 1.2em;">■</span> **Red** pixel right in the center of the screen ($X = 16, Y = 16$, which translate to memory address `$0410`):

```asm
LDA #$02    ; load color index 2 (Red) in the Accumulator
STA $0410   ; store the color at the center pixel address
            ; ($0200 + (16 * 32) + 16)
BRK         ; halt the CPU
```
