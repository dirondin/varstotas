# @name Marble
# @name.ru Мрамор
# @name.lt Marmuras
# @name.be Мармур
# @texture_generator 256
# @description Veined marble (pure Python, no packages): soft veins bent by fractal noise, polished.
# @description.ru Мрамор с прожилками (чистый Python, без пакетов): мягкие прожилки, изогнутые фрактальным шумом, полированный.
# @description.lt Gyslotas marmuras (grynas Python, be paketų): švelnios gyslos, išlenktos fraktaliniu triukšmu, poliruotas.
# @description.be Мармур з пражылкамі (чысты Python, без пакетаў): мяккія пражылкі, выгнутыя фрактальным шумам, паліраваны.
# @param veins:int = 3 "Veins across"
# @param.ru veins "Прожилок поперёк"
# @param.lt veins "Gyslų skersai"
# @param.be veins "Пражылак упоперак"
# @param turbulence:number = 1.2 "Turbulence"
# @param.ru turbulence "Извилистость"
# @param.lt turbulence "Vingiuotumas"
# @param.be turbulence "Звілістасць"
# @param base:text = "#ece8e1" "Stone colour"
# @param.ru base "Цвет камня"
# @param.lt base "Akmens spalva"
# @param.be base "Колер каменю"
# @param vein:text = "#5d5a57" "Vein colour"
# @param.ru vein "Цвет прожилок"
# @param.lt vein "Gyslų spalva"
# @param.be vein "Колер пражылак"
# @param seed:int = 7 "Variant"
# @param.ru seed "Вариант"
# @param.lt seed "Variantas"
# @param.be seed "Варыянт"
#
# Protocol: `python3 marble.py texture`, stdin {"size", "params", "dir"}; the answer is a
# material spec as JSON on the last stdout line. The colour map goes inline as base64 RGB
# (one could also write a PNG into "dir" and answer {"texture": "marble.png"}).
import base64, json, math, sys


def lattice(i, j, period, seed):
    i, j = i % period, j % period
    h = (i * 374761393 + j * 668265263 + seed * 2147483647) & 0xFFFFFFFF
    h = ((h ^ (h >> 13)) * 1274126177) & 0xFFFFFFFF
    return ((h ^ (h >> 16)) & 0xFFFF) / 65535.0


def noise(x, y, period, seed):
    """Value noise repeating every `period` cells."""
    xi, yi = math.floor(x), math.floor(y)
    fx, fy = x - xi, y - yi
    sx, sy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)
    a = lattice(xi, yi, period, seed)
    b = lattice(xi + 1, yi, period, seed)
    c = lattice(xi, yi + 1, period, seed)
    d = lattice(xi + 1, yi + 1, period, seed)
    return (a + (b - a) * sx) + ((c + (d - c) * sx) - (a + (b - a) * sx)) * sy


def fbm(x, y, period, seed, octaves=4):
    s, amp, norm, k = 0.0, 0.5, 0.0, 1
    for o in range(octaves):
        s += noise(x * k, y * k, period * k, seed + o) * amp
        norm += amp
        amp *= 0.5
        k *= 2
    return s / norm


def hex_rgb(s, default):
    s = str(s).strip().lstrip("#")
    try:
        return [int(s[i:i + 2], 16) for i in (0, 2, 4)]
    except ValueError:
        return default


def texture(size, params):
    veins = max(int(params.get("veins", 3)), 1)
    turb = float(params.get("turbulence", 1.2))
    seed = int(params.get("seed", 7))
    base = hex_rgb(params.get("base", ""), [236, 232, 225])
    vein = hex_rgb(params.get("vein", ""), [93, 90, 87])
    out = bytearray(size * size * 3)
    i = 0
    for y in range(size):
        v = (y + 0.5) / size
        for x in range(size):
            u = (x + 0.5) / size
            # veins: a diagonal sine (whole periods along u and v: seamless), bent by noise
            t = fbm(u * 2, v * 2, 2, seed, 5)
            s = math.sin((u * veins + v + turb * (t - 0.5)) * 2 * math.pi)
            line = (1 - abs(s)) ** 14  # thin dark lines where the sine crosses zero
            cloud = 0.94 + 0.08 * fbm(u * 8, v * 8, 8, seed + 11, 3)
            for c in range(3):
                val = (base[c] * cloud) * (1 - line) + vein[c] * line
                out[i] = max(0, min(255, int(val)))
                i += 1
    return {
        "name": "Marble",
        "texture": {"size": size, "rgb": base64.b64encode(bytes(out)).decode()},
        "roughness": 0.15,
        "clearcoat": 0.3,
        "width": 1200,
    }


if __name__ == "__main__":
    args = json.load(sys.stdin)
    if sys.argv[1] != "texture":
        sys.exit("unknown call " + sys.argv[1])
    print(json.dumps(texture(int(args.get("size", 256)), args.get("params", {}))))
