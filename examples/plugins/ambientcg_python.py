# @name ambientCG Search (Python)
# @material_source
# @description An example material source in Python: ambientCG materials matching a search, downloaded as a zip of 1K maps. The protocol: `plugin.py catalogue | download`, the arguments as JSON on stdin, the answer as JSON on the last line of stdout.
# @param search:text = "metal" "Search"
# @notice ambientCG materials are CC0: free for any use, also commercial; credit optional.
import json
import os
import subprocess
import sys
import urllib.parse


def get(url, out=None):
    cmd = ["curl", "-sSfL", "--max-time", "180", "--user-agent", "Varstotas", url]
    if out:
        cmd += ["-o", out]
    return subprocess.run(cmd, check=True, capture_output=True).stdout


args = json.load(sys.stdin)
params = args.get("params", {})
command = sys.argv[1] if len(sys.argv) > 1 else ""

if command == "catalogue":
    q = urllib.parse.quote(params.get("search", ""))
    url = "https://ambientcg.com/api/v2/full_json?type=Material&limit=200&include=displayData,dimensionsData,previewData&sort=Popular&q=" + q
    found = json.loads(get(url)).get("foundAssets", [])
    print(json.dumps([
        {
            "id": a["assetId"],
            "name": a.get("displayName") or a["assetId"],
            "category": a.get("displayCategory", ""),
            "tags": a.get("tags", []),
            "width": (a.get("dimensionX") or 100) * 10,
            "thumb": a.get("previewImage", {}).get("128-PNG", ""),
            "page": "https://ambientcg.com/view?id=" + a["assetId"],
            "license": "CC0",
        }
        for a in found
    ]))
elif command == "download":
    zip_path = os.path.join(args["dir"], args["id"] + "_1K-JPG.zip")
    get("https://ambientcg.com/get?file=%s_1K-JPG.zip" % args["id"], zip_path)
    # The maps are found in the zip by their names (…_Color, …_NormalGL, …_Roughness).
    print(json.dumps({"zip": zip_path}))
else:
    sys.exit("unknown command " + command)
