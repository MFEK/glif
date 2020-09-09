# Run via gen.sh!
import fontforge
import psMat
import glob

font = fontforge.font()
font.fontname = font.familyname = font.fullname = "QglifIconFont"

WIDTH = 700

fn = open("../../src/imgui/icons.rs", "wb")

i = 0
for i, f in enumerate(glob.glob("*.svg")):
    g = font.createChar(0xF000+i)
    g.glyphname = f.split(".")[0]
    s = psMat.compose(psMat.scale(0.7), psMat.translate(0, -(font.ascent / 7)))
    g.importOutlines(f)
    g.transform(s)
    g.left_side_bearing = g.right_side_bearing = 0
    equalize = int((WIDTH - g.width) / 2)
    g.left_side_bearing = g.right_side_bearing = equalize
    g.width = WIDTH
    import sys
    bytes_ = "[{}]".format(", ".join([str(i) for i in chr(0xF000+i).encode("utf-8")]+["0"]))
    fn.write("pub const {}: &[u8] = &{}; // U+{:4X}\n".format(g.glyphname.upper(), bytes_, 0xF000+i).encode("ascii"))

fn.close()


font.encoding = "UnicodeBMP"
font.encoding = "compacted"
font.autoWidth(0)

font.save("icons.sfd")
font.generate("icons.ttf", flags=("no-hints", "no-flex", "omit-instructions"))
