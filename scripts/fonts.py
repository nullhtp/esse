#!/usr/bin/env python3
"""Cut the five static faces esse ships from Literata's variable fonts.

The app embeds static instances rather than the variable font itself: gpui
loads fonts through font-kit's in-memory source, which groups faces by the
family name Core Text reports and then picks one by weight and slant. It does
not instance variable axes, so a variable file would give one weight and let
the rasterizer fake the rest.

Every face therefore reports the family "Literata" and carries its weight in
OS/2. The typographic-family names (16/17) are removed so that Core Text has
exactly one answer for the family, and `fonts::faces` in the app can ask for
one family at three weights.

    pip install fonttools
    python3 scripts/fonts.py path/to/Literata[opsz,wght].ttf \\
                             path/to/Literata-Italic[opsz,wght].ttf

The upstream files are the ones at github.com/google/fonts/tree/main/ofl/literata
(OFL 1.1; the licence travels with the faces in resources/fonts/OFL.txt).
"""

import os
import sys

from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

# One optical size for the whole app. Literata's opsz runs 7-72; 16 sits on the
# body text (17-20 px), which is nearly everything esse draws, and carries the
# 11 px labels and the headings either side of it without a second family.
OPSZ = 16.0

OUT = "resources/fonts"
FAMILY = "Literata"
VERSION = "esse 1.0"

# (upright or italic source, weight, italic, style name, name shown to a human)
FACES = [
    ("upright", 400, False, "Regular", "Regular"),
    ("upright", 500, False, "Medium", "Medium"),
    ("upright", 600, False, "SemiBold", "SemiBold"),
    ("italic", 400, True, "Italic", "Italic"),
    ("italic", 600, True, "SemiBoldItalic", "SemiBold Italic"),
]

# fsSelection bits
ITALIC, BOLD, REGULAR = 1 << 0, 1 << 5, 1 << 6


def build(source, weight, italic, style, pretty):
    font = TTFont(source)
    instantiateVariableFont(font, {"wght": weight, "opsz": OPSZ}, inplace=True, static=True)

    os2 = font["OS/2"]
    os2.usWeightClass = weight
    kept = os2.fsSelection & ~(ITALIC | BOLD | REGULAR)
    os2.fsSelection = kept | (ITALIC if italic else REGULAR)

    head = font["head"]
    head.macStyle = (head.macStyle & ~0b11) | (0b10 if italic else 0)

    full, postscript = f"{FAMILY} {pretty}", f"{FAMILY}-{style}"
    names = font["name"]
    for nid in (16, 17, 21, 22):
        names.removeNames(nameID=nid)
    for nid, value in (
        (1, FAMILY),
        (2, "Italic" if italic else "Regular"),
        (3, f"{VERSION};{postscript}"),
        (4, full),
        (6, postscript),
    ):
        names.setName(value, nid, 3, 1, 0x409)  # Windows / Unicode / en-US
        names.setName(value, nid, 1, 0, 0)  # Macintosh / Roman / en

    # Static faces, so the axis records would only describe what is no longer
    # there — and Core Text reads them.
    if "STAT" in font:
        del font["STAT"]

    os.makedirs(OUT, exist_ok=True)
    path = f"{OUT}/{postscript}.ttf"
    font.save(path)
    return path


def main():
    if len(sys.argv) != 3:
        sys.exit(f"usage: {sys.argv[0]} <Literata[opsz,wght].ttf> <Literata-Italic[opsz,wght].ttf>")
    sources = {"upright": sys.argv[1], "italic": sys.argv[2]}

    for which, weight, italic, style, pretty in FACES:
        path = build(sources[which], weight, italic, style, pretty)
        print(f"{path}  {os.path.getsize(path) / 1024:.0f} KB")


if __name__ == "__main__":
    main()
