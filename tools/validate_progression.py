"""Compare Stoker's first objective and taxi decisions/arrivals to original M68000."""
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
with tempfile.TemporaryDirectory() as tmp:
 out=Path(tmp)/'data';subprocess.run([str(root/'target/release/shadowrealm'),'progression-data',str(a.rom.resolve()),str(out)],check=True)
 data=json.loads((out/'progression.json').read_text())
 for i,t in enumerate(data['purchases']):
  reset();c.mem_write(0xff0100,bytes(t['before']));l(0xfffbfe,t['balance']);l(0xfffc1e,t['flags']);l(0xfffc30,t['leads']);c.mem_write(0xfffc27,bytes([t['inquiries']]));l(0xffdd64,0)
  call(0x213fa)
  assert bytes(c.mem_read(0xff0100,768))==bytes(t['after']),f'belongings party/Karma {i}'
  assert (rl(0xfffbfe),rl(0xfffc1e),rl(0xfffc30),c.mem_read(0xfffc27,1)[0],c.reg_read(UC_M68K_REG_A0))==(t['after_balance'],t['after_flags'],t['after_leads'],t['after_inquiries'],t['node']),f'belongings state {i}'
 print(f'PASS: {len(data["purchases"])} complete original belongings purchases/refusals, flags, nuyen, every party byte and Karma; no stubs',flush=True)
 for i,t in enumerate(data['gates']):
  reset();l(0xfffc1e,t['flags']);c.mem_write(0xfffbf8,bytes([t['current']]));c.mem_write(0xfff09c,bytes([t['cursor']]));call(0x20714,0x1f246)
  assert c.reg_read(UC_M68K_REG_A0)==t['node'],f'taxi story/same-district gate {i}'
 print(f'PASS: {len(data["gates"])} original taxi story/same-district gates; stop at dialogue entry, no stubs',flush=True)
 for i,t in enumerate(data['taxis']):
  reset();l(0xfffbfe,t['balance']);l(0xfffc22,t['carried']);c.mem_write(0xfffbf8,bytes([t['current']]));c.mem_write(0xfff09c,bytes([t['cursor']]));c.mem_write(0xfff0e0,bytes([t['fare']]));call(0x208a8)
  assert (rl(0xfffbfe),rl(0xfffc22),c.reg_read(UC_M68K_REG_A0))==(t['after_balance'],t['after_carried'],t['node']),f'taxi payment {i}'
  assert c.mem_read(0xfffbf8,1)[0]==(t['target'] if t['travelled'] else t['current']),f'taxi district {i}'
  assert c.mem_read(0xfff0e0,1)[0]==(0 if t['travelled'] else t['fare']),f'taxi fare scratch {i}'
 print(f'PASS: {len(data["taxis"])} complete original taxi payments/free rides/visa gates, nuyen and carried flags; no stubs',flush=True)
 for i,t in enumerate(data['armbands']):
  reset();c.mem_write(0xff018f,b'\x20');l(0xfffc22,t['carried']);c.mem_write(0xfffbf8,bytes([t['current']]));c.mem_write(0xfff09c,bytes([t['cursor']]));c.mem_write(0xfff0e0,bytes([t['fare']]));call(0x2076a)
  assert c.reg_read(UC_M68K_REG_A0)==t['node'],f'armband notice {i}'
  assert rl(0xfffc22)==t['after_carried'] and c.mem_read(0xfffbf8,1)[0]==(t['target'] if t['travelled'] else t['current']),f'armband travel/visa {i}'
 print(f'PASS: {len(data["armbands"])} original armband free-ride and visa decisions; no stubs',flush=True)
 for i,t in enumerate(data['routes']):
  reset();w(0xffddc4,t['source']);w(0xffdf40,2);w(0xffdf8e,3);w(0xfffc0e,1);c.reg_write(UC_M68K_REG_D0,t['target'])
  # Skip only the leading fade/sound call. This original suffix owns map state.
  call(0x136e4)
  assert (rw(0xffddc4),rw(0xffdf40),rw(0xffdf8e),rw(0xfffc0e),c.mem_read(0xfff0a0,1)[0])==(t['target'],0,65535,0,255),f'taxi route {i}'
  at=struct.unpack_from('>I',rom,0x3232+t['target']*4)[0];c.mem_write(0xffddc6,rom[at:at+96]);call(0x2b34,0x2b76)
  assert [rw(0xff0100+s*256+p) for s in range(3) for p in [0,4]]==sum(t['positions'],[]),f'taxi arrival {i}'
  assert [rw(0xffddc6+p)&0xff80 for p in [0,2]]==t['camera'],f'taxi camera {i}'
 print(f'PASS: {len(data["routes"])} city route state and descriptor camera/party arrivals; fade/sound prefix omitted, no instruction patches',flush=True)
