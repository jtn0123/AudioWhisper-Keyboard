"""File-only ASR comparison and JNI-free replay. Install explicit fixture APKs first.

Required: ANDROID_SERIAL (owned runtime), --apk and --test-apk (sealed installed files).
The normal run creates a fresh device oracle; --native-only --oracle REPORT reuses its WAVs.
Every response contains only hashes, counts, timings and mapped-library booleans.
"""
import argparse,hashlib,json,os,runpy,subprocess,re
from pathlib import Path
root=Path(__file__).resolve().parents[1];os.chdir(root)
p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=Path);p.add_argument('--native-only',action='store_true');p.add_argument('--oracle',type=Path);p.add_argument('--apk',type=Path,required=True);p.add_argument('--test-apk',type=Path,required=True);args=p.parse_args();args.output.mkdir(parents=True,exist_ok=False)
serial=os.environ['ANDROID_SERIAL'];assert serial!='emulator-5554','Reserved emulator';pkg='com.jtn0123.audiowhisper.keyboard';adb=['adb','-s',serial]
runner=pkg+'.test/com.jtn0123.audiowhisper.dictation.DictationInstrumentation'
assert runner in subprocess.check_output(adb+['shell','pm','list','instrumentation',pkg],text=True)
resolver=runpy.run_path('scripts/render-benchmark/apk-maps.py')
images=resolver['images'](adb,pkg,args.apk)+resolver['images'](adb,pkg+'.test',args.test_apk)
for apk,lo,hi,name in images:
 assert name in resolver['loaded'](f'1000-2000 r-xp {lo:x} 00:00 1 {apk}\n'*101,images)
s=runpy.run_path('scripts/render-benchmark/store-session.py');maps=[]
with s['fixture_session'](args.output/'stores'):
 subprocess.run(adb+['shell','run-as',pkg,'rm','-f','files/dictation-comparison.json'],check=True)
 command=adb+['shell','am','instrument','-w']
 if args.native_only:command+=['-e','reuse','true','-e','nativeOnly','true','-e','timings','false']
 command+=[runner]
 with (args.output/'instrumentation.log').open('w') as log:
  process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
  for line in process.stdout:
   log.write(line);log.flush()
   match=re.fullmatch(r'INSTRUMENTATION_STATUS: fixture=(\d+)\n',line)
   if match:
    pid=subprocess.check_output(adb+['shell','pidof',pkg],text=True).strip();assert re.fullmatch(r'\d+',pid)
    raw=subprocess.check_output(adb+['exec-out','run-as',pkg,'cat','/proc/'+pid+'/maps'],text=True)
    names=resolver['loaded'](raw,images)
    record={'fixture':int(match[1]),'sherpa_jni_loaded':'libsherpa-onnx-jni.so' in names,'sherpa_c_api_loaded':'libsherpa-onnx-c-api.so' in names,'onnx_loaded':'libonnxruntime.so' in names}
    maps.append(record)
    (args.output/'maps.json').write_text(json.dumps(maps,indent=2)+'\n')
  assert process.wait()==0
 assert 'dictation_complete=true' in (args.output/'instrumentation.log').read_text()
 raw=subprocess.check_output(adb+['exec-out','run-as',pkg,'cat','files/dictation-comparison.json']);report=json.loads(raw)
 assert report['identical'] and len(report['fixtures'])==7
 assert report['production_listener']['identical'] and report['production_listener']['errors']==0
 assert report['native_only']==args.native_only
 assert len(report['timings'])==(0 if args.native_only else 20)
 assert len(maps)==7
 if args.native_only:
  assert args.oracle is not None
  assert report['oracle_sha256']==hashlib.sha256(args.oracle.read_bytes()).hexdigest()
  assert not any(x['sherpa_jni_loaded'] for x in maps)
  assert any(x['sherpa_c_api_loaded'] and x['onnx_loaded'] for x in maps)
 else:
  subprocess.run(adb+['shell','run-as',pkg,'cp','files/dictation-comparison.json','files/dictation-oracle.json'],check=True)
 (args.output/'comparison.json').write_bytes(raw)
 (args.output/'maps.json').write_text(json.dumps(maps,indent=2)+'\n')
 print(json.dumps({'identical':True,'fixtures':7,'timings':len(report['timings']),'native_only':args.native_only,'jni_mapped_samples':sum(x['sherpa_jni_loaded'] for x in maps),'report_sha256':hashlib.sha256(raw).hexdigest()}),flush=True)
