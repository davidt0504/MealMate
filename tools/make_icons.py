#!/usr/bin/env python3
"""Generate the Android launcher icons from the brand art.

Run after changing `docs/brand/kimatta-app-icon.png`. Pillow is the only dependency and is
deliberately not a project dependency -- this is a one-off generator, not part of the build:

    python3 -m venv /tmp/icons && /tmp/icons/bin/pip install Pillow
    /tmp/icons/bin/python tools/make_icons.py

Why the source cannot be used directly. The art is a rounded navy tile with the glyph drawn on
it, saved as RGB with no alpha, so its corners are opaque white rather than transparent. Handed
to Android as an adaptive foreground it would show white triangles over the background layer.
The glyph also fills 68% of the canvas, and Android only guarantees the middle 66/108 of an
adaptive icon survives masking -- a circular mask would clip the roof and the bowl.

So two things happen here: the tile is cut out of its white surround (giving real transparency),
and the whole tile is scaled down until the *glyph* fits the guaranteed-safe zone. The adaptive
background is the tile's own navy, so the tile edge sits on identical colour and is invisible;
what the launcher masks is effectively a full-bleed navy icon with a correctly inset glyph.
"""

import sys
import xml.etree.ElementTree as ET
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "docs" / "brand" / "kimatta-app-icon.png"
RES = ROOT / "android" / "app" / "src" / "main" / "res"

# Sampled from the source, not guessed. Also written into colors.xml as the adaptive background.
NAVY = (0x0B, 0x21, 0x3D)
NAVY_TOLERANCE = 45

# Android adaptive icons are a 108dp canvas of which the middle 66dp is guaranteed to survive
# every launcher's mask shape. Legacy icons are unmasked, so they use the plain 48dp buckets.
SAFE_FRACTION = 66 / 108
LEGACY_DP = {"mdpi": 48, "hdpi": 72, "xhdpi": 96, "xxhdpi": 144, "xxxhdpi": 192}
ADAPTIVE_DP = {"mdpi": 108, "hdpi": 162, "xhdpi": 216, "xxhdpi": 324, "xxxhdpi": 432}


def is_navy(pixel):
    """True for the tile's background colour, within tolerance for JPEG-ish gradients."""
    return all(abs(pixel[i] - NAVY[i]) < NAVY_TOLERANCE for i in range(3))


def cut_out_tile(image):
    """Return the rounded tile as RGBA, everything outside it fully transparent.

    Colour alone cannot separate inside from outside: the glyph is white and so is the surround.
    The tile is solid navy at both ends of every row it occupies, though -- the glyph never
    reaches the edge -- so each row's first and last navy pixel bound the tile on that row, and
    that bound holds through the rounded corners where the span simply gets shorter.
    """
    width, height = image.size
    pixels = image.load()
    out = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    out_px = out.load()
    rows = 0
    for y in range(height):
        navy_xs = [x for x in range(width) if is_navy(pixels[x, y])]
        if len(navy_xs) < 50:  # above or below the tile, or a sliver of corner
            continue
        rows += 1
        for x in range(navy_xs[0], navy_xs[-1] + 1):
            r, g, b = pixels[x, y]
            out_px[x, y] = (r, g, b, 255)
    if rows == 0:
        sys.exit("no navy tile found in the source -- has the art changed?")
    return out


def glyph_bbox(image):
    """Bounding box of the artwork drawn on the tile, ignoring the tile itself."""
    width, height = image.size
    pixels = image.load()
    min_x, min_y, max_x, max_y = width, height, 0, 0
    for y in range(height):
        navy_xs = [x for x in range(width) if is_navy(pixels[x, y])]
        if len(navy_xs) < 50:
            continue
        # Inset past the tile's own anti-aliased edge, which is neither navy nor glyph.
        for x in range(navy_xs[0] + 8, navy_xs[-1] - 8):
            if not is_navy(pixels[x, y]):
                min_x, max_x = min(min_x, x), max(max_x, x)
                min_y, max_y = min(min_y, y), max(max_y, y)
    return min_x, min_y, max_x, max_y


def write(image, folder, name, size):
    target = RES / folder
    target.mkdir(parents=True, exist_ok=True)
    image.resize((size, size), Image.LANCZOS).save(target / name)


def main():
    if not SOURCE.exists():
        sys.exit(f"missing source art: {SOURCE}")
    source = Image.open(SOURCE).convert("RGB")
    width, _ = source.size

    tile = cut_out_tile(source)
    x0, y0, x1, y1 = glyph_bbox(source)
    glyph = max(x1 - x0, y1 - y0)
    # Scale the tile so the glyph — not the tile — lands inside the guaranteed-safe zone.
    scale = SAFE_FRACTION * (width / glyph)
    print(f"glyph {x1 - x0}x{y1 - y0}px of {width}px canvas; tile scaled to {scale:.1%}")

    for bucket, dp in LEGACY_DP.items():
        write(tile, f"mipmap-{bucket}", "ic_launcher.png", dp)

    for bucket, dp in ADAPTIVE_DP.items():
        canvas = Image.new("RGBA", (dp, dp), (0, 0, 0, 0))
        inner = max(1, round(dp * scale))
        canvas.paste(tile.resize((inner, inner), Image.LANCZOS), ((dp - inner) // 2,) * 2)
        write(canvas, f"mipmap-{bucket}", "ic_launcher_foreground.png", dp)

    anydpi = RES / "mipmap-anydpi-v26"
    anydpi.mkdir(parents=True, exist_ok=True)
    icon = ET.Element("adaptive-icon")
    icon.set("xmlns:android", "http://schemas.android.com/apk/res/android")
    ET.SubElement(icon, "background").set("android:drawable", "@color/ic_launcher_background")
    ET.SubElement(icon, "foreground").set("android:drawable", "@mipmap/ic_launcher_foreground")
    ET.indent(icon, space="    ")
    (anydpi / "ic_launcher.xml").write_text(
        '<?xml version="1.0" encoding="utf-8"?>\n' + ET.tostring(icon, encoding="unicode") + "\n",
        encoding="utf-8",
    )

    colors = ET.Element("resources")
    colour = ET.SubElement(colors, "color")
    colour.set("name", "ic_launcher_background")
    colour.text = "#{:02X}{:02X}{:02X}".format(*NAVY)
    ET.indent(colors, space="    ")
    (RES / "values" / "ic_launcher_background.xml").write_text(
        '<?xml version="1.0" encoding="utf-8"?>\n' + ET.tostring(colors, encoding="unicode") + "\n",
        encoding="utf-8",
    )
    print("wrote legacy + adaptive icons and the background colour")


if __name__ == "__main__":
    main()
