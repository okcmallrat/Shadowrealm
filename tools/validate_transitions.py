"""Compare native seam plans with original $3168 and $2b78 execution.
Requires Unicorn 2.1.4. Does not emulate the full game or interior scripts.
"""
import argparse,csv,hashlib,struct,subprocess,tempfile
from pathlib import Path
from unicorn import Uc,UC_ARCH_M68K,UC_MODE_BIG_ENDIAN
from unicorn.m68k_const import *
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom',type=Path);a=p.parse_args();rom=a.rom.read_bytes()
if hashlib.sha256(rom).hexdigest()!='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e':p.error('requires the supplied ROM revision')
root=Path(__file__).resolve().parents[1];binary=root/'target/release/shadowrealm'
c=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
for address,size in [(0,0x200000),(0xff0000,0x10000),(0x400000,0x1000)]:c.mem_map(address,size)
c.mem_write(0,rom)
def putw(address,value):c.mem_write(address,struct.pack('>H',value))
def getw(address):return struct.unpack('>H',bytes(c.mem_read(address,2)))[0]
def getl(address):return struct.unpack('>I',bytes(c.mem_read(address,4)))[0]
with tempfile.TemporaryDirectory() as temp:
 output=Path(temp)/'transitions.csv'
 subprocess.run([str(binary),'transition-data',str(a.rom.resolve()),str(output)],check=True)
 rows=list(csv.DictReader(output.open()));spawn_cases=0
 for row in rows:
  source=int(row['source_map']);trigger=int(row['trigger']);camera=int(row['camera_input'])
  c.mem_write(0xff0000,bytes(65536));c.reg_write(UC_M68K_REG_SR,0x2700)
  c.reg_write(UC_M68K_REG_A7,0xffff00);c.mem_write(0xffff00,struct.pack('>I',0x400000))
  putw(0xffddc4,source);putw(0xffddc6,camera);c.mem_write(0xfffbf9,bytes([trigger&31]))
  c.emu_start(0x3168,0x400000,count=10000)
  if c.reg_read(UC_M68K_REG_PC)!=0x400000:raise RuntimeError('routing routine did not return')
  supported=int(row['target_map'])>=0
  if bool(c.reg_read(UC_M68K_REG_SR)&1)!=supported:raise RuntimeError(f'eligibility mismatch: {row}')
  if not supported:continue
  if getw(0xffddc4)!=int(row['target_map']) or getl(0xffe114)!=int(row['spawn']):raise RuntimeError(f'route mismatch: {row}')
  c.reg_write(UC_M68K_REG_A7,0xffff00)
  c.emu_start(0x2b78,0x2bd0,count=10000)
  if c.reg_read(UC_M68K_REG_PC)!=0x2bd0:raise RuntimeError('spawn routine did not finish')
  actual=[getw(0xffddc6),getw(0xffddc8)]
  for slot in range(3):actual.extend([getw(0xff0100+slot*256),getw(0xff0104+slot*256)])
  expected=[int(row[key]) for key in ['camera_x','camera_y','x0','y0','x1','y1','x2','y2']]
  if actual!=expected:raise RuntimeError(f'spawn mismatch: {row}, original {actual}')
  if getl(0xffe114)!=0:raise RuntimeError('spawn pointer was not consumed')
  spawn_cases+=1
print(f'PASS: {len(rows)} native route/eligibility cases and {spawn_cases} camera/party spawn applications match original M68000 routines')
