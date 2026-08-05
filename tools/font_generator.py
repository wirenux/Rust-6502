import sys
import os
from PIL import Image

PALETTE = [
    (0, 0, 0), (255, 255, 255), (255, 0, 0), (0, 255, 255),
    (255, 0, 255), (0, 255, 0), (0, 0, 255), (255, 255, 0),
    (255, 128, 0), (128, 64, 0), (255, 64, 64), (32, 32, 32),
    (128, 128, 128), (64, 64, 255), (64, 255, 64), (200, 200, 200),
]

def find_closest_color(rgb):
    r, g, b = rgb[:3]
    min_dist = float('inf')
    closest_idx = 0
    for idx, (pr, pg, pb) in enumerate(PALETTE):
        dist = (r - pr)**2 + (g - pg)**2 + (b - pb)**2
        if dist < min_dist:
            min_dist = dist
            closest_idx = idx
    return closest_idx

def process_image(path):
    img = Image.open(path).convert('RGBA')
    if img.size != (16, 16):
        img = img.resize((16, 16), Image.NEAREST)

    bg = Image.new('RGBA', (16, 16), (0, 0, 0, 255))
    bg.paste(img, (0, 0), img)
    final_img = bg.convert('RGB')

    LUMA_THRESHOLD = 96

    pixels = []
    for y in range(16):
        for x in range(16):
            r, g, b = final_img.getpixel((x, y))
            luminance = 0.299 * r + 0.587 * g + 0.114 * b
            pixels.append(1 if luminance > LUMA_THRESHOLD else 0)
    
    return pixels

SCANCODES = {
    0x0E: ('backtick', 'tilde'),
    0x15: ('q', 'Q'), 0x16: ('1', 'exclamation'), 0x1A: ('z', 'Z'), 0x1B: ('s', 'S'),
    0x1C: ('a', 'A'), 0x1D: ('w', 'W'), 0x1E: ('2', 'at'), 0x21: ('c', 'C'),
    0x22: ('x', 'X'), 0x23: ('d', 'D'), 0x24: ('e', 'E'), 0x25: ('4', 'dollar'),
    0x26: ('3', 'hash'), 0x29: ('space', 'space'), 0x2A: ('v', 'V'), 0x2B: ('f', 'F'),
    0x2C: ('t', 'T'), 0x2D: ('r', 'R'), 0x2E: ('5', 'percentage'), 0x31: ('n', 'N'),
    0x32: ('b', 'B'), 0x33: ('h', 'H'), 0x34: ('g', 'G'), 0x35: ('y', 'Y'),
    0x36: ('6', 'hat'), 0x3A: ('m', 'M'), 0x3B: ('j', 'J'), 0x3C: ('u', 'U'),
    0x3D: ('7', 'and'), 0x3E: ('8', 'asterix'), 0x41: ('comma', 'lessthan'), 0x42: ('k', 'K'),
    0x4C: ('semicolon', "2points"),
    0x43: ('i', 'I'), 0x44: ('o', 'O'), 0x45: ('0', 'rightpar'), 0x46: ('9', 'leftpar'),
    0x49: ('dot', 'greaterthan'), 0x4A: ('slash', 'question'), 0x4B: ('l', 'L'), 0x4D: ('p', 'P'),
    0x4E: ('minus', 'underscore'), 0x52: ('quote2', 'quote'), 0x54: ('leftbracket', 'leftparenthesis'), 0x55: ('equal', 'plus'),
    0x5B: ('rightbracket', 'rightparenthesis'), 0x5D: ('backslash', 'pipe'),
    0x75: ('up', 'up'), 0x72: ('down', 'down'), 0x74: ('right', 'right'), 0x6B: ('left', 'left'),
}

def sanitize_name(name):
    return name.replace('(', 'lparen').replace(')', 'rparen').replace('<', 'lt').replace('>', 'gt')

def get_image_label(category, name):
    safe_name = sanitize_name(name)
    return f"img_{category}_{safe_name.lower()}"

def find_image_file(font_dir, name, is_shifted):
    name_lower = name.lower()
    categories = ["cap", "min", "number", "other", "symbol"] if is_shifted else ["min", "cap", "number", "other", "symbol"]
    for category in categories:
        path = os.path.join(font_dir, category, f"{name_lower}.png")
        if os.path.exists(path):
            return category, path
    return None, None

def generate_asm(font_dir, output_file):
    with open(output_file, "w") as f:
        f.write(".segment \"RODATA\"\n\n")
        
        for category in ["cap", "min", "number", "other", "symbol"]:
            cat_dir = os.path.join(font_dir, category)
            if not os.path.exists(cat_dir): continue
            
            for filename in sorted(os.listdir(cat_dir)):
                if filename.endswith(".png"):
                    name = os.path.splitext(filename)[0]
                    label = get_image_label(category, name)
                    
                    f.write(f"{label}:\n")
                    pixels = process_image(os.path.join(cat_dir, filename))
                    for i in range(0, 256, 16):
                        chunk = pixels[i:i+16]
                        f.write("    .byte " + ", ".join(f"${b:02X}" for b in chunk) + "\n")
                    f.write("\n")
        
        def write_table(table_name, shift=False):
            f.write(f"{table_name}:\n")
            for i in range(0, 256, 16):
                line = []
                for j in range(16):
                    sc = i + j
                    if sc in SCANCODES:
                        name = SCANCODES[sc][1] if shift else SCANCODES[sc][0]
                        cat, _ = find_image_file(font_dir, name, is_shifted=shift)
                        if cat:
                            prefix = ">" if table_name.endswith("_hi") else "<"
                            line.append(f"{prefix}{get_image_label(cat, name)}")
                        else:
                            line.append("$00")
                    else:
                        line.append("$00")
                f.write("    .byte " + ", ".join(line) + "\n")
            f.write("\n")

        write_table("table_lo", shift=False)
        write_table("table_hi", shift=False)
        write_table("table_shift_lo", shift=True)
        write_table("table_shift_hi", shift=True)
            
    print(f"Generated {output_file}")

if __name__ == "__main__":
    generate_asm("assets/font", "font.inc")
