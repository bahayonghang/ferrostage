"""Create owned, silent, three-frame codec fixtures; no third-party visual assets."""
from hashlib import sha256
from pathlib import Path
import json
from PIL import Image, ImageDraw

root = Path(__file__).resolve().parent
frames = []
for index, color in enumerate(((210, 65, 40), (40, 160, 90), (60, 95, 210))):
    frame = Image.new("RGBA", (160, 90), (*color, 255))
    draw = ImageDraw.Draw(frame)
    draw.rectangle((10 + index * 45, 25, 35 + index * 45, 65), fill=(250, 245, 225, 255))
    frames.append(frame)
receipts = []
for extension in ("gif", "webp"):
    path = root / f"synthetic-three-frames.{extension}"
    frames[0].save(path, save_all=True, append_images=frames[1:], duration=[120, 240, 360], loop=0, lossless=True)
    decoded = Image.open(path)
    dimensions = decoded.size
    delays = []
    colors = []
    for index in range(decoded.n_frames):
        decoded.seek(index)
        rgba = decoded.convert("RGBA")
        delays.append(decoded.info.get("duration"))
        colors.append(rgba.getpixel((0, 0)))
    receipts.append(dict(file=path.name, bytes=path.stat().st_size, sha256=sha256(path.read_bytes()).hexdigest(), frames=decoded.n_frames, dimensions=dimensions, encoded_delay_ms=[120,240,360], pillow_delay_ms=delays, corner_rgba=colors))
(root / "synthetic-animation-identities.json").write_text(json.dumps(receipts, indent=2), encoding="utf-8")
print(json.dumps(receipts, indent=2))
