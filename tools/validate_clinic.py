"""Compare uninsured solo recovery core and clinic arrival with original M68000.
Stops before original dialogue/UI calls; no instruction stubs.
Companion archives, insurance and mission callbacks are outside this subset.
"""
import argparse,hashlib,json,struct,subprocess,tempfile
from pathlib import Path
from unicorn import Uc,UC_ARCH_M68K,UC_MODE_BIG_ENDIAN,UC_HOOK_CODE
from unicorn.m68k_const import *
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom',type=Path);a=p.parse_args();rom=a.rom.read_bytes()
assert hashlib.sha256(rom).hexdigest()=='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e'
root=Path(__file__).resolve().parents[1]
c=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
for base,size in [(0,0x200000),(0xff0000,0x10000),(0x400000,0x1000)]:c.mem_map(base,size)
c.mem_write(0,rom)
def w(p,v):c.mem_write(p,struct.pack('>H',v))
def l(p,v):c.mem_write(p,struct.pack('>I',v))
def rw(p):return struct.unpack('>H',c.mem_read(p,2))[0]
def rl(p):return struct.unpack('>I',c.mem_read(p,4))[0]
def run(start,stop):
 c.reg_write(UC_M68K_REG_SR,0x2700);c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000)
 hook=c.hook_add(UC_HOOK_CODE,lambda uc,addr,size,user:uc.emu_stop(),begin=stop,end=stop)
 try:c.emu_start(start,0x400000,count=100_000)
 finally:c.hook_del(hook)
 assert c.reg_read(UC_M68K_REG_PC)==stop
with tempfile.TemporaryDirectory() as tmp:
 out=Path(tmp)/'out';subprocess.run([str(root/'target/release/shadowrealm'),'clinic-data',str(a.rom.resolve()),str(out)],check=True)
 data=json.loads((out/'clinic.json').read_text())
 for i,case in enumerate(data['cases']):
  c.mem_write(0xff0100,bytes(case['before']));l(0xfffc1e,0);l(0xfffbfe,case['balance']);run(0xc726,0xc7c0)
  actual=bytes(c.mem_read(0xff0100,256));expected=bytes(case['after'])
  assert actual==expected,[(i,hex(j),expected[j],actual[j]) for j in range(256) if actual[j]!=expected[j]]
  assert rl(0xfffbfe)==case['after_balance'] and case['balance']-rl(0xfffbfe)==case['fee']
  assert c.reg_read(UC_M68K_REG_A0)==data['node']
 assert struct.unpack_from('>II',rom,data['node'])==(data['text'],0xffffffff)
 run(0x2a44,0x2a52);plan=data['plan'];assert c.reg_read(UC_M68K_REG_D0)&65535==plan['map'] and rl(0xffe114)==plan['spawn']
 run(0x2b78,0x2bd0)
 assert [rw(0xffddc6),rw(0xffddc8)]==plan['camera']
 assert [[rw(0xff0100+i*256),rw(0xff0104+i*256)] for i in range(3)]==plan['positions']
 assert rl(0xffe114)==0
 print(f"PASS: {len(data['cases'])} full-actor recovery/fee cores; original uninsured node, route prefix and arrival; no stubs")
