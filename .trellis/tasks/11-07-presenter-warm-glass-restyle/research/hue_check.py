"""Count green/purple chrome pixels (hue 90-300 deg, HSV saturation > 25%) in presenter screenshots.

Usage: python hue_check.py <png>... ; pass --exclude x0,y0,x1,y1 to drop a PDF region for the next file.
Pixels darker than value 12% are ignored (saturation is unstable near black).
A pixel counts only when its whole 3x3 neighbourhood matches, which drops 1px ClearType text fringes.
"""
import colorsys, sys
from PIL import Image

def count(path, boxes):
    im = Image.open(path).convert("RGB")
    w, h = im.size
    px = im.load()
    def match(x, y):
        r, g, b = (v / 255 for v in px[x, y])
        hh, s, v = colorsys.rgb_to_hsv(r, g, b)
        return v > .12 and s > .25 and 90 <= hh * 360 <= 300

    hits = total = 0
    for y in range(1, h - 1, 2):
        for x in range(1, w - 1, 2):
            if any(x0 <= x < x1 and y0 <= y < y1 for x0, y0, x1, y1 in boxes):
                continue
            total += 1
            if all(match(x + dx, y + dy) for dy in (-1, 0, 1) for dx in (-1, 0, 1)):
                hits += 1
    return hits, total

args, boxes = sys.argv[1:], []
i = 0
while i < len(args):
    if args[i] == "--exclude":
        boxes.append(tuple(int(v) for v in args[i + 1].split(",")))
        i += 2
        continue
    hits, total = count(args[i], boxes)
    print(f"{args[i]}: {hits}/{total} = {hits / total:.4%}")
    boxes = []
    i += 1
