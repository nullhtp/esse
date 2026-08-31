# The faces esse is set in

Literata, by TypeTogether — a serif drawn for long-form reading on screen, and
one of the few with a proper Cyrillic that essays here need. Licensed under the
[SIL Open Font License 1.1](OFL.txt), which travels with the files.

Five static instances, cut from the upstream variable fonts at
[google/fonts/ofl/literata](https://github.com/google/fonts/tree/main/ofl/literata)
with `scripts/fonts.py`: weights 400, 500 and 600 upright, 400 and 600 italic,
all at optical size 16. They are static because gpui picks a face by family and
weight and does not instance variable axes — see the script for the details.

To cut them again, download the two `Literata[opsz,wght].ttf` files and run:

    pip install fonttools
    python3 scripts/fonts.py Literata\[opsz,wght\].ttf Literata-Italic\[opsz,wght\].ttf

The app embeds these files at compile time (`crates/esse-app/src/fonts.rs`), so
nothing has to be installed on the machine that runs esse.
