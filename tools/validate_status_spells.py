"""Compare instant status casts, visible status clocks and original tile transforms."""
import argparse,hashlib,json,struct,subprocess,tempfile
from pathlib import Path
from unicorn import Uc,UC_ARCH_M68K,UC_MODE_BIG_ENDIAN,UC_HOOK_CODE
from unicorn.m68k_const import *
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom',type=Path);a=p.parse_args()
rom=a.rom.read_bytes();assert hashlib.sha256(rom).hexdigest()=='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e'
root=Path(__file__).resolve().parents[1]
c=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
for base,size in [(0,0x200000),(0xff0000,0x10000),(0x400000,0x20000),(0x600000,0x1000)]:c.mem_map(base,size)
c.mem_write(0,rom)
def w(at,v):c.mem_write(at,struct.pack('>H',v))
def l(at,v):c.mem_write(at,struct.pack('>I',v))
def rw(at):return struct.unpack('>H',c.mem_read(at,2))[0]
def rl(at):return struct.unpack('>I',c.mem_read(at,4))[0]
def reset():
 c.mem_write(0xff0000,bytes(65536))
 for reg in [UC_M68K_REG_D0,UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D3,UC_M68K_REG_D4,UC_M68K_REG_D5,UC_M68K_REG_D6,UC_M68K_REG_D7,UC_M68K_REG_A0,UC_M68K_REG_A1,UC_M68K_REG_A2,UC_M68K_REG_A3,UC_M68K_REG_A4,UC_M68K_REG_A5,UC_M68K_REG_A6]:c.reg_write(reg,0)
 c.reg_write(UC_M68K_REG_SR,0x2700)
def call(start,stop=0x600000):
 c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x600000)
 hook=c.hook_add(UC_HOOK_CODE,lambda uc,at,size,user:uc.emu_stop() if at==stop else None)
 try:c.emu_start(start,stop,count=200000)
 finally:c.hook_del(hook)
 assert c.reg_read(UC_M68K_REG_PC)==stop,(hex(start),hex(c.reg_read(UC_M68K_REG_PC)))
for at in (0x1232,0x3404,0x530de):c.mem_write(at,b'\x4e\x75');c.ctl_remove_cache(at,at+2)
with tempfile.TemporaryDirectory() as tmp:
 out=Path(tmp)/'status';subprocess.run([str(root/'target/release/shadowrealm'),'status-data',str(a.rom.resolve()),str(out)],check=True)
 data=json.loads((out/'statuses.json').read_text())
 for i,t in enumerate(data['casts']):
  reset();c.mem_write(0xff0100,bytes(t['before']));w(0xfff9f8,t['seed']);w(0xffdf86,17)
  c.mem_write(0xffd86c,bytes(t['occupied']));l(0xfffc04,0xff0100)
  c.reg_write(UC_M68K_REG_A0,0xff0100+256*t['caster']);c.reg_write(UC_M68K_REG_A1,0xff0100+256*t['target']);call(0x1033e)
  got=bytes(c.mem_read(0xff0100,2560));expected=bytes(t['after'])
  assert got==expected,f'status cast {i}: {[(hex(j),expected[j],got[j]) for j in range(2560) if got[j]!=expected[j]][:16]}'
  assert rw(0xfff9f8)==t['after_seed'] and rw(0xffdf84)==t['engaged'] and rw(0xffdf86)==t['timeout'],f'cast RNG/alert {i}'
  assert list(map(bool,c.mem_read(0xffd86c,18)))==t['after_occupied'],f'cast graphics {i}'
 print(f'Original Invisibility/Rockskin casts through initializer, allocation and drain: {len(data["casts"])}; only audio/HUD omitted',flush=True)
 for i,t in enumerate(data['clocks']):
  reset();c.mem_write(0xff0100,bytes(t['before']));w(0xffdf88,t['phase'])
  for slot in range(10):
   at=0xff0100+256*slot
   if c.mem_read(at+0x10,1)[0]&1 and rw(at+0xc)!=0:
    c.reg_write(UC_M68K_REG_A0,at);call(0x46f8,0x4714)
  assert bytes(c.mem_read(0xff0100,2560))==bytes(t['after']),f'status clocks {i}'
 print(f'Original visible-actor status clock phases: {len(data["clocks"])}; no stubs',flush=True)
 for i,t in enumerate(data['graphics']):
  reset();desc=rl(0x3232+4*t['map']);grid=rl(desc+0x14);definitions=rl(desc+0x18)
  c.mem_write(0xff2700,rom[grid:grid+8192]);w(0xffddea,rw(desc+0x24));l(0xffddfe,rl(desc+0x38))
  if definitions&1:
   c.reg_write(UC_M68K_REG_A3,definitions&~1);c.reg_write(UC_M68K_REG_A1,0x400000)
   c.emu_start(0x1610,0x1670,count=2000000);assert c.reg_read(UC_M68K_REG_PC)==0x1670
   definitions=0x400000
  l(0xffddde,definitions);w(0xffddc6,t['camera'][0]*16);w(0xffddc8,t['camera'][1]*16)
  c.mem_write(0xff0100,bytes(t['actor']));l(0xff0112,t['frame']);l(0xff0144,0xffa000);l(0xff0148,0xffb000)
  flags=t['actor'][0x11]
  for bit,entry in [(0x80,0x3efe),(0x20,0x3e86)]:
   if flags&bit:c.reg_write(UC_M68K_REG_A0,0xff0100);c.reg_write(UC_M68K_REG_A2,t['frame']);call(entry)
  pointer=0xffb000 if flags&0x20 else 0xffa000
  got=bytes(c.mem_read(pointer,len(t['bytes'])))
  assert got==bytes(t['bytes']),f'graphics {i}, map {t["map"]}, frame {t["frame"]:x}: {[(j,b,got[j]) for j,b in enumerate(t["bytes"]) if got[j]!=b][:12]}'
  c.reg_write(UC_M68K_REG_A0,0xff0100);c.reg_write(UC_M68K_REG_A3,0xffd000);c.reg_write(UC_M68K_REG_D5,1);call(0x622c)
  actual=[]
  for at in range(0xffd000,c.reg_read(UC_M68K_REG_A3),8):
   y,size_link,attribute,x=struct.unpack('>HHHH',c.mem_read(at,8));size=size_link>>8
   actual.append([((x-rw(0xff010a)+32768)&65535)-32768,((y-rw(0xff010c)+32768)&65535)-32768,((size>>2)&3)+1,(size&3)+1,attribute])
  assert actual==t['pieces'],f'status sprite attributes {i}'
 print(f'PASS: {len(data["graphics"])} complete original status tile transforms and sprite records across five maps; no stubs',flush=True)
