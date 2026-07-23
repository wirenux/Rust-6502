import sys
from PIL import Image

PALETTE = [
    (0, 0, 0),
    (255, 255, 255),
    (255, 0, 0),
    (0, 255, 255),
    (255, 0, 255),
    (0, 255, 0),
    (0, 0, 255),
    (255, 255, 0),
    (255, 128, 0),
    (128, 64, 0),
    (255, 64, 64),
    (32, 32, 32),
    (128, 128, 128),
    (64, 64, 255),
    (64, 255, 64),
    (200, 200, 200),
]

def find_closest_color(rgb):
    r, g, b = rgb[:3]
    min_dist = float('inf')
    closest_idx = 0
    for idx, (pr, pg, pb) in enumerate(PALETTE):
        dist = (r - pr)**2 + (g- pg)**2 + (b - pb)**2
        if dist < min_dist:
            min_dist = dist
            closest_idx = idx
    return closest_idx

def convert_image(image_path, output_asm_path):
    img = Image.open(image_path).convert('RGB')
    img = img.resize((32, 32))

    pixel_bytes = []
    for y in range(32):
        for x in range(32):
            rgb = img.getpixel((x, y))
            color_idx = find_closest_color(rgb)
            pixel_bytes.append(color_idx)

    asm_template = f"""
.segment "CODE"

.segment "RODATA"

image_data:
"""
    lines = []
    for i in range(0, len(pixel_bytes), 16):
        chunk = pixel_bytes[i:i+16]
        bytes_str = ", ".join(f"${b:02X}" for b in chunk)
        lines.append(f"    .byte {bytes_str}")

        full_asm = asm_template + "\n".join(lines)

        with open(output_asm_path, "w") as f:
            f.write(full_asm)

        print(f"Succefully converted '{image_path}' -> '{output_asm_path}' !")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python img2_6502asm.py <image.png>")
    else :
        input_file = sys.argv[1]
        convert_image(input_file, "output.s")