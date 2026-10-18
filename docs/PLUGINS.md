# Плагины и библиотека компонентов

## Плагины
Плагин — это файл `.rhai` или `.py` в папке `~/.varstotas/plugins/` (подпапки тоже просматриваются, на два уровня). Плагины загружаются при запуске; после правки нужно выбрать **Extensions → Reload Plugins**. Там же есть **Open Plugins Folder**.

Примеры лежат в `examples/plugins/`. При каждом запуске приложение копирует их в `~/.varstotas/plugins/examples/` — свои плагины держите в другой подпапке, иначе они будут перезаписаны:
- `stairs.rhai` — лестница: параметры и одна точка.
- `beam.rhai` — балка между двумя точками.
- `import_points_csv.rhai` — импорт контура из CSV.
- `export_faces_json.py` — экспорт граней в JSON на Python.
- `unity_material.rhai` — импорт материалов Unity (`.mat`), см. «Плагины материалов».
- `texture_folders.rhai` — источник материалов из папок с текстурами на диске.
- `ambientcg_python.py` — источник материалов на Python: поиск по ambientCG.

### Заголовок
Описание плагина — это комментарии в начале файла (`//` для Rhai, `#` для Python). Заголовок заканчивается на первой строке, которая не является комментарием.

```rhai
// @name Stairs
// @menu Construction/Stairs…
// @description A straight flight of treads rising along +Y from the clicked point.
// @param width = 900 "Width"
// @param steps:int = 15 "Steps"
// @pick 1 "Click the bottom front corner of the stairs"
```

| Тег | Смысл |
|---|---|
| `@name` | название (по умолчанию — имя файла) |
| `@menu A/B/C` | путь в меню **Extensions** (по умолчанию — название) |
| `@description` | подсказка и текст в диалоге |
| `@param key = default "Label"` | параметр, который пользователь вводит в диалоге перед запуском. Число по умолчанию — длина: можно вводить с единицами (`1.2m`), в скрипт приходит в мм. Тип задаётся явно так: `key:int`, `key:number`, `key:text`, `key:bool`, `key:length`. Строка в кавычках — текст, `true` / `false` — флажок. Последние введённые значения запоминаются до конца сеанса. |
| `@pick N "Prompt"` | перед запуском пользователь кликает N точек в модели, с привязками. Они приходят в массиве `points` (мировые мм). |
| `@select "Prompt"` | перед запуском (после точек) пользователь выбирает объекты кликами: грань, ребро или группа открытого контекста добавляется или убирается из выделения, Enter — продолжить, Esc — отмена. Объекты приходят в массиве `objects` (как сущности скрипта, тот же порядок, что в выделении); уже выделенное до запуска тоже попадает. |
| `@import csv,txt` | импортёр: пункт в **File → Import**, путь файла приходит в `path`. В меню Extensions такой плагин не попадает. |
| `@export json` | экспортёр: пункт в **File → Export**, путь приходит в `path` |
| `@material_import mat,material` | импортёр материалов: **Materials → Import…** читает эти файлы плагином (см. ниже) |
| `@material_source` | источник материалов в окне **Online Materials** (см. ниже) |

Параметры, `points`, `objects` и `path` — обычные переменные скрипта. В редакторе Script Console они выделены тёмно-бирюзовым цветом и есть в списке автодополнения: видно, что значение приходит из заголовка.

| Переменная | Откуда | Что внутри |
|---|---|---|
| ключ `@param` | диалог | число (длина — в мм), текст или `true` / `false` |
| `points` | `@pick N` | массив из N точек (мировые мм) |
| `objects` | `@select` | массив сущностей (грани, рёбра, группы) |
| `path` | `@import` / `@export` | путь выбранного файла |

Тот же заголовок работает в коде **Script Console**: Run с `@param` / `@pick` / `@select` проходит диалог, клики и выбор, затем выполняет код консоли:

```rhai
// @param h = 200 "Height"
// @select "Select faces to raise"
for o in objects { pushpull(o, h); }
```

### Rhai
Запуск Rhai-плагина — один шаг Undo с названием плагина; если плагин упал с ошибкой, модель не меняется. Вывод `print` попадает в Script Console, последняя строка появляется в строке состояния. Для импортёров и экспортёров в API есть `read_text(path)`, `write_text(path, text)`, `file_exists`, `list_files(dir, ext)`, `list_dirs(dir)`, `parse_json(text)`, `fetch_text(url)`, `fetch_file(url, path)` и `all_faces()` — все грани модели, включая вложенные группы (`f.vertices` возвращает мировые координаты).

### Python
Python-плагин запускается так же, как File → Run Script…: через `python3`, вывод идёт в Script Console. Значения приходят через `varstotas.params()`:

```python
# @name Faces as JSON
# @export json
import varstotas
m = varstotas.connect()
path = varstotas.params()["path"]
```

Изменения модели делайте внутри `with m.operation("…")`, чтобы они стали одним шагом Undo.

### Плагины материалов
Два вида плагинов, в меню Extensions их нет.

**Импортёр** (`@material_import ext,…`). Расширения плагина появляются в фильтре диалога **Materials → Library → Import…** («All materials» и отдельный фильтр плагина). Материалы из файла добавляются в документ; последний становится текущим для Paint. `@param` такого плагина берут значения по умолчанию.

