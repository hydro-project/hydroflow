To collect memory/latency versus cluster size:
```bash
$ hydro deploy topolotree_latency.hydro.py -- gcp pn,pn_delta,topolo 2,3,4,5,6,7,8 1 1
```

To collect latency vs throughput:
```bash
$ hydro deploy topolotree_latency.hydro.py -- gcp pn,pn_delta,topolo 6 1/1,2/1,4/1,8/1,16/1,32/1,64/1,128/1,256/1,512/1,1024/1,1024/2,1024/4,1024/8
```