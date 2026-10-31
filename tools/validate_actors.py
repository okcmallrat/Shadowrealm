"""Differential checks of native sprite records, clock and HUD against ROM routines.
Requires Unicorn 2.1.4. These are routine-level checks, not full-game emulation.
"""
import argparse, csv, hashlib, struct, subprocess, tempfile
from pathlib import Path
from unicorn import Uc, UC_ARCH_M68K, UC_MODE_BIG_ENDIAN, UC_HOOK_CODE, UC_HOOK_MEM_WRITE
from unicorn.m68k_const import *
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('rom',type=Path)
a=p.parse_args();rom=a.rom.read_bytes()
if hashlib.sha256(rom).hexdigest()!='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e':
    p.error('requires the supplied ROM revision')
root=Path(__file__).resolve().parents[1];binary=root/'target/release/shadowrealm'
def word(p):return struct.unpack_from('>H',rom,p)[0]
def long(p):return struct.unpack_from('>I',rom,p)[0]
def cpu_new():
    cpu=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);cpu.ctl_set_cpu_model(UC_CPU_M68K_M68000)
    for base,size in [(0,0x200000),(0x400000,0x1000),(0xff0000,0x10000),(0xc00000,0x1000)]:cpu.mem_map(base,size)
    cpu.mem_write(0,rom);cpu.reg_write(UC_M68K_REG_SR,0x2700)
    return cpu
cpu=cpu_new()
def call(address,entity=None):
    cpu.reg_write(UC_M68K_REG_A7,0xffff00);cpu.mem_write(0xffff00,struct.pack('>I',0x400000))
    if entity is not None:cpu.mem_write(0xff0100,entity)
    cpu.reg_write(UC_M68K_REG_A0,0xff0100)
    cpu.emu_start(address,0x400000,count=2_000_000)
    if cpu.reg_read(UC_M68K_REG_PC)!=0x400000:raise RuntimeError(f'routine {address:x} did not return')
