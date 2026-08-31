#!/usr/bin/env python3
"""Draws the app icon, so it can be redrawn rather than only replaced.

One image, described by distance fields and written straight out as a PNG:
a dark tile in the Write-mode background, a spark above two lines of text in
the paper colour of every other screen. `scripts/bundle.sh` turns it into
`resources/esse.icns`.

    python3 scripts/icon.py resources/esse-icon.png [size]
"""

import math
import os
import struct
import sys
import zlib

# The app's own palette (crates/esse-app/src/theme.rs).
GROUND = (0x15, 0x17, 0x1B)  # write::BACKGROUND — the screen an essay is written on
PAPER = (0xFB, 0xFA, 0xF7)  # BACKGROUND — ink on paper, the other half of the app
SPARK = (0x7A, 0xA2, 0xF7)  # write::CARET — the one bright thing in Write mode


def rounded_rect(x, y, left, top, right, bottom, radius):
    """Signed distance to a rounded rectangle: negative inside."""
    cx = min(max(x, left + radius), right - radius)
    cy = min(max(y, top + radius), bottom - radius)
    return math.hypot(x - cx, y - cy) - radius


def circle(x, y, cx, cy, radius):
    return math.hypot(x - cx, y - cy) - radius


def coverage(distance):
    """One pixel's worth of antialiasing around the edge."""
    return min(max(0.5 - distance, 0.0), 1.0)


def blend(under, over, alpha):
    return tuple(round(u + (o - u) * alpha) for u, o in zip(under, over))


def draw(size):
    """The icon as rows of RGBA bytes."""
    s = size
    # macOS leaves the corners air: the tile is inset, with the squircle-ish
    # radius Apple's own icons use.
    inset = 0.098 * s
    left, top, right, bottom = inset, inset, s - inset, s - inset
    radius = 0.2237 * (right - left)

    spark = (0.5 * s, 0.335 * s, 0.062 * s)
    # (width, centre x, centre y, thickness) — two lines of writing, the
    # second one shorter, the way a paragraph ends.
    lines = [
        (0.430 * s, 0.5 * s, 0.560 * s, 0.050 * s),
        (0.300 * s, 0.5 * s, 0.690 * s, 0.050 * s),
    ]

    rows = []
    for row in range(s):
        y = row + 0.5
        pixels = bytearray()
        for column in range(s):
            x = column + 0.5

            tile = coverage(rounded_rect(x, y, left, top, right, bottom, radius))
            if tile <= 0.0:
                pixels += bytes((0, 0, 0, 0))
                continue

            colour = GROUND
            ink = max(
                coverage(
                    rounded_rect(
                        x,
                        y,
                        cx - width / 2,
                        cy - thickness / 2,
                        cx + width / 2,
                        cy + thickness / 2,
                        thickness / 2,
                    )
                )
                for width, cx, cy, thickness in lines
            )
            if ink > 0.0:
                colour = blend(colour, PAPER, ink)

            lit = coverage(circle(x, y, *spark))
            if lit > 0.0:
                colour = blend(colour, SPARK, lit)

            pixels += bytes((*colour, round(255 * tile)))
        rows.append(bytes(pixels))
    return rows


def png(rows, size):
    raw = b"".join(b"\x00" + row for row in rows)

    def chunk(kind, body):
        piece = kind + body
        return struct.pack(">I", len(body)) + piece + struct.pack(">I", zlib.crc32(piece))

    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


# The ten images `iconutil` wants, by the names it wants them under.
ICONSET = [
    (16, "icon_16x16"),
    (32, "icon_16x16@2x"),
    (32, "icon_32x32"),
    (64, "icon_32x32@2x"),
    (128, "icon_128x128"),
    (256, "icon_128x128@2x"),
    (256, "icon_256x256"),
    (512, "icon_256x256@2x"),
    (512, "icon_512x512"),
    (1024, "icon_512x512@2x"),
]


def write(path, size):
    with open(path, "wb") as file:
        file.write(png(draw(size), size))
    print(f"{path}: {size}×{size}")


def main():
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    path = sys.argv[1]

    # A whole iconset, drawn at every size rather than scaled down to it: the
    # 16-pixel one has three shapes in it and cannot afford a soft edge.
    if path.endswith(".iconset"):
        os.makedirs(path, exist_ok=True)
        for size, name in ICONSET:
            write(os.path.join(path, f"{name}.png"), size)
        return

    write(path, int(sys.argv[2]) if len(sys.argv) > 2 else 1024)


if __name__ == "__main__":
    main()