**Источник** (`@material_source`). Плагин появляется в выпадающем списке **Source** окна **Online Materials** (после Poly Haven и ambientCG). Под списком показаны описание плагина и его `@param`. Кнопка **Load** (или Enter в поле) заново запрашивает каталог с этими значениями. У параметров с `folder` / `dir` / `path` / `file` в имени есть кнопка «…» для выбора, и такой путь передаётся абсолютным. Дальше всё как у встроенных источников: поиск, категории, превью, Download & Use / Download / Use / Web Page, двойной клик. Скачанное сохраняется в библиотеку, в `<library>/<название плагина>/`.

Функции плагина на Rhai (код выполняется отдельно от модели, в фоне; файлами и сетью занимаются функции API):

```rhai
// @name My Site
// @material_source
// @param api_key:text = "" "API key"
fn catalogue(params) {            // → массив материалов
    let list = parse_json(fetch_text("https://example.com/api?key=" + params.api_key));
    list.map(|a| #{ id: a.id, name: a.title, category: a.kind, tags: a.tags, width: 2000,
                    thumb: a.preview, page: a.url, license: "CC0" })
}
fn download(id, dir, params) {    // → спецификация материала
    fetch_file("https://example.com/files/" + id + ".zip", dir + "/m.zip");
    #{ zip: "m.zip" }
}

// импортёр: @material_import mat
fn import_material(path, params) { … }   // → спецификация или массив спецификаций
```

Поля материала в каталоге: обязателен `id`; дальше `name`, `category`, `keywords` / `tags` (строка или массив, по ним работает поиск), `width` (реальная ширина текстуры, мм), `thumb` (URL или путь к картинке), `page` (страница для Web Page), `license`. `dir` в `download` — рабочая папка `~/.varstotas/cache/online/<плагин>/<id>`, относительные пути ответа отсчитываются от неё. У импортёра относительные пути отсчитываются от папки файла.

**Python**: `python3 plugin.py catalogue|download|import`. Аргументы приходят JSON-объектом в stdin: `params`, `id`, `dir`, `path`. Ответ — JSON в последней строке stdout; прочие строки уходят в лог. Ошибка — ненулевой код выхода, текстом служит последняя строка stderr (`sys.exit("сообщение")`). Пример: `ambientcg_python.py`.

**Спецификация материала** — JSON-объект (в Rhai — map):

| Поле | Смысл |
|---|---|
| `name` | имя (по умолчанию — имя из каталога / файла) |
| `width` | ширина текстуры, мм |
| `color` | `"#rrggbb"` или `[r, g, b]` (0..1, или 0..255); с картой цвета по умолчанию — средний цвет карты |
| `opacity` | 0..1 |
| `maps` | `{ color, normal, roughness, ao, metallic, emission }` — пути к картинкам (PNG, JPEG, TGA, BMP; normal — OpenGL); `texture` — то же, что `maps.color` |
| `folder`, `zip` | папка / zip-архив, карты в которых опознаются по именам: `_color` / `_diff` / `_albedo` / `_basecolor`, `_normal` / `_nor_gl` / `_nrm`, `_rough`, `_ao` / `_occlusion`, `_metal` / `_metalness`, `_emission`. DirectX-нормали (`_dx`) и карты высоты пропускаются; если карты цвета по имени нет, ей становится первая неопознанная картинка |
| `metallic`, `roughness`, `transmission`, `ior`, `emission` (цвет), `emission_strength`, `normal_scale`, `clearcoat`, `clearcoat_roughness`, `sheen`, `sheen_tint`, `anisotropy`, `anti_tiling`, `bevel` (мм) | физические параметры (Physical в карточке Materials) |

Пример импортёра — `unity_material.rhai`. Он читает `.mat` (Standard / URP / HDRP Lit): `m_Name`, `_BaseColor` / `_Color`, `_Metallic`, `_Smoothness` / `_Glossiness` (roughness = 1 − smoothness), `_BumpScale`, `_EmissionColor` и текстуры `_BaseMap` / `_MainTex`, `_BumpMap`, `_OcclusionMap`, `_EmissionMap`. Текстуры ищутся по GUID в `.meta`-файлах папки `Assets` проекта. Ширину текстуры задаёт параметр `width`. PSD / TIFF и `_MetallicGlossMap` (металл в канале R) не переносятся.

## Библиотека компонентов
Раздел **Components** в Default Tray (включается галочкой в меню **Window**) показывает файлы `.vrst` из папки библиотеки. По умолчанию это `~/.varstotas/components`; кнопка «…» выбирает другую папку, «Default» в заголовке корневой папки возвращает папку по умолчанию. Подпапки показываются как категории со стрелкой сворачивания.

- **Клик по эскизу** ставит содержимое файла как компонент: он следует за курсором, углом габарита к точке привязки, клик фиксирует. Определение в документе одно, сколько бы раз компонент ни ставили (сопоставление по имени файла). Esc отменяет.
- **Save Selection to Library…** сохраняет выделенное в `<папка>/<имя>.vrst` под новым именем, если такое уже занято. Начало координат файла — угол габарита. Если выделена одна группа или компонент, в файл попадает её содержимое.
- Эскиз — каркас в изометрии. Подсказка над эскизом показывает размеры. Поиск фильтрует по имени. Кнопка ⟳ перечитывает папку; изменённые файлы перечитываются и при обычном показе раздела.

Параметрические компоненты, которые строятся скриптом (`@generator`), описаны в `docs/GENERATORS.md`.
