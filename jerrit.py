import os
import sys
import json

gUsageStr="""
Usage: """+sys.argv[0]+""" <file> <opt>

<opt> optional args:

-h        - Display this usage info and exit
-sf <str> - Optional, used to find <any string> in comments in json file.
            This MUST always be given as last argument!
"""

# Validate argument count
if len (sys.argv) < 1 : sys.exit (gUsageStr)

# Save and setup env
fname=sys.argv[1]
mainJF=""
gStatus=""
gProj=""
gBranch=""
gRefs=""
gCurPatchSet=12321
gCommitID=""
gFindStr=""
f=''

args = sys.argv[1:]
while len(args):
    if args[0] == '-h':
        sys.exit (gUsageStr)
    elif args[0] == '-sf':
        args = args[1:] # shift to start of string 
        gFindStr=args[0]
        args = args[1:] # shift to 2nd word in string
        while len(args):
            gFindStr=gFindStr+" "+args[0]
            args = args[1:] # shift to next word in string
    args = args[1:] # shift to next arg

# Open JSON file
try:
    f=open(fname,)
    mainJF=json.load(f)
except Exception as e:
    print (str(e))
    if f: f.close()
    sys.exit (-1)

gStatus=mainJF["status"]
print ("status: "+gStatus)

gProj = mainJF["project"]
print ("project: "+gProj)

gBranch = mainJF["branch"]
print ("branch: "+gBranch)

gCurPatchSet = str(mainJF["currentPatchSet"]["number"])
print ("currentPatchSet: "+gCurPatchSet)

gCommitID = mainJF["currentPatchSet"]["revision"]
print ("commit: "+gCommitID)

gRefs = mainJF["currentPatchSet"]["ref"]
print ("refs: "+gRefs)

if len(gFindStr):
    item_dict = json.loads(json.dumps(mainJF['comments']))
    for i in range (len(item_dict)-1, -1, -1):
        lstr = str(item_dict[i]['message'])
        if gFindStr in lstr:
            #print ("---------"+str(i))
            print (lstr)

f.close()
sys.exit(0)

