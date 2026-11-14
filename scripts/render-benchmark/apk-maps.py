"""Resolve ELF mappings backed by uncompressed APK members, rather than filename guesses."""
from pathlib import PurePosixPath
import hashlib,stat,struct,subprocess,zipfile

def images(adb, package, apk):
 raw=subprocess.check_output(adb+['shell','pm','path',package],text=True).strip().splitlines()
 assert len(raw)==1 and raw[0].startswith('package:');remote=raw[0][8:]
 actual=subprocess.check_output(adb+['shell','sha256sum',remote],text=True).split()[0]
 assert actual==hashlib.sha256(apk.read_bytes()).hexdigest()
 result=[]
 with zipfile.ZipFile(apk) as z:
  entries=z.infolist()
  for e in entries:
   p=PurePosixPath(e.filename);assert not p.is_absolute() and '..' not in p.parts
   assert stat.S_IFMT(e.external_attr>>16) in (0,stat.S_IFREG,stat.S_IFDIR)
   assert e.file_size<512_000_000
  assert sum(e.file_size for e in entries)<1_000_000_000
  with apk.open('rb') as f:
   for e in entries:
    if not e.filename.endswith('.so'):continue
    assert e.compress_type==zipfile.ZIP_STORED
    f.seek(e.header_offset);header=f.read(30);assert header[:4]==b'PK\x03\x04'
    name,extra=struct.unpack_from('<HH',header,26)
    offset=e.header_offset+30+name+extra
    result.append((remote,offset,offset+e.file_size,e.filename.rsplit('/',1)[-1]))
 return result

def loaded(raw, entries):
 names=set();valid=0
 for line in raw.splitlines():
  fields=line.split(maxsplit=5)
  if len(fields)<5 or '-' not in fields[0]:continue
  try:int(fields[0].split('-')[0],16);offset=int(fields[2],16)
  except ValueError:continue
  valid+=1
  if len(fields)<6:continue
  path=fields[5].removesuffix(' (deleted)')
  if path.endswith('.so'):names.add(path.rsplit('/',1)[-1])
  for apk,lo,hi,name in entries:
   if path==apk and lo<=offset<hi:names.add(name)
 assert valid>100,'Invalid process-map observation'
 return names
