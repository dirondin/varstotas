# @name Sketchfab (Python)
# @model_source search
# @description Downloadable models of Sketchfab (Creative Commons; the credit goes with the component). Searching is open; downloading needs your API token (sketchfab.com/settings/password). Type a search and press Enter. The protocol: `plugin.py search | download`, the arguments as JSON on stdin, the answer as JSON on the last line of stdout.
# @description.ru Скачиваемые модели Sketchfab (Creative Commons; указание автора сохраняется в компоненте). Поиск открыт; для загрузки нужен ваш API-токен (sketchfab.com/settings/password). Введите запрос и нажмите Enter. Протокол: `plugin.py search | download`, аргументы — JSON в stdin, ответ — JSON в последней строке stdout.
# @description.lt Atsisiunčiami Sketchfab modeliai (Creative Commons; autorystė išsaugoma komponente). Paieška atvira; atsisiuntimui reikia jūsų API žetono (sketchfab.com/settings/password). Įveskite užklausą ir paspauskite Enter. Protokolas: `plugin.py search | download`, argumentai – JSON per stdin, atsakymas – JSON paskutinėje stdout eilutėje.
# @param token:text = "" "API token"
# @param.ru token "API-токен"
# @param.lt token "API žetonas"
# @notice Models provided by Sketchfab (sketchfab.com). Each has its own licence (CC0, CC-BY, CC-BY-NC, Standard…): it goes with the component; credit the author and Sketchfab, and check NC / ND / Standard licences before commercial use.
# @notice.ru Модели предоставлены Sketchfab (sketchfab.com). У каждой своя лицензия (CC0, CC-BY, CC-BY-NC, Standard…): она сохраняется в компоненте; указывайте автора и Sketchfab и проверяйте лицензии NC / ND / Standard перед коммерческим использованием.
# @notice.lt Modelius teikia Sketchfab (sketchfab.com). Kiekvienas turi savo licenciją (CC0, CC-BY, CC-BY-NC, Standard…): ji išsaugoma komponente; nurodykite autorių ir Sketchfab ir prieš komercinį naudojimą patikrinkite NC / ND / Standard licencijas.
import json
import os
import subprocess
import sys
import urllib.parse


def get(url, out=None, token=""):
    cmd = ["curl", "-sSfL", "--max-time", "600", "--user-agent", "Varstotas", url]
    if token:
        cmd += ["-H", "Authorization: Token " + token]
    if out:
        cmd += ["-o", out]
    return subprocess.run(cmd, check=True, capture_output=True).stdout


def thumb(m):
    images = (m.get("thumbnails") or {}).get("images") or []
    small = [i for i in images if i.get("width", 0) >= 128] or images
    return min(small, key=lambda i: i.get("width", 0)).get("url", "") if small else ""


args = json.load(sys.stdin)
params = args.get("params", {})
command = sys.argv[1] if len(sys.argv) > 1 else ""

if command == "search":
    q = urllib.parse.quote(args.get("query", ""))
    found = json.loads(get("https://api.sketchfab.com/v3/search?type=models&downloadable=true&count=24&q=" + q)).get("results", [])
    print(json.dumps([
        {
            "id": m["uid"],
            "name": m.get("name") or m["uid"],
            "category": ", ".join(c.get("name", "") for c in m.get("categories", [])),
            "tags": [t.get("name", "") for t in m.get("tags", [])],
            "author": (m.get("user") or {}).get("displayName", ""),
            "license": ((m.get("license") or {}).get("label") or "Creative Commons"),
            "thumb": thumb(m),
            "page": m.get("viewerUrl", ""),
            "polycount": m.get("faceCount", 0),
        }
        for m in found
    ]))
elif command == "download":
    token = params.get("token", "")
    if not token:
        sys.exit("Set your Sketchfab API token (sketchfab.com/settings/password) to download")
    links = json.loads(get("https://api.sketchfab.com/v3/models/%s/download" % args["id"], token=token))
    if "glb" in links:
        path = os.path.join(args["dir"], args["id"] + ".glb")
        get(links["glb"]["url"], path)
        print(json.dumps({"file": path}))
    elif "gltf" in links:
        # A zip of scene.gltf with its textures.
        path = os.path.join(args["dir"], args["id"] + ".zip")
        get(links["gltf"]["url"], path)
        print(json.dumps({"zip": path}))
    else:
        sys.exit("no glTF download for this model")
else:
    sys.exit("unknown command " + command)
