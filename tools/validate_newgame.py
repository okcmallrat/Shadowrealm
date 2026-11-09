"""Compare all three native new-game bootstraps with original M68000 routines."""
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
def rw(at):return struct.unpack('>H',c.mem_read(at,2))[0]
def rl(at):return struct.unpack('>I',c.mem_read(at,4))[0]
def reset():
 c.mem_write(0xff0000,bytes(65536));c.reg_write(UC_M68K_REG_SR,0x2700)
 for r in [UC_M68K_REG_D0,UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D3,UC_M68K_REG_D4,UC_M68K_REG_D5,UC_M68K_REG_D6,UC_M68K_REG_D7,UC_M68K_REG_A0,UC_M68K_REG_A1,UC_M68K_REG_A2,UC_M68K_REG_A3,UC_M68K_REG_A4,UC_M68K_REG_A5,UC_M68K_REG_A6]:c.reg_write(r,0)
def call(start,stop=0x400000):
 c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000)
 hook=c.hook_add(UC_HOOK_CODE,lambda uc,at,size,user:uc.emu_stop() if at==stop else None)
 try:c.emu_start(start,stop,count=100000)
 finally:c.hook_del(hook)
 assert c.reg_read(UC_M68K_REG_PC)==stop,(hex(start),hex(c.reg_read(UC_M68K_REG_PC)))
ranges=[(0xe,2),(0x2c,2),(0x4c,4),(0x56,2),(0x58,55),(0x9a,3),(0x90,2),(0xa0,2),(0x8f,1),(0x9d,1)]
def pack(at):return b''.join(bytes(c.mem_read(at+o,n))for o,n in ranges)
with tempfile.TemporaryDirectory() as tmp:
 out=Path(tmp)/'data';subprocess.run([str(root/'target/release/shadowrealm'),'newgame-data',str(a.rom.resolve()),str(out)],check=True)
 cases=json.loads((out/'newgame.json').read_text())
 # HUD tile transfer only: keep original resource clamping/health/condition code.
 c.mem_write(0x1232,b'\x4e\x75')
 for t in cases:
  reset()
  # Original archive initialization precedes archetype selection. Compare every
  # represented archive, including unchanged Samurai record 0 for other classes.
  call(0x59318)
  assert bytes(c.mem_read(0xfff1fc,11*74))==bytes(t['archives'])
  assert bytes(c.mem_read(0xfffc0c,2))==b'\xff\xff'
  # Final class-selection block, including its original $42b0 template copy.
  c.mem_write(0xfff0dc,bytes([t['template']]));call(0x59308)
  expected=bytearray(rom[0x7291a+t['template']*256:0x72a1a+t['template']*256]);expected[0x3e:0x40]=b'\x00\x00'
  assert bytes(c.mem_read(0xff0100,256))==expected
  assert c.mem_read(0xff0210,1)[0]==0 and c.mem_read(0xff0310,1)[0]==0
  call(0x7fc,0x868) # All represented global resets, money, lead, map IDs.
  call(0x3084) # Original descriptor + map flag + both map-grid copies.
  call(0xa7b4) # Actor pointers, including Joshua as selected runner.
  call(0x1e68c);call(0x566a2)
  call(0x2b34) # Original descriptor positions.
  call(0x38fa) # Initial HUD health/condition/ammo adjustment, no gameplay stub.
  assert pack(0xff0100)==bytes(t['packed']),f'hero compact {t["template"]}'
  assert [rw(0xff0100),rw(0xff0104)]==t['position']
  assert c.mem_read(0xff0110,1)[0]==t['actor'][0x10]
  assert [rw(0xffddc6),rw(0xffddc8)]==t['camera']
  assert rw(0xffddc4)==t['map'] and c.mem_read(0xfffbf8,1)[0]==t['map']
  assert rl(0xfffc04)==0xff0100 and rl(0xfffbfe)==t['nuyen']
  for at,k in [(0xfffc30,'leads'),(0xfffc1e,'recovery'),(0xfffc2c,'contacts'),(0xfffc22,'carried')]:assert rl(at)==t[k],k
  assert rw(0xfffc2a)==t['known'] and c.mem_read(0xfffc27,1)[0]==t['inquiries']
  assert bytes(c.mem_read(0xfffbec,10))==bytes(t['job']) and c.mem_read(0xfffc1c,1)[0]==t['job_flags']
  assert bytes(c.mem_read(0xfffc12,10))==bytes(t['opinions'])
  assert bytes(c.mem_read(0xfff9fa,170))==bytes(t['books'])
  print(f'PASS: original archetype {t["template"]}, complete persistent hero, 11 archives, five spellbooks, solo identity, wallet/story/jobs/map/camera/spawn; only HUD tile transfer omitted',flush=True)
 print('PASS: all three class-selection and represented new-game initialization paths; cinematic, menu rendering and boot warm-up cadence are outside this comparison',flush=True)
