"""Compare hotel admission/payment/rest and Karma quote/commit with original M68000."""
import argparse,hashlib,json,struct,subprocess,tempfile
from pathlib import Path
from unicorn import Uc,UC_ARCH_M68K,UC_MODE_BIG_ENDIAN,UC_HOOK_CODE
from unicorn.m68k_const import *
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom',type=Path);a=p.parse_args()
rom=a.rom.read_bytes();assert hashlib.sha256(rom).hexdigest()=='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e'
root=Path(__file__).resolve().parents[1]
c=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
for base,size in [(0,0x200000),(0xff0000,65536),(0x400000,4096)]:c.mem_map(base,size)
c.mem_write(0,rom)
def w(at,v):c.mem_write(at,struct.pack('>H',v))
def l(at,v):c.mem_write(at,struct.pack('>I',v))
def rl(at):return struct.unpack('>I',c.mem_read(at,4))[0]
def reset():
 c.mem_write(0xff0000,bytes(65536));c.reg_write(UC_M68K_REG_SR,0x2700)
 for r in [UC_M68K_REG_D0,UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D3,UC_M68K_REG_D4,UC_M68K_REG_D5,UC_M68K_REG_D6,UC_M68K_REG_D7,UC_M68K_REG_A0,UC_M68K_REG_A1,UC_M68K_REG_A2,UC_M68K_REG_A3,UC_M68K_REG_A4,UC_M68K_REG_A5,UC_M68K_REG_A6]:c.reg_write(r,0)
def call(start,stops=(0x400000,)):
 c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000)
 hook=c.hook_add(UC_HOOK_CODE,lambda uc,at,size,user:uc.emu_stop() if at in stops else None)
 try:c.emu_start(start,0x400000,count=100000)
 finally:c.hook_del(hook)
 pc=c.reg_read(UC_M68K_REG_PC);assert pc in stops,(hex(start),hex(pc))
 return pc
def patch_ui(addresses):
 for at in addresses:c.mem_write(at,b'\x4e\x75');c.ctl_remove_cache(at,at+2)
with tempfile.TemporaryDirectory() as tmp:
 out=Path(tmp)/'data';subprocess.run([str(root/'target/release/shadowrealm'),'advancement-data',str(a.rom.resolve()),str(out)],check=True)
 data=json.loads((out/'advancement.json').read_text())
 reset();c.mem_write(0xfffbf8,b'\x00\x03');call(0x1f65a,(0x1f246,))
 assert c.reg_read(UC_M68K_REG_A0)==0xf4910
 assert c.mem_read(0xfff10d,1)[0]==75
 print('PASS: original Gates hotel entrance, opening and 75-nuyen quote',flush=True)
 for t in data['payments']:
  reset();l(0xfffbfe,t['before']);c.mem_write(0xfff10d,b'\x4b');call(0x21458)
  assert rl(0xfffbfe)==(t['before'] if t['after'] is None else t['after'])
  assert c.mem_read(0xfff10d,1)[0]==(75 if t['after'] is None else 255)
  assert c.reg_read(UC_M68K_REG_A0)==(0xf5af8 if t['after'] is None else 0xf5ab8)
 print(f'PASS: {len(data["payments"])} original payment/funds/overflow boundaries',flush=True)
 # Only font/tile display, waiting and confirmation input are omitted.
 patch_ui([0x12c5a,0xc0da,0xc216,0x11c0,0xd53c,0xc3e6,0x1232,0xf312])
 for i,t in enumerate(data['upgrades']):
  reset();at=0xff0100+t['slot']*256;c.mem_write(at,bytes(t['before']));l(0xfffc04,at)
  c.mem_write(0xfff0d0,bytes([t['col'],t['row']]))
  call(0xd82a)
  assert c.mem_read(0xfff0b3,1)[0]==(255 if t['maximum'] else 0),f'cap {i}'
  if t['quoted']:
   q=bytes(c.mem_read(0xfff0b6,3));index=t['id'] if t['id']<8 else t['id']-8
   assert q==bytes([t['next'],index,t['cost']]),f'quote {i}: {list(q)} != {[t["next"],index,t["cost"]]}'
  c.mem_write(0xfff0dc,bytes([0 if t['confirmed'] else 1]));call(0xd33a,(0xd26a,0xd0d6))
  actual=bytes(c.mem_read(at,256));expected=bytes(t['after'])
  assert actual==expected,f'upgrade {i}: {[(hex(j),actual[j],expected[j]) for j in range(256) if actual[j]!=expected[j]]}'
 print(f'PASS: {len(data["upgrades"])} original quotes and complete Karma decisions/actor writes, all 20 cells, byte values, races, class gates, specialization costs and cancellation',flush=True)
 for i,t in enumerate(data['rests']):
  reset()
  for slot,raw in enumerate(t['before']):c.mem_write(0xff0100+slot*256,bytes(raw))
  l(0xfffc04,0xff0100);call(0xef08)
  for slot,raw in enumerate(t['after']):
   actual=bytes(c.mem_read(0xff0100+slot*256,256));expected=bytes(raw)
   assert actual==expected,f'rest {i}/{slot}: {[(hex(j),actual[j],expected[j]) for j in range(256) if actual[j]!=expected[j]]}'
 print(f'PASS: {len(data["rests"])} full original three-runner rests, health/condition byte boundaries, active and wounded flags; only display/wait omitted',flush=True)
