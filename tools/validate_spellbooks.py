"""Compare original spellbook initialization, estimated drain, and selection prefixes.
Stops before spell/weapon equipment mutation; no instruction stubs.
"""
import argparse,hashlib,json,struct,subprocess,tempfile
from pathlib import Path
from unicorn import Uc,UC_ARCH_M68K,UC_MODE_BIG_ENDIAN,UC_HOOK_CODE
from unicorn.m68k_const import *
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom',type=Path);a=p.parse_args()
rom=a.rom.read_bytes();assert hashlib.sha256(rom).hexdigest()=='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e'
root=Path(__file__).resolve().parents[1]
c=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
for base,size in [(0,0x200000),(0xff0000,0x10000),(0x400000,0x1000)]:c.mem_map(base,size)
c.mem_write(0,rom)
def w(p,v):c.mem_write(p,struct.pack('>H',v))
def l(p,v):c.mem_write(p,struct.pack('>I',v))
def rw(p):return struct.unpack('>H',c.mem_read(p,2))[0]
def reset():
 c.mem_write(0xff0000,bytes(65536))
 for r in [UC_M68K_REG_D0,UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D3,UC_M68K_REG_D4,UC_M68K_REG_D5,UC_M68K_REG_D6,UC_M68K_REG_D7,UC_M68K_REG_A0,UC_M68K_REG_A1,UC_M68K_REG_A2,UC_M68K_REG_A3,UC_M68K_REG_A4]:c.reg_write(r,0)
 c.reg_write(UC_M68K_REG_SR,0x2700)
def call(start,stops=()):
 hook=c.hook_add(UC_HOOK_CODE,lambda uc,addr,size,user:uc.emu_stop() if addr in stops or addr==0x400000 else None)
 c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000)
 try:c.emu_start(start,0x400000,count=100000)
 finally:c.hook_del(hook)
 pc=c.reg_read(UC_M68K_REG_PC);assert pc in (*stops,0x400000),hex(pc)
 return pc
with tempfile.TemporaryDirectory() as temp:
 out=Path(temp)/'spellbooks';subprocess.run([str(root/'target/release/shadowrealm'),'spellbook-data',str(a.rom.resolve()),str(out)],check=True)
 data=json.loads((out/'spellbooks.json').read_text())
 reset();call(0x566a2);assert bytes(c.mem_read(0xfff9fa,170))==bytes(data['initial'])
 print('Original spellbook initialization: 170 bytes; no stubs',flush=True)
 for i,t in enumerate(data['drains']):
  reset();c.mem_write(0xff0100,bytes(t['actor']));c.reg_write(UC_M68K_REG_A0,0xff0100);c.reg_write(UC_M68K_REG_D7,t['id']);c.reg_write(UC_M68K_REG_D4,t['force'] if t['force']<128 else 0xff00|t['force']);call(0x10660)
  got=c.reg_read(UC_M68K_REG_D2)&65535
  assert got==t['drain'],f'drain {i}: {got} != {t["drain"]}; id={t["id"]} force={t["force"]}; stats={[t["actor"][j] for j in [0x80,0x7c,0x4d]]}; registers={[hex(c.reg_read(r)) for r in [UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D4,UC_M68K_REG_D6]]}'
 print(f'Original estimated drain: {len(data["drains"])}; no stubs',flush=True)
 counts=[0,0,0]
 for i,t in enumerate(data['decisions']):
  reset();c.mem_write(0xfff9fa,bytes(data['initial']));c.mem_write(0xff0400,bytes(t['actor']));w(0xfff9f8,t['seed'])
  c.mem_write(0xfffc0c,b'\xff\xff')
  for actor in t['party']:
   slot=int.from_bytes(bytes(actor[0x3e:0x40]),'big');c.mem_write(0xff0100+slot*256,bytes(actor))
   if slot:c.mem_write(0xfffc0b+slot,b'\x00')
  c.reg_write(UC_M68K_REG_A0,0xff0400)
  pc=call(0x13a92,(0x13ae4,0x13b7a,0x13ae6));kind={0x13ae4:0,0x13b7a:1,0x13ae6:2}[pc];counts[kind]+=1
  assert kind==t['kind'],f'selection {i}: {kind} != {t["kind"]}'
  assert rw(0xfff9f8)==t['after_seed'],f'RNG {i}'
  assert bytes(c.mem_read(0xff0400,256))==bytes(t['actor']),f'actor mutation {i}'
  if kind==1:
   assert c.reg_read(UC_M68K_REG_D7)&255==t['id'],f'spell ID {i}'
   assert c.reg_read(UC_M68K_REG_D4)&255==t['force'],f'force {i}'
   if t['id']==11:assert c.mem_read(0xffe190,1)[0]==t['heal'],f'healing target {i}'
 print(f'PASS: {len(data["decisions"])} original selection prefixes (keep/spell/inventory={counts}); no stubs',flush=True)

 # Only HUD transfer routines are omitted for complete equipment/drain calls.
 for address in (0x1232,0x3404):
  c.mem_write(address,b'\x4e\x75');c.ctl_remove_cache(address,address+2)
 for i,t in enumerate(data['equipment']):
  reset();c.mem_write(0xff0100,bytes(t['before']));c.reg_write(UC_M68K_REG_A0,0xff0100)
  if t['kind']==0:
   c.reg_write(UC_M68K_REG_D7,t['id']);c.reg_write(UC_M68K_REG_D4,t['force']);c.reg_write(UC_M68K_REG_D5,t['index']);call(0x13b7a)
  else:call(0x13ae6)
  got=bytes(c.mem_read(0xff0100,256));expected=bytes(t['after'])
  assert got==expected,f'equipment {i}, kind={t["kind"]}: {[(hex(j),expected[j],got[j]) for j in range(256) if got[j]!=expected[j]][:12]}'
 print(f'Complete equipment conversion: {len(data["equipment"])}; only HUD omitted',flush=True)
 for i,t in enumerate(data['applied_drains']):
  reset();c.mem_write(0xff0100,bytes(t['before']));l(0xfffc04,0xff0100);c.reg_write(UC_M68K_REG_A0,0xff0100);c.reg_write(UC_M68K_REG_D7,t['id']);c.reg_write(UC_M68K_REG_D4,t['force']);call(0x10422)
  got=bytes(c.mem_read(0xff0100,256));expected=bytes(t['after'])
  assert got==expected,f'applied drain {i}: {[(hex(j),expected[j],got[j]) for j in range(256) if got[j]!=expected[j]][:12]}'
 print(f'PASS: {len(data["applied_drains"])} complete drain applications; only HUD omitted',flush=True)
