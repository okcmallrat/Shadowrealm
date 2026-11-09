"""Compare complete post-objective Stoker encounter rolls and close spawns to M68000."""
import argparse,hashlib,json,struct,subprocess,tempfile
from pathlib import Path
from unicorn import Uc,UC_ARCH_M68K,UC_MODE_BIG_ENDIAN
from unicorn.m68k_const import *
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom',type=Path);a=p.parse_args();rom=a.rom.read_bytes();assert hashlib.sha256(rom).hexdigest()=='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e'
root=Path(__file__).resolve().parents[1];c=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
for at,size in [(0,0x200000),(0xff0000,65536),(0x400000,4096)]:c.mem_map(at,size)
c.mem_write(0,rom)
def w(at,v):c.mem_write(at,struct.pack('>H',v))
def l(at,v):c.mem_write(at,struct.pack('>I',v))
def rw(at):return struct.unpack('>H',c.mem_read(at,2))[0]
def reset():
 c.mem_write(0xff0000,bytes(65536));c.reg_write(UC_M68K_REG_SR,0x2700)
 for r in [UC_M68K_REG_D0,UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D3,UC_M68K_REG_D4,UC_M68K_REG_D5,UC_M68K_REG_D6,UC_M68K_REG_D7,UC_M68K_REG_A0,UC_M68K_REG_A1,UC_M68K_REG_A2,UC_M68K_REG_A3,UC_M68K_REG_A4,UC_M68K_REG_A5,UC_M68K_REG_A6]:c.reg_write(r,0)
def call(start):
 c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000);c.emu_start(start,0x400000,count=1000000);assert c.reg_read(UC_M68K_REG_PC)==0x400000
with tempfile.TemporaryDirectory() as tmp:
 out=Path(tmp)/'data';subprocess.run([str(root/'target/release/shadowrealm'),'stoker-data',str(a.rom.resolve()),str(out)],check=True);data=json.loads((out/'stoker.json').read_text())
 for i,t in enumerate(data['visits']):
  reset();w(0xffddc4,2);c.mem_write(0xfffbf8,b'\x02');descriptor=struct.unpack_from('>I',rom,0x3232+8)[0];c.mem_write(0xffddc6,rom[descriptor:descriptor+96]);w(0xffdf8a,65535);call(0xee10)
  c.mem_write(0xff0100,bytes(t['before']));w(0xffddc6,t['camera'][0]);w(0xffddc8,t['camera'][1]);w(0xfff9f8,t['seed']);w(0xffdf86,123)
  for slot in range(3,10):
   raw=t['before'][slot*256:(slot+1)*256]
   if raw[0x10]&9==1:c.mem_write(0xffd86c+struct.unpack_from('>H',bytes(raw),0x30)[0],b'\x01')
  call(0x249c4)
  actual=bytes(c.mem_read(0xff0100,len(t['after'])));expected=bytes(t['after']);assert actual==expected,f'visit {i}: {[(hex(j),actual[j],expected[j]) for j in range(len(actual)) if actual[j]!=expected[j]][:20]}'
  assert c.reg_read(UC_M68K_REG_A0)==t['node'],f'node {i}'
  assert rw(0xfff9f8)==t['after_seed'] and rw(0xffdf84)==t['engaged'] and rw(0xffdf86)==t['timeout'],f'RNG/alert {i}'
  assert c.mem_read(0xffe177,1)[0]==t['count'],f'encounter count {i}'
 print(f'PASS: {len(data["visits"])} complete original Stoker action-233 rolls, both dialogue outcomes, close placement, actor/graphics allocation, firearm + unarmed group, pool exhaustion, count/alert and shared RNG; no stubs',flush=True)
