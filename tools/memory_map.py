import matplotlib.pyplot as plt

memory_map = [
    ("Zero Page", 1.5, "#ffffcc", "0x0000", "0x00FF"),
    ("CPU Stack", 1.5, "#29abe2", "0x0100", "0x01FF"),
    ("Screen\nMemory", 2.5, "#f7931e", "0x0200", "0x05FF"),
    ("General RAM\n(Free Space)", 5.0, "#999999", "0x0600", "0xBFDF"),
    ("Serial I/O", 1, "#f15a24", "0xBFE0", "0xBFEF"),
    ("PS/2 KBD", 1, "#f7931e", "0xBFF0", "0xBFFF"),
    ("ROM Space", 3.0, "#fcee21", "0xC000", "0xFFF9"),
    ("Vectors", .8, "#8cc63f", "0xFFFA", "0xFFFF")
]

fig, ax = plt.subplots(figsize=(16, 4))
ax.axis('off')

current_x = 0

for label, width, color, left_addr, right_addr in memory_map:
    rect = plt.Rectangle((current_x, 0.4), width, 0.4, facecolor=color, edgecolor='black', linewidth=1.5)
    ax.add_patch(rect)

    ax.text(current_x + width/2, 0.6, label, ha='center', va='center', fontsize=10, family='sans-serif', fontweight='bold')

    if current_x == 0:
        ax.text(current_x, 0.35, left_addr, ha='right', va='top', fontsize=11, family='monospace', rotation=35)

    ax.text(current_x + width, 0.35, right_addr, ha='right', va='top', fontsize=11, family='monospace', rotation=35)

    current_x += width

plt.xlim(-1, current_x + 1)
plt.ylim(0, 1)
plt.tight_layout()

filename = '6502_memory_map_horizontal.png'
plt.savefig(filename, dpi=300, bbox_inches='tight')
print(f"Diagram successfully saved to {filename}")