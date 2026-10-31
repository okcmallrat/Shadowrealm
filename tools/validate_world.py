"""Check collision raster/query and map animations against original M68000 routines."""
import argparse,csv,hashlib,struct,subprocess,tempfile
from pathlib import Path
from unicorn import Uc,UC_ARCH_M68K,UC_MODE_BIG_ENDIAN,UC_HOOK_CODE,UC_HOOK_MEM_WRITE
from unicorn.m68k_const import *
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom',type=Path);a=p.parse_args();rom=a.rom.read_bytes()
if hashlib.sha256(rom).hexdigest()!='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e':p.error('requires supplied ROM revision')
root=Path(__file__).resolve().parents[1];binary=root/'target/release/shadowrealm'
def word(p):return struct.unpack_from('>H',rom,p)[0]
def long(p):return struct.unpack_from('>I',rom,p)[0]
def cpu_new():
 c=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
 for base,size in [(0,0x200000),(0x400000,0x1000),(0xff0000,0x10000),(0xc00000,0x1000)]:c.mem_map(base,size)
 c.mem_write(0,rom);c.reg_write(UC_M68K_REG_SR,0x2700);return c
queries=0;animated=0
with tempfile.TemporaryDirectory() as temp:
 for index in range(56):
  output=Path(temp)/str(index)
  result=subprocess.run([str(binary),'world-data',str(a.rom.resolve()),str(index),str(output)],capture_output=True,text=True)
  if result.returncode:raise RuntimeError(f'native map {index}: {result.stderr}')
  c=cpu_new();desc=long(0x3232+index*4);c.mem_write(0xffddc6,rom[desc:desc+96]);c.mem_write(0xffdf8a,b'\xff\xff')
  def call(address):
   c.reg_write(UC_M68K_REG_A7,0xffff00);c.mem_write(0xffff00,struct.pack('>I',0x400000));c.reg_write(UC_M68K_REG_A0,0xff0100)
   c.emu_start(address,0x400000,count=2_000_000)
   if c.reg_read(UC_M68K_REG_PC)!=0x400000:raise RuntimeError(f'routine {address:x} did not return')
  call(0xee10)
  if output.joinpath('collision.bin').read_bytes()!=bytes(c.mem_read(0xff1700,4096)):raise RuntimeError(f'map {index}: collision raster mismatch')
  c.mem_write(0xff0110,b'\x0f')
  for row in csv.DictReader(output.joinpath('queries.csv').open()):
   c.reg_write(UC_M68K_REG_D5,int(row['x']));c.reg_write(UC_M68K_REG_D6,int(row['y']));call(0xe7c8)
   actual=c.reg_read(UC_M68K_REG_D0)&255
   if actual!=int(row['result']):raise RuntimeError(f'map {index} query {row}: original returned {actual}')
   queries+=1
  # Set the same shared definition base used after $29fc.
  definitions=long(desc+0x18)
  if definitions&1:
   c.reg_write(UC_M68K_REG_A3,definitions&~1);c.reg_write(UC_M68K_REG_A1,0xff6700)
   c.reg_write(UC_M68K_REG_A7,0xffff00);c.emu_start(0x1610,0x1670,count=2_000_000)
   if c.reg_read(UC_M68K_REG_PC)!=0x1670:raise RuntimeError('definition decompression failed')
   definitions=0xff6700
  c.mem_write(0xffddde,struct.pack('>I',definitions));call(0x164a8)
  vram=bytearray(65536);cursor=long(desc+0x34)+16
  while long(cursor):
   source,dest,size=struct.unpack_from('>III',rom,cursor);cursor+=12
   if source&1:raise RuntimeError('unexpected compressed graphics setup')
   vram[dest:dest+size]=rom[source:source+size]
  destination=[0]
  def command(cpu,address,size,user):destination[0]=cpu.reg_read(UC_M68K_REG_D0)&65535
  def write(cpu,access,address,size,value,user):
   if address==0xc00000:
    start=destination[0];vram[start:start+size]=value.to_bytes(size,'big');destination[0]=(start+size)&65535
  c.hook_add(UC_HOOK_CODE,command,begin=0xe9c,end=0xe9c);c.hook_add(UC_HOOK_MEM_WRITE,write,begin=0xc00000,end=0xc00003)
  for tick in range(16):
   call(0x163da)
   if output.joinpath(f'vram-{tick:02}.bin').read_bytes()!=vram:raise RuntimeError(f'map {index}, tick {tick}: environment animation mismatch')
  if long(long(desc+0x3c)):animated+=1
  if index==0:
   # Full real-template HUD refresh, not the earlier blank/sample HUD fixture.
   c=cpu_new();c.mem_write(0xff0100,output.joinpath('runners.bin').read_bytes())
   c.mem_write(0xfffc04,struct.pack('>I',0xff0100))
   vram=bytearray(65536);destination=[0]
   c.hook_add(UC_HOOK_CODE,command,begin=0xe9c,end=0xe9c);c.hook_add(UC_HOOK_MEM_WRITE,write,begin=0xc00000,end=0xc00003)
   call(0x334c);call(0x3404);call(0x39f2)
   native_window=output.joinpath('window.bin').read_bytes()
   if native_window!=vram[0xf000:]:
    differences=[(hex(0xf000+i),native_window[i:i+2].hex(),vram[0xf000+i:0xf002+i].hex()) for i in range(0,4096,2) if native_window[i:i+2]!=vram[0xf000+i:0xf002+i]]
    raise RuntimeError(f'real-template party HUD mismatch: {differences[:12]}')
   if output.joinpath('hud-vram.bin').read_bytes()[0xd280:0xdf80]!=vram[0xd280:0xdf80]:raise RuntimeError('real-template portrait/equipment graphics differ')

print('PASS: real-template party HUD (portraits, weapons, health, condition, resource) matches original refresh')
print(f'PASS: 56 collision rasters, {queries} collision queries; 16 VRAM ticks for every map ({animated} animated maps), matching original routines')
