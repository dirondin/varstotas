# Check Sofa 02 corner UVs against the downloaded Poly Haven glTF (asset not bundled).
import json,math,pathlib,struct,sys
if len(sys.argv)!=3: raise SystemExit('usage: sofa_uv_check.py sofa_02_1k.gltf model.vrst')
source=pathlib.Path(sys.argv[1])
g=json.loads(source.read_text());buffers=[(source.parent/b['uri']).read_bytes() for b in g['buffers']]
def read(a):
 a=g['accessors'][a];v=g['bufferViews'][a['bufferView']];n={'SCALAR':1,'VEC2':2,'VEC3':3}[a['type']];fmt={5126:'f',5125:'I',5123:'H'}[a['componentType']];width=struct.calcsize(fmt)*n;stride=v.get('byteStride',width);start=v.get('byteOffset',0)+a.get('byteOffset',0);return [struct.unpack_from('<'+fmt*n,buffers[v['buffer']],start+i*stride) for i in range(a['count'])]
def key(p):return tuple(round(x,4) for x in p)
source_faces={}
source_triangles=[]
for node in g['nodes']:
 for prim in g['meshes'][node['mesh']]['primitives']:
  pp=read(prim['attributes']['POSITION']);uv=read(prim['attributes']['TEXCOORD_0']);idx=[x[0] for x in read(prim['indices'])];t=node.get('translation',[0,0,0]);pts=[((p[0]+t[0])*1000,-(p[2]+t[2])*1000,(p[1]+t[1])*1000) for p in pp]
  for i in range(0,len(idx),3):
   tri=idx[i:i+3];source_triangles.append([(pts[k],(uv[k][0],-uv[k][1])) for k in tri]);source_faces[tuple(sorted(key(pts[k]) for k in tri))]={key(pts[k]):(uv[k][0],-uv[k][1]) for k in tri}
import itertools, collections
buckets=collections.defaultdict(list)
cell=lambda points: tuple(math.floor(sum(p[i] for p in points)/3.0) for i in range(3))
for tri in source_triangles:buckets[cell([p for p,uv in tri])].append(tri)
def fallback(points):
 c=cell(points);best=None
 for delta in itertools.product([-1,0,1],repeat=3):
  for tri in buckets.get(tuple(a+b for a,b in zip(c,delta)),[]):
   for ordered in itertools.permutations(tri):
    errors=[sum((x-y)**2 for x,y in zip(p,q)) for p,(q,uv) in zip(points,ordered)]
    if max(errors)<1e-8 and (best is None or sum(errors)<best[0]):best=(sum(errors),ordered)
 return {key(p):uv for p,(q,uv) in zip(points,best[1])} if best else None

def dot(a,b):return sum(x*y for x,y in zip(a,b))
def cross(a,b):return(a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def unit(v):
 n=math.sqrt(dot(v,v));return tuple(x/n for x in v)
def values(ss):return [x['value'] for x in ss if x.get('value') is not None]
path=pathlib.Path(sys.argv[2])
model=json.loads(path.read_text())['model'];errors=[];missing=0
for d in values(model['defs']):
 mesh=d['mesh']
 for face in values(mesh['faces']):
  pts=[mesh['vertices'][k['idx']]['value']['pos'] for k in face['outer']];src=source_faces.get(tuple(sorted(key(p) for p in pts)))
  if src is None:src=fallback(pts)
  if src is None:missing+=1;continue
  n=face['normal'];u=((1 if n[2]>=0 else -1),0,0) if abs(n[2])>.999 else unit(cross((0,0,1),n));v=(0,1,0) if abs(n[2])>.999 else cross(n,u)
  pl=face.get('uv') or {};off=pl.get('offset',[0,0]);scale=pl.get('scale',[1,1]);angle=-math.radians(pl.get('angle_deg',0));co=math.cos(angle);si=math.sin(angle)
  for p in pts:
   x=dot(p,u)-off[0];y=dot(p,v)-off[1];rx=x*co-y*si;ry=x*si+y*co
   actual=(rx/scale[0]/1000,(ry+pl.get('shear',0)*rx)/scale[1]/1000);want=src[key(p)];errors.append(max(abs(a-b) for a,b in zip(actual,want)))
print(path.name,'corners',len(errors),'unmatched faces',missing,'UV max error',max(errors),'mean',sum(errors)/len(errors),'bad corners',sum(e>1e-5 for e in errors))

assert missing==0 and max(errors)<1e-8, "Imported UV coordinates differ from Sofa 02 source"
