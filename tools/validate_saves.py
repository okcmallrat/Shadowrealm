"""Compare full original save images, SRAM banks and persistent load reconstruction."""
import argparse,hashlib,json,struct,subprocess,tempfile
from pathlib import Path
from unicorn import Uc,UC_ARCH_M68K,UC_MODE_BIG_ENDIAN,UC_HOOK_CODE
from unicorn.m68k_const import *
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom',type=Path);a=p.parse_args()
rom=a.rom.read_bytes();assert hashlib.sha256(rom).hexdigest()=='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e'
root=Path(__file__).resolve().parents[1]
c=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
for base,size in [(0,0x200000),(0x200000,0x4000),(0xff0000,65536),(0x400000,4096)]:c.mem_map(base,size)
c.mem_write(0,rom)
def w(at,v):c.mem_write(at,struct.pack('>H',v))
def l(at,v):c.mem_write(at,struct.pack('>I',v))
def rw(at):return struct.unpack('>H',c.mem_read(at,2))[0]
def rl(at):return struct.unpack('>I',c.mem_read(at,4))[0]
def reset():
 c.mem_write(0xff0000,bytes(65536));c.reg_write(UC_M68K_REG_SR,0x2700)
 for r in [UC_M68K_REG_D0,UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D3,UC_M68K_REG_D4,UC_M68K_REG_D5,UC_M68K_REG_D6,UC_M68K_REG_D7,UC_M68K_REG_A0,UC_M68K_REG_A1,UC_M68K_REG_A2,UC_M68K_REG_A3,UC_M68K_REG_A4,UC_M68K_REG_A5,UC_M68K_REG_A6]:c.reg_write(r,0)
def call(start,stop=0x400000):
 c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000)
 hook=c.hook_add(UC_HOOK_CODE,lambda uc,at,size,user:uc.emu_stop() if at==stop else None)
 try:c.emu_start(start,stop,count=250000)
 finally:c.hook_del(hook)
 assert c.reg_read(UC_M68K_REG_PC)==stop,(hex(start),hex(c.reg_read(UC_M68K_REG_PC)))
def pack(at):
 return b''.join(bytes(c.mem_read(at+offset,size)) for offset,size in [(0xe,2),(0x2c,2),(0x4c,4),(0x56,2),(0x58,55),(0x9a,3),(0x90,2),(0xa0,2),(0x8f,1),(0x9d,1)])
with tempfile.TemporaryDirectory() as tmp:
 out=Path(tmp)/'data';subprocess.run([str(root/'target/release/shadowrealm'),'save-data',str(a.rom.resolve()),str(out)],check=True)
 data=json.loads((out/'saves.json').read_text())
 for i,t in enumerate(data['writes']):
  reset();c.mem_write(0xfff1fc,bytes(t['archives']));c.mem_write(0xfff9fa,bytes(t['global']));c.mem_write(0xff0100,bytes(t['actors']));w(0xffddc4,t['map']);w(0xffddc6,t['camera'][0]);w(0xffddc8,t['camera'][1]);c.mem_write(0xfff0dc,bytes([t['slot']]))
  call(0x596c4)
  expected=bytes(t['image']);assert bytes(c.mem_read(0xfff1e8,2048))==expected,f'save buffer {i}'
  primary=t['slot']*0x1000
  for base in [primary,primary+0x800]:assert bytes(c.mem_read(0x200000+base*2+1,4095))[::2]==expected,f'SRAM bank {i}/{base}'
  c.reg_write(UC_M68K_REG_D0,primary);c.reg_write(UC_M68K_REG_D7,2047);c.reg_write(UC_M68K_REG_A1,0xfff1e8);call(0x593fc);assert c.reg_read(UC_M68K_REG_SR)&1==0
  bad=20+i%1564;at=0x200000+(primary+bad)*2+1;c.mem_write(at,bytes([expected[bad]^1]));c.reg_write(UC_M68K_REG_D0,primary);c.reg_write(UC_M68K_REG_D7,2047);c.reg_write(UC_M68K_REG_A1,0xfff1e8);call(0x593fc);assert c.reg_read(UC_M68K_REG_SR)&1,f'bad checksum admitted {i}'
 print(f'PASS: {len(data["writes"])} complete original save buffers and both SRAM copies, with compact overwrites/global/coordinates/checksum and corruption rejection; no stubs',flush=True)
 # Omit only map/graphics/recovery loader $3074; compact restoration, companion
 # initialization, coordinates, flags/control and portrait choice still execute.
 c.mem_write(0x3074,b'\x4e\x75');c.ctl_remove_cache(0x3074,0x3076)
 for i,t in enumerate(data['loads']):
  reset();c.mem_write(0xfff1e8,bytes(t['image']));c.mem_write(0xff0100,rom[0x7291a:0x72a1a]);call(0x594e0)
  assert rw(0xffddc4)==t['map'] and [rw(0xffddc6),rw(0xffddc8)]==t['camera'],f'load map/camera {i}'
  assert rl(0xfffc04)==0xff0100 and rl(0xff0150)==t['portrait'],f'load control/portrait {i}'
  for slot,record in enumerate(t['packed']):
   at=0xff0100+slot*256;assert pack(at)==bytes(record),f'load compact {i}/{slot}'
   assert [rw(at),rw(at+4)]==t['positions'][slot] and c.mem_read(at+0x10,1)[0]==t['flags'][slot],f'load position/flags {i}/{slot}'
 print(f'PASS: {len(data["loads"])} original persistent loads, companion initialization/archives, party bytes, map/camera/positions, Joshua control and portrait; map/graphics/recovery loader omitted',flush=True)