with tempfile.TemporaryDirectory() as temp:
    catalog=Path(temp)/'actors.csv'
    subprocess.run([str(binary),'actor-catalog',str(a.rom.resolve()),str(catalog)],check=True)
    rows=list(csv.DictReader(catalog.open()))
    groups={};sequences=set()
    for row in rows:
        bank,state,index=[int(row[k]) for k in ['bank','state','frame_index']]
        table=long(0x62ac+bank*4);seq=long(table+state*4)
        sequences.add((bank,state))
        if int(row['sequence'])!=seq or int(row['speed'])!=word(seq+2) or (row['terminal']=='true')!=(word(seq)&0x8000!=0):
            raise RuntimeError('sequence header mismatch')
        frame=long(seq+4+index*4)
        if int(row['frame'])!=frame:raise RuntimeError('sequence pointer mismatch')
        groups.setdefault(frame,[])
        if int(row['piece'])>=0:
            value=tuple(int(row[k]) for k in ['x','y','width','height','attribute'])
            entry=(int(row['piece']),value)
            if entry not in groups[frame]:groups[frame].append(entry)
    for frame,entries in groups.items():
        entity=bytearray(256)
        for offset,value in [(0xa,128),(0xc,128),(0x20,0x100),(0x2c,0x2000)]:struct.pack_into('>H',entity,offset,value)
        struct.pack_into('>I',entity,0x12,frame)
        cpu.reg_write(UC_M68K_REG_A3,0xffb000);cpu.reg_write(UC_M68K_REG_D5,1)
        call(0x622c,bytes(entity))
        length=cpu.reg_read(UC_M68K_REG_A3)-0xffb000
        actual=[]
        for i in range(0,length,8):
            y,size_link,attribute,x=struct.unpack('>HHHH',bytes(cpu.mem_read(0xffb000+i,8)))
            size=size_link>>8
            # Coords wrap on 68000; convert back to signed piece-relative positions.
            dx=((x-128+32768)&65535)-32768;dy=((y-128+32768)&65535)-32768
            actual.append((dx,dy,((size>>2)&3)+1,(size&3)+1,attribute))
        expected=[value for _,value in sorted(entries)]
        if actual!=expected:raise RuntimeError(f'sprite builder mismatch for frame {frame:x}')
    # The command emits the actual native clock timeline, not a reimplementation.
    animation=Path(temp)/'animation'
    subprocess.run([str(binary),'render-animation',str(a.rom.resolve()),'0','5','8',str(animation),'96'],check=True,capture_output=True)
    timeline=list(csv.DictReader((animation/'frames.csv').open()))
    entity=bytearray(256)
    for offset,value in [(0xe,5),(0x16,8*4),(0x18,8*28),(0x1c,0x1000),(0x20,0x100),(0x2c,0x2000),(0xca,8)]:struct.pack_into('>H',entity,offset,value)
    cpu.mem_write(0xff0100,bytes(entity));cpu.mem_write(0xffdf2c,struct.pack('>I',0xff0000))
    for row in timeline:
        actual_entity=bytes(cpu.mem_read(0xff0100,256))
        phase,frame=struct.unpack_from('>H',actual_entity,0x1a)[0],struct.unpack_from('>H',actual_entity,0x1e)[0]
        if (phase,frame)!=(int(row['phase']),int(row['sequence_frame'])):raise RuntimeError('clock state mismatch')
        call(0x5fdc);call(0x600e)
        pointer=struct.unpack('>I',bytes(cpu.mem_read(0xff0112,4)))[0]
        seq=long(long(0x62ac+5*4)+8*4)
        frame=struct.unpack('>H',bytes(cpu.mem_read(0xff011e,2)))[0]
        if pointer!=long(seq+4+frame*4):raise RuntimeError('original selector pointer mismatch')
    # Execute the original HUD setup and health-table writer with VDP writes captured.
    cpu=cpu_new();vram=bytearray(65536);destination=[0]
    def control(cpu,address,size,user):destination[0]=cpu.reg_read(UC_M68K_REG_D0)&65535
    def write(cpu,access,address,size,value,user):
        if address==0xc00000:
            start=destination[0]
            vram[start:start+size]=value.to_bytes(size,'big');destination[0]=(start+size)&65535
    cpu.hook_add(UC_HOOK_CODE,control,begin=0xe9c,end=0xe9c)
    cpu.hook_add(UC_HOOK_MEM_WRITE,write,begin=0xc00000,end=0xc00003)
    call(0x334c)
    # Explicit fixture: three active slots with blank portrait definitions and no item.
    for slot in range(3):
        entity=bytearray(256);entity[0x10]=1
        struct.pack_into('>H',entity,0x3e,slot)
        struct.pack_into('>H',entity,0x2c,0x2000)
        struct.pack_into('>I',entity,0x50,0x34f8)
        cpu.mem_write(0xff0100+slot*256,bytes(entity))
    call(0x3404)
    for slot,health in enumerate([10,7,4]):
        entity=bytearray(256);entity[0x10]=1;entity[0x4c]=health
        struct.pack_into('>H',entity,0x3e,slot)
        cpu.reg_write(UC_M68K_REG_D1,0);call(0x3674,bytes(entity));call(0x36fc,bytes(entity))
    assets=Path(temp)/'hud'
    subprocess.run([str(binary),'hud-assets',str(a.rom.resolve()),str(assets)],check=True)
    if (assets/'window.bin').read_bytes()!=vram[0xf000:]:
        native=(assets/'window.bin').read_bytes()
        differences=[(hex(0xf000+i),hex(int.from_bytes(native[i:i+2],'big')),hex(int.from_bytes(vram[0xf000+i:0xf002+i],'big'))) for i in range(0,4096,2) if native[i:i+2]!=vram[0xf000+i:0xf002+i]]
        raise RuntimeError(f'HUD window mismatch: {differences[:20]}')
    if (assets/'vram.bin').read_bytes()[0xdac0:0xdf80]!=vram[0xdac0:0xdf80]:raise RuntimeError('HUD graphics mismatch')
    if (assets/'vram.bin').read_bytes()[0xd280:0xdac0]!=vram[0xd280:0xdac0]:raise RuntimeError('portrait/equipment graphics mismatch')
print(f'PASS: {len(sequences)} sequences; {len(groups)} distinct sprite frames match original $622c; 96 clock ticks match $5fdc/$600e; HUD assets match $334c/$3404/$3674/$36fc')
