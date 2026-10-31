"""Linear M68000 disassembly. Output may include data; it is not recovered source."""
import argparse
from pathlib import Path
from capstone import Cs, CS_ARCH_M68K, CS_MODE_BIG_ENDIAN, CS_MODE_M68K_000
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('rom', type=Path)
p.add_argument('--start', type=lambda s: int(s, 0), default=0x200)
p.add_argument('--end', type=lambda s: int(s, 0), default=0x400)
a = p.parse_args()
b = a.rom.read_bytes()
if not 0 <= a.start < a.end <= len(b):
    p.error('range must lie within the ROM')
md = Cs(CS_ARCH_M68K, CS_MODE_BIG_ENDIAN | CS_MODE_M68K_000)
for i in md.disasm(b[a.start:a.end], a.start):
    print(f'{i.address:06x}  {i.bytes.hex():<24} {i.mnemonic:<12} {i.op_str}'.rstrip())
