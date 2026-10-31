"""Cross-check native output against data and instruction semantics in the supplied ROM."""
import argparse
import hashlib
from pathlib import Path
import subprocess
import tempfile

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('rom', type=Path)
a = p.parse_args()
b = a.rom.read_bytes()
if hashlib.sha256(b).hexdigest() != '92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e':
    p.error('this reference check requires the original supplied ROM revision')
# Verify the constants used by native input against the actual ROM table.
expected_directions = [0xffff, 0, 0x8000, 0xffff, 0xc000, 0xe000, 0xa000, 0xffff,
                       0x4000, 0x2000, 0x6000, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff]
actual = [int.from_bytes(b[i:i+2], 'big') for i in range(0x19d2, 0x19f2, 2)]
if actual != expected_directions:
    raise RuntimeError('unexpected direction table')
# Independent transcription of ROL.L #4 / ROR.B #1 / OR.L at $3b6.
rows = []
for row in b[0x4ee:0x6c6]:
    d2, d4, d1 = 0x10000000, 0, row
    for _ in range(8):
        d2 = ((d2 << 4) | (d2 >> 28)) & 0xffffffff
        carry = d1 & 1
        d1 = ((d1 >> 1) | (d1 << 7)) & 255
        if carry:
            d4 |= d2
    rows.append([(d4 >> shift) & 15 for shift in range(28, -1, -4)])
expected = bytearray(320 * 224 * 3)
# Text positions for the supplied U-region ROM, following $3cc..$430.
for address, y in [(0x484, 8), (0x4bf, 9), (0x4a4, 10)]:
    x = b[address]
    text = b[address+1:b.index(0, address+1)]
    for i, char in enumerate(text):
        for row in range(8):
            for col in range(8):
                value = rows[(char-32)*8+row][col] * 255
                pos = ((y*8+row)*320+(x+i)*8+col)*3
                expected[pos:pos+3] = bytes([value])*3
binary = Path(__file__).resolve().parents[1] / 'target/release/shadowrealm'
with tempfile.TemporaryDirectory() as directory:
    output = Path(directory) / 'screen.ppm'
    subprocess.run([str(binary), 'region-screen', str(a.rom.resolve()), str(output)], check=True)
    actual_image = output.read_bytes()
if actual_image != b'P6\n320 224\n255\n' + expected:
    raise RuntimeError('rendered screen differs from instruction-based reference')
print('PASS: ROM identity, 16 direction entries, and all 71,680 screen pixels')
print('PPM SHA-256:', hashlib.sha256(actual_image).hexdigest())
