# Тэксты інтэрфейсу Varstotas: беларуская мова.
# Раздзелы адпавядаюць абласцям ключоў (першае слова ключа). Гл. docs/LOCALIZATION.md.

## ======== Агульнае ========

language-name = Беларуская

# Сістэмны файлавы менеджар, для пунктаў «Паказаць у …» / «Адкрыць папку …».
file-manager = файлавы менеджар
    .macos = Finder
    .windows = Правадыр

# Паказаць файл у сістэмным файлавым менеджары.
reveal-in-file-manager = Паказаць у файлавым менеджары
    .macos = Паказаць у Finder
    .windows = Паказаць у Правадыры

ok = OK
cancel = Скасаваць
close = Закрыць
apply = Ужыць
delete = Выдаліць

## ======== Праграма ========

app-edit-dimension-text = Рэдагаваць тэкст памеру
app-dimension-text-hint = Пусты тэкст вяртае вымераную даўжыню; `<>` пазначае яе.
# { $step } is the grid step with its unit.
app-grid-step = Крок сеткі { $step }
# { $input } is what was typed.
app-invalid-grid-step = Няправільны крок сеткі: { $input }
app-invalid-value = Няправільнае значэнне: { $input }
app-undo-move-axes = Перамясціць восі
app-no-credits = У гэтай мадэлі няма спампаваных кампанентаў з указаннем аўтарства
# { $name } is the undo name.
app-flash-undo = Адмена: { $name }
app-nothing-to-undo = Няма чаго адмяняць
app-flash-redo = Паўтор: { $name }
app-nothing-to-redo = Няма чаго вяртаць
app-select-to-group = Спачатку вылучыце тое, што трэба згрупаваць
app-undo-make-group = Стварыць групу
app-undo-make-component = Стварыць кампанент
app-select-to-explode = Вылучыце групу або кампанент, каб разбіць іх
app-undo-explode = Разбіць
app-undo-hide = Схаваць
app-undo-unhide-all = Паказаць усё
# { $name } is the tool's name.
app-not-implemented = «{ $name }» яшчэ не рэалізаваны
app-undo-delete = Выдаліць
app-cant-move-into-itself = Нельга перамясціць групу або кампанент у саміх сябе
app-undo-move-out-of-group = Вынесці з групы
app-undo-move-into-group = Перамясціць у групу
app-undo-add-scene = Дадаць сцэну
app-undo-update-scene = Абнавіць сцэну
app-undo-delete-scene = Выдаліць сцэну
app-undo-move-scene = Перамясціць сцэну
app-undo-cut = Выразаць
app-no-guides = Накіравальных няма
app-snapping-on = Прывязка ўключана
app-snapping-off = Прывязка выключана
app-select-one-component = Вылучыце адзін кампанент
app-undo-make-unique = Зрабіць унікальным
# { $count } is the number of plugins.
app-plugins-loaded = Загружана плагінаў: { $count }
app-select-one-group-or-component = Вылучыце адну групу або адзін кампанент
app-undo-component-parameters = Параметры кампанента
app-undo-make-parametric = Зрабіць параметрычным
app-undo-active-cut = Актыўны зрэз
app-undo-reverse-section = Разгарнуць зрэз
app-undo-soften-edges = Згладзіць рэбры
app-select-faces-to-reverse = Вылучыце грані, якія трэба перавярнуць
app-undo-reverse-faces = Перавярнуць грані
app-select-face-to-orient = Вылучыце грань, каб арыентаваць суседнія грані так жа
app-undo-orient-faces = Арыентаваць грані
app-faces-agree = Усе звязаныя грані ўжо арыентаваны аднолькава
# { $count } is the number of faces.
app-faces-reversed = { $count ->
        [one] Перавернута { $count } грань
        [few] Перавернута { $count } грані
        [many] Перавернута { $count } граней
       *[other] Перавернута { $count } грані
    }
app-move-axes-hint = Зрушэнне па чырвонай, зялёнай, сіняй восі і неабавязковы паварот вакол сіняй: `dx, dy, dz[, вугал]`.
app-undo-reset-axes = Скінуць восі
app-select-to-flip = Вылучыце тое, што трэба адлюстраваць
app-undo-flip-red = Адлюстраваць па чырвонай
app-undo-flip-green = Адлюстраваць па зялёнай
app-undo-flip-blue = Адлюстраваць па сіняй
app-select-to-intersect = Вылучыце грані, групы або кампаненты для перасячэння
app-undo-intersect-model = Перасячэнне з мадэллю
app-undo-intersect-selection = Перасячэнне з вылучаным
app-undo-intersect-context = Перасячэнне з кантэкстам
app-no-intersections = Перасячэнняў не знойдзена
# { $count } is the number of edges.
app-intersections-created = { $count ->
        [one] Створана { $count } рабро перасячэння
        [few] Створана { $count } рабры перасячэння
        [many] Створана { $count } рэбраў перасячэння
       *[other] Створана { $count } рабра перасячэння
    }
app-nothing-to-copy = Няма чаго капіраваць: нічога не вылучана
# { $error } is the error text.
app-copy-failed = Не атрымалася скапіраваць: { $error }
app-text-no-characters = У тэксце няма бачных знакаў
app-clipboard-empty = Буфер абмену пусты
# { $factor } is the scale factor.
app-resized = Маштаб зменены ×{ $factor }
app-scene-update = Абнавіць
app-scene-add = Дадаць
app-scene-delete = Выдаліць
app-scene-move-left = Улева
app-scene-move-right = Управа
app-pause-animation = Прыпыніць анімацыю
app-play-animation = Прайграць анімацыю
# { $path } is a file path, { $error } the error text.
app-cant-use = Немагчыма выкарыстаць { $path }: { $error }
app-undo-add-material = Дадаць матэрыял
app-dropped-material = Дададзены матэрыял { $name }: пстрыкніце па грані, каб пафарбаваць яе
app-no-scenes = Няма сцэн для прайгравання. Дадайце сцэны ў панэлі «Сцэны».
# { $path } is a file path.
app-opened = Адкрыта: { $path }
app-could-not-open = Не атрымалася адкрыць { $path }: { $error }
app-saved = Захавана: { $path }
app-save-failed = Не атрымалася захаваць: { $error }
# { $warning } is a warning text.
app-exported-warning = Экспартавана: { $path }. { $warning }
app-exported = Экспартавана: { $path }
app-export-failed = Не атрымалася экспартаваць: { $error }
# { $format } is the format name.
app-undo-import = Імпарт { $format }
app-imported = Імпартавана: { $path }
app-autosave-failed = Не атрымалася аўтазахаваць у { $path }: { $error }
# { $name } is a file name.
app-recovered = Адноўлена з аўтазахавання: { $name }
app-could-not-recover = Не атрымалася аднавіць { $name }: { $error }

## File dialog filters

app-filter-image = Відарыс
app-filter-script = Скрыпт
app-filter-model = Мадэль Varstotas

## ======== Меню ========

menu-intersect-faces = Перасекчы грані
menu-flip-along = Адлюстраваць па
menu-axes-place = Размясціць
menu-axes-move = Перамясціць…
menu-axes-reset = Скінуць
menu-active-cut = Актыўны зрэз
menu-reverse = Разгарнуць
menu-align-view = Выраўнаваць выгляд
menu-unfold = Разгортка…
menu-texture = Тэкстура
menu-texture-position = Размяшчэнне
menu-uv-editor = UV-рэдактар…
menu-mark-seam = Адзначыць як шво
menu-clear-seam = Зняць шво
menu-file = Файл
menu-new-from-template = Новы з шаблона
menu-open-recent = Адкрыць нядаўні
menu-preview-image = Відарыс папярэдняга прагляду
menu-preview-image-hint = Малюнак, які бібліятэка «Кампаненты» паказвае для гэтага файла (інакш яна малюе мадэль)
menu-import = Імпарт
menu-export = Экспарт
menu-unfold-export = Разгортка (SVG / PDF / DXF)…
menu-edit = Праўка
menu-view = Выгляд
menu-face-style = Стыль граней
menu-shaded = З запаўненнем
menu-shaded-textures = З запаўненнем і тэкстурамі
menu-monochrome = Манахром
menu-hidden-line = Схаваныя лініі
menu-wireframe = Каркас
menu-realistic = Рэалістычны
menu-x-ray = Рэнтген
menu-edge-style = Стыль рэбраў
menu-edges = Рэбры
menu-edges-hint = Рэбры ў гэтым стылі граней (у рэалістычным рэжыме ёсць свой пераключальнік)
menu-profiles = Контуры
menu-guides = Накіравальныя
menu-axes = Восі
menu-ground = Зямля
menu-sky = Неба
menu-sky-hint = Градыент неба за мадэллю; выключана — звычайны светлы фон
menu-grid = Сетка
menu-shadows = Цені
menu-section-planes = Секушчыя плоскасці
menu-section-cuts = Зрэзы
menu-section-fill = Запаўненне зрэзаў
menu-viewports = Выгляды
menu-viewports-one = Адзін
menu-viewports-two = Два (побач)
menu-viewports-four = Чатыры (2 × 2)
menu-presets = Прэсеты
menu-animation = Анімацыя
menu-pause = Паўза
menu-play = Прайграць
menu-next-scene = Наступная сцэна
menu-previous-scene = Папярэдняя сцэна
menu-add-scene = Дадаць сцэну
menu-camera = Камера
menu-standard-views = Стандартныя віды
menu-parallel-projection = Паралельная праекцыя
menu-perspective = Перспектыва
menu-draw = Маляваць
menu-shapes = Фігуры
menu-tools = Інструменты
menu-solid-tools = Цвёрдацельныя аперацыі
menu-snapping = Прывязкі
menu-extensions = Плагіны
menu-window = Акно
menu-side-bar = Бакавая панэль
menu-sheets = Лісты…
menu-help = Даведка
menu-user-guide = Кіраўніцтва карыстальніка
menu-user-guide-hint = Як карыстацца Varstotas: інструменты, меню, файлы, клавішы (F1 над мадэллю: актыўны інструмент)
menu-examples = Прыклады
menu-examples-hint = Адкрыць прыклады ў рэпазіторыі ў браўзеры
menu-script-reference = Даведнік па скрыптах
menu-script-reference-hint = Скрыпты, генератары і скрыпты кампанентаў: функцыі, уласцівасці, прыклады
menu-licences = Ліцэнзіі…
menu-licences-hint = Ліцэнзія Varstotas, кампаненты з адкрытым кодам, фарматы файлаў і анлайн-бібліятэкі з іх умовамі
menu-about = Пра Varstotas
menu-increments = Прырашчэнні
menu-all-on = Уключыць усе
menu-all-off = Выключыць усе
menu-show-grid = Паказваць сетку
menu-increments-move = Перамяшчэнне
menu-increments-rotate = Паварот
menu-increments-scale = Маштабаванне
menu-erase = Сцерці
# { $name } is the undo name of the last change.
menu-undo-named = Адмяніць: { $name }
menu-redo-named = Вярнуць: { $name }

## ======== Каманды ========

cmd-new = Новы
cmd-open = Адкрыць…
cmd-save = Захаваць
cmd-save-as = Захаваць як…
cmd-preview-show = Паказаць…
cmd-preview-from-file = З файла відарыса…
cmd-preview-from-view = З бягучага выгляду
cmd-preview-remove = Выдаліць
cmd-import-obj = OBJ…
cmd-import-stl = STL…
cmd-import-dxf = DXF…
cmd-import-gltf = glTF / GLB…
cmd-mf = 3MF…
cmd-import-dae = COLLADA (DAE)…
cmd-import-fbx = FBX…
cmd-import-ply = PLY…
cmd-import-csv = Кропкі (CSV)…
cmd-export-image = 2D-графіка (PNG)…
cmd-export-animation = Анімацыя (GIF / PNG)…
cmd-export-glb = glTF (GLB)…
cmd-export-drawing = 2D-чарцёж (SVG / PDF)…
cmd-sheets = Лісты (PDF / SVG)…
cmd-export-report = Справаздача: дэталі і плошчы (CSV)…
cmd-export-faces-json = Грані (JSON)…
cmd-export-attributions = Указанні аўтарства (аўтары спампаваных мадэляў)…
cmd-run-script = Выканаць скрыпт…
cmd-quit = Выйсці
cmd-undo = Адмяніць
cmd-redo = Вярнуць
cmd-cut = Выразаць
cmd-copy = Капіраваць
cmd-paste = Уставіць
cmd-paste-in-place = Уставіць на месца
cmd-delete = Выдаліць
cmd-purge-unused = Выдаліць нявыкарыстанае
cmd-delete-guides = Выдаліць накіравальныя
cmd-select-all = Вылучыць усё
cmd-select-none = Зняць вылучэнне
cmd-hide = Схаваць
cmd-unhide-all = Паказаць усё
cmd-make-group = Стварыць групу
cmd-make-component = Стварыць кампанент…
cmd-make-unique = Зрабіць унікальным
cmd-make-parametric = Зрабіць параметрычным
cmd-explode = Разбіць
cmd-close-group = Закрыць групу
cmd-intersect-model = З мадэллю
cmd-intersect-selection = З вылучаным
cmd-intersect-context = З кантэкстам
cmd-reverse-faces = Перавярнуць грані
cmd-orient-faces = Арыентаваць грані
cmd-flip-red = Чырвоны напрамак
cmd-flip-green = Зялёны напрамак
cmd-flip-blue = Сіні напрамак
cmd-unfold = Разгортка…
cmd-physics = Фізіка…
cmd-text-3d = 3D-тэкст…
cmd-library-3d = Анлайн-бібліятэка 3D…
cmd-materials-online = Анлайн-бібліятэка матэрыялаў…
cmd-zoom-extents = Па памерах мадэлі
cmd-snapping = Прывязка
cmd-show-grid = Паказваць сетку
cmd-side-bar = Бакавая панэль
cmd-render = Рэндэр…
cmd-work-planes = Працоўныя плоскасці…
cmd-model-info = Звесткі пра дакумент…
cmd-component-params = Параметры кампанента…
cmd-generator-editor = Рэдактар генератараў…
cmd-console = Кансоль скрыптаў
cmd-log = Журнал
cmd-preferences = Налады…

## Descriptions (one line for the toolbar tooltip)

cmd-desc-flip = Адлюстраваць вылучанае па гэтай восі.
cmd-desc-snap = Гэтая прывязка ўкл. / выкл.
cmd-desc-new = Стварыць новы дакумент.
cmd-desc-open = Адкрыць файл Varstotas.
cmd-desc-save = Захаваць дакумент.
cmd-desc-save-as = Захаваць дакумент пад новай назвай.
cmd-desc-preview-show = Малюнак, які бібліятэка «Кампаненты» паказвае для гэтага файла.
cmd-desc-preview-from-file = Выкарыстаць файл відарыса як папярэдні прагляд дакумента.
cmd-desc-preview-from-view = Выкарыстаць бягучы выгляд як папярэдні прагляд дакумента.
cmd-desc-preview-remove = Прыбраць папярэдні прагляд (бібліятэка малюе мадэль).
cmd-desc-run-script = Выканаць файл скрыпта Rhai над мадэллю.
cmd-desc-quit = Закрыць Varstotas.
cmd-desc-cut = Скапіраваць вылучанае і сцерці яго.
cmd-desc-copy = Скапіраваць вылучанае.
cmd-desc-paste = Размясціць скапіраваную геаметрыю мышшу.
cmd-desc-paste-in-place = Уставіць туды, дзе была геаметрыя.
cmd-desc-delete = Сцерці вылучанае.
cmd-desc-purge-unused = Выдаліць матэрыялы і кампаненты, якія мадэль больш не выкарыстоўвае.
cmd-desc-delete-guides = Сцерці ўсе накіравальныя лініі і кропкі.
cmd-desc-select-all = Вылучыць усё ў актыўным кантэксце.
cmd-desc-select-none = Зняць вылучэнне.
cmd-desc-hide = Схаваць вылучанае.
cmd-desc-unhide-all = Паказаць усё схаванае.
cmd-desc-make-group = Змясціць вылучанае ў групу.
cmd-desc-make-component = Ператварыць вылучанае ў кампанент.
cmd-desc-make-unique = Даць вылучанаму кампаненту ўласнае вызначэнне: яго праўка больш не змяняе іншыя копіі.
cmd-desc-make-parametric = Дадаць вылучанай групе параметры (Параметры кампанента).
cmd-desc-explode = Разбіць вылучаныя групы і кампаненты.
cmd-desc-close-group = Выйсці з групы, якая рэдагуецца.
cmd-desc-reverse-faces = Памяняць месцамі твар і адваротны бок вылучаных граней.
cmd-desc-orient-faces = Павярнуць грані побач з вылучанай у той жа бок.
cmd-desc-unfold = Разгарнуць вылучаныя грані ў плоскасць (SVG / PDF / DXF).
cmd-desc-physics = Скінуць вылучаныя групы і кампаненты на мадэль пад дзеяннем цяжару або апусціць грань як тканіну.
cmd-desc-text-3d = Дадаць тэкст як 3D-геаметрыю (групу).
cmd-desc-zoom-extents = Змясціць усю мадэль у выгляд.
cmd-desc-snapping = Прывязка ўкл. / выкл.
cmd-desc-show-grid = Сетка на зямлі ўкл. / выкл.
cmd-desc-side-bar = Бакавая панэль з карткамі (Уласцівасці, Матэрыялы …) паказана або схавана.
cmd-desc-render = Малюнкі мадэлі з трасіроўкай шляхоў (Акно → Рэндэр).
cmd-desc-work-planes = Захаваныя плоскасці малявання.
cmd-desc-model-info = Статыстыка дакумента, Выдаліць нявыкарыстанае.
cmd-desc-component-params = Параметры вылучанага кампанента: значэнні, якія паварочваюць, зрушваюць, паказваюць або фарбуюць яго часткі.
cmd-desc-library-3d = Бясплатныя мадэлі з анлайн-бібліятэк.
cmd-desc-materials-online = Бясплатныя PBR-матэрыялы з анлайн-бібліятэк.
cmd-desc-uv-editor = Раскладка тэкстурных каардынат выбраных граняў.
cmd-desc-generator-editor = Пісаць і запускаць скрыпты генератараў.
cmd-desc-console = Выконваць каманды Rhai над мадэллю.
cmd-desc-log = Памылкі і папярэджанні з усіх месцаў: скрыпты, імпарт / экспарт, бібліятэкі, спампоўкі …
cmd-desc-preferences = Налады Varstotas.
# { $name } is "Undo" or "Redo".
cmd-desc-undo-redo = { $name } апошнюю змену.
cmd-desc-intersect-model = Дадаць рэбры там, дзе вылучаныя грані рэжуць мадэль.
cmd-desc-intersect-selection = Дадаць рэбры там, дзе вылучаныя грані рэжуць адна адну.
cmd-desc-intersect-context = Дадаць рэбры там, дзе вылучаныя грані рэжуць групу, якая рэдагуецца.
cmd-desc-solid-outer-shell = Пакінуць знешнюю абалонку двух вылучаных целаў.
cmd-desc-solid-intersect = Пакінуць агульны аб’ём двух вылучаных целаў.
cmd-desc-solid-union = Аб’яднаць два вылучаныя целы.
cmd-desc-solid-subtract = Выразаць першае з другога з двух вылучаных целаў.
cmd-desc-solid-trim = Абрэзаць другое першым з двух вылучаных целаў.
cmd-desc-solid-split = Раздзяліць два вылучаныя целы на часткі.
# { $card } is the name of the card.
cmd-desc-card = Картка «{ $card }» бакавой панэлі паказана або схавана.
# { $name } is the command's name.
cmd-desc-default = { $name }.

cmd-name-preview-show = Відарыс папярэдняга прагляду: паказаць

cmd-name-preview-from-file = Відарыс папярэдняга прагляду з файла

cmd-name-preview-from-view = Відарыс папярэдняга прагляду з выгляду

cmd-name-preview-remove = Выдаліць відарыс папярэдняга прагляду

cmd-name-intersect-model = Перасячэнне з мадэллю

cmd-name-intersect-selection = Перасячэнне з вылучаным

cmd-name-intersect-context = Перасячэнне з кантэкстам

cmd-name-flip-red = Адлюстраваць па чырвонай

cmd-name-flip-green = Адлюстраваць па зялёнай

cmd-name-flip-blue = Адлюстраваць па сіняй

# { $name } is the format's name, as in the menu.
cmd-name-import = Імпарт { $name }
cmd-name-export = Экспарт { $name }
# { $name } is the solid operation's name.
cmd-name-solid = Цела: { $name }
# { $name } is the inference's name.
cmd-name-snap = Прывязка: { $name }
# { $card } is the name of the card.
cmd-name-card = Бакавая панэль: { $card }

## ======== Інструменты ========

tool-select = Вылучэнне
tool-eraser = Гумка
tool-paint = Заліўка
tool-texture-position = Размяшчэнне тэкстуры
tool-line = Лінія
tool-arc = Дуга па 2 кропках
tool-arc-center = Дуга
tool-arc3 = Дуга па 3 кропках
tool-pie = Сектар
tool-bezier = Крывая Безье
tool-smooth-curve = Гладкая крывая
tool-freehand = Ад рукі
tool-edit-curve = Правіць крывую
tool-rectangle = Прамавугольнік
tool-rotated-rectangle = Павернуты прамавугольнік
tool-circle = Акружнасць
tool-polygon = Шматвугольнік
tool-box = Паралелепіпед
tool-cylinder = Цыліндр
tool-cone = Конус
tool-pyramid = Піраміда
tool-sphere = Сфера
tool-torus = Тор
tool-push-pull = Цягні/Штурхай
tool-move = Перамяшчэнне
tool-rotate = Паварот
tool-scale = Маштабаванне
tool-offset = Зрушэнне контуру
tool-chamfer = Фаска
tool-fillet = Згладжванне
tool-solid-tools = Цвёрдацельныя аперацыі
tool-follow-me = Працягванне
tool-tape = Рулетка
tool-protractor = Транспарцір
tool-axes = Восі
tool-dimension = Памеры
tool-text = Тэкст
tool-section = Секушчая плоскасць
tool-guide-plane = Накіравальная плоскасць
tool-slice = Разрэз
tool-orbit = Вярчэнне
tool-pan = Панарама
tool-zoom = Маштаб
tool-position-camera = Размясціць камеру
tool-look-around = Агледзецца
tool-walk = Абход

tool-select-desc = Вылучайце аб’екты, каб змяняць іх іншымі інструментамі.
tool-eraser-desc = Сціраць рэбры, накіравальныя і іншыя аб’екты.
tool-paint-desc = Ужываць матэрыял да граней і груп.
tool-texture-position-desc = Перамяшчаць, паварочваць і маштабаваць тэкстуру грані шпількамі.
tool-line-desc = Маляваць рэбры. Замкнёныя плоскія контуры становяцца гранямі.
tool-arc-desc = Маляваць дугу па двух канцавых кропках і прагібе.
tool-arc3-desc = Маляваць дугу праз тры кропкі.
tool-arc-center-desc = Маляваць дугу па цэнтры і двух кропках.
tool-pie-desc = Маляваць замкнёную грань-сектар па цэнтры і двух кропках.
tool-bezier-desc = Маляваць крывую, якую фарміруюць ручкі: пстрычка — вугал, націснуць і цягнуць — гладкая кропка.
tool-smooth-curve-desc = Маляваць гладкую крывую праз кропкі, на якія вы пстрыкаеце; замкнёная, яна ўтварае грань.
tool-freehand-desc = Маляваць крывую ад рукі: націснуць і цягнуць; скончаная ў сваім пачатку, яна ўтварае грань.
tool-edit-curve-desc = Змяняць форму намаляванай крывой: цягнуць яе кропкі і ручкі, дадаваць або выдаляць кропкі.
tool-rectangle-desc = Маляваць прамавугольную грань па двух вуглах.
tool-rotated-rectangle-desc = Маляваць прамавугольнік пад любым вуглом па рабры і шырыні.
tool-circle-desc = Маляваць акружнасць па цэнтры і радыусе.
tool-polygon-desc = Маляваць правільны шматвугольнік.
tool-box-desc = Маляваць паралелепіпед: вуглы асновы, потым вышыня. Яго памеры застаюцца рэдагаванымі.
tool-cylinder-desc = Маляваць цыліндр: цэнтр, радыус, вышыня. Яго памеры застаюцца рэдагаванымі.
tool-cone-desc = Маляваць конус: цэнтр, радыус, вышыня. Яго памеры застаюцца рэдагаванымі.
tool-pyramid-desc = Маляваць піраміду: вуглы асновы, потым вышыня. Яе памеры застаюцца рэдагаванымі.
tool-sphere-desc = Маляваць сферу, што ляжыць на грані: цэнтр, радыус. Яе памеры застаюцца рэдагаванымі.
tool-torus-desc = Маляваць тор, што ляжыць на грані: цэнтр, радыус, трубка. Яго памеры застаюцца рэдагаванымі.
tool-push-pull-desc = Штурхаць або цягнуць грань, каб дадаць або прыбраць аб’ём.
tool-move-desc = Перамяшчаць, расцягваць або капіраваць аб’екты.
tool-rotate-desc = Паварочваць, расцягваць або капіраваць аб’екты вакол восі.
tool-scale-desc = Змяняць памер і расцягваць аб’екты.
tool-offset-desc = Зрушваць рэбры грані ўнутр або вонкі.
tool-chamfer-desc = Зрэзаць рабро паміж дзвюма гранямі плоскай фаскай.
tool-fillet-desc = Згладзіць рабро паміж дзвюма гранямі.
tool-solid-tools-desc = Спалучаць цвёрдыя групы: знешняя абалонка, перасячэнне, аб’яднанне, выразанне, абрэзка, раздзяленне.
tool-follow-me-desc = Цягнуць грань уздоўж шляху.
tool-tape-desc = Вымяраць адлегласці і ствараць накіравальныя.
tool-protractor-desc = Вымяраць вуглы і ствараць накіравальныя пад вуглом.
tool-axes-desc = Перамяшчаць або перарыентаваць восі малявання.
tool-dimension-desc = Ставіць лінейныя памеры паміж кропкамі або на рэбрах.
tool-text-desc = Дадаць тэкст з выноснай лініяй, накіраванай на мадэль.
tool-section-desc = Рэзаць мадэль секушчай плоскасцю, каб убачыць унутранасць.
tool-guide-plane-desc = Дадаць працоўную плоскасць для малявання ў паветры; восі і сетка прывязваюцца да яе.
tool-slice-desc = Рэзаць вылучанае або мадэль плоскасцю; пры жаданні раздзяліць на дзве групы.
tool-orbit-desc = Круціць камеру вакол мадэлі.
tool-pan-desc = Зрушваць камеру ўбок.
tool-zoom-desc = Наблізіць або аддаліць камеру.
tool-position-camera-desc = Стаць у кропку на ўзроўні вачэй, каб глядзець на мадэль.
tool-look-around-desc = Павярнуць галаву: круціць выгляд вакол вока.
tool-walk-desc = Хадзіць па мадэлі на ўзроўні вачэй.

solid-union = Аб’яднанне
solid-intersect = Перасячэнне
solid-subtract = Адніманне
solid-trim = Абрэзка
solid-split = Раздзяленне
solid-outer-shell = Знешняя абалонка

## Slice

tool-slice-undo = Разрэз
tool-slice-undo-split = Разрэз з раздзяленнем
tool-slice-nothing = Разрэз: плоскасць нічога не рэжа
tool-slice-status-first = Пстрыкніце кропку на плоскасці разрэзу.
tool-slice-status-second = Пстрыкніце другую кропку. Двайная пстрычка / Enter: плоскасць грані; стрэлкі: нармаль уздоўж чырвонай / зялёнай / сіняй / рабра.
tool-slice-status-third = Пстрыкніце трэцюю кропку, каб рэзаць.
# { $step } is one of the tool-slice-status-* texts; { $split } is "on" or "off".
tool-slice-status = { $step } Рэжа вылучанае або ўсё ў актыўным кантэксце. %COPY%: { $split ->
        [on] раздзяленне ЎКЛ.: часткі становяцца дзвюма групамі
       *[off] раздзяленне ВЫКЛ.: дадаюцца толькі рэбры
    }.

## Measurements box labels (shared)

tool-vcb-measurements = Вымярэнні
tool-vcb-radius = Радыус
tool-vcb-angle = Вугал
tool-vcb-length = Даўжыня
tool-vcb-segments = Сегменты
tool-vcb-distance = Адлегласць
tool-vcb-width-angle = Шырыня, вугал
tool-vcb-dimensions = Памеры
tool-vcb-height = Вышыня
tool-vcb-sides = Бакі
tool-vcb-bulge = Прагіб
tool-vcb-origin = Пачатак
tool-vcb-point = Кропка
tool-vcb-eye-height = Вышыня вачэй
tool-vcb-height-offset = Зрушэнне па вышыні
tool-vcb-fov = Вугал агляду

## Arcs, rectangles

tool-undo-pie = Сектар
tool-undo-arc = Дуга
tool-undo-arc3 = Дуга па 3 кропках
tool-undo-rotated-rectangle = Павернуты прамавугольнік
# { $label } is the shape's name (an undo name).
tool-degenerate = { $label }: выраджаныя ўваходныя даныя
tool-cursor-angle = Вугал
tool-arc-center-first = Выберыце цэнтр (грань задае плоскасць; стрэлкі фіксуюць плоскасць). Увядзіце `{ $segs }s` для колькасці сегментаў.
tool-arc-center-start = Выберыце пачатковую кропку (радыус) або ўвядзіце радыус.
tool-arc-center-end = Рухайцеся вакол цэнтра і пстрыкніце, каб скончыць, або ўвядзіце вугал.
tool-arc-what-pie = сектар
tool-arc-what-arc = дуга
tool-arc3-first = Выберыце пачатковую кропку дугі.
tool-arc3-second = Выберыце другую кропку, праз якую праходзіць дуга, або ўвядзіце даўжыню.
tool-arc3-third = Выберыце канцавую кропку або ўвядзіце вугал дугі.
tool-rotrect-first = Выберыце першы вугал прамавугольніка.
tool-rotrect-second = Выберыце другі вугал (першае рабро) або ўвядзіце даўжыню (`l,w`).
tool-rotrect-third = Рухайце, каб задаць шырыню і вугал, і пстрыкніце, або ўвядзіце `шырыня` ці `шырыня,вугал`.

## Bezier / curves

tool-bezier-end = Enter або двайная пстрычка завяршае, пстрычка на першай кропцы замыкае ў грань
tool-bezier-freehand-start = Націсніце і цягніце, каб маляваць; скончыце ў пачатку, каб замкнуць у грань.
tool-bezier-freehand-draw = Цягніце, каб маляваць; адпусціце, каб скончыць.
tool-bezier-smooth-first = Пстрыкніце першую кропку крывой.
# { $end } is the tool-bezier-end text.
tool-bezier-smooth-next = Пстрыкніце наступную кропку; { $end }. Backspace выдаляе апошнюю кропку.
tool-bezier-handles-first = Пстрыкніце першую кропку крывой; націсніце і цягніце, каб выцягнуць ручкі.
tool-bezier-handles-drag = Цягніце, каб сфарміраваць крывую праз гэтую кропку; адпусціце, каб размясціць яе.
# { $end } is the tool-bezier-end text.
tool-bezier-handles-next = Пстрыкніце наступную кропку (цягніце для ручак); { $end }. Backspace выдаляе апошнюю кропку.

## Camera tools

tool-position-camera-status = Пстрыкніце, каб стаць у кропку, або цягніце з месца, дзе стаяць, туды, куды глядзець. Увядзіце вышыню вачэй.
tool-look-around-status = Цягніце, каб аглядацца. Увядзіце вышыню вачэй і націсніце Enter.
tool-walk-status = Трымайце кнопку і рухайце ўверх/уніз, каб ісці, улева/управа, каб паварочваць (далей — хутчэй). Shift = уверх/уніз і ўбок, %CMD% = бег. Стрэлкі — крокі.

## Fillet / Chamfer

tool-undo-fillet = Згладжванне
tool-undo-chamfer = Фаска
# { $what } is the lower-case operation name, { $error } the reason.
tool-chamfer-cant = { $what }: не ўдалося ўжыць да гэтага рабра: { $error }
tool-chamfer-what-fillet = Згладжванне
tool-chamfer-what-chamfer = Фаска
tool-fillet-sizing = Адсуньцеся ад рабра, каб задаць радыус, пстрыкніце, каб скончыць. Увядзіце радыус або `12s`; Esc скасоўвае.
tool-chamfer-sizing = Адсуньцеся ад рабра, каб задаць памер, пстрыкніце, каб скончыць. Увядзіце адлегласць; Esc скасоўвае.
tool-fillet-hover = Пстрыкніце рабро паміж дзвюма гранямі, каб згладзіць яго (вылучаныя рэбры: усе адразу); двайная пстрычка = апошні радыус.
tool-chamfer-hover = Пстрыкніце рабро паміж дзвюма гранямі, каб зрабіць фаску (вылучаныя рэбры: усе адразу); двайная пстрычка = апошні памер.

## Tape measure, protractor, axes

tool-undo-resize-model = Змяніць памер мадэлі
tool-undo-resize = Змяніць памер
tool-undo-create-guide = Стварыць накіравальную
tool-undo-place-axes = Размясціць восі
tool-tape-guides-on = накіравальныя ўключаны
tool-tape-measure-only = толькі вымярэнне
# { $g } is tool-tape-guides-on or tool-tape-measure-only.
tool-tape-status-resize = Увядзіце новую даўжыню, каб змяніць памер мадэлі, або выберыце пачатковую кропку ({ $g }).
tool-tape-status-start = Выберыце пачатковую кропку (рабро — для паралельнай накіравальнай). %COPY% пераключае накіравальныя ({ $g }).
tool-tape-status-end = Выберыце канцавую кропку або ўвядзіце адлегласць ({ $g }).
tool-protractor-center = Выберыце цэнтр транспарціра (стрэлкі фіксуюць яго вось).
tool-protractor-base = Выберыце базавы напрамак.
tool-protractor-angle = Выберыце вугал (прывязка праз кожныя 15°) або ўвядзіце градусы.
tool-axes-origin = Выберыце новы пачатак або ўвядзіце x,y,z.
tool-axes-red = Выберыце кропку на новай чырвонай восі.
tool-axes-green = Выберыце кропку ў напрамку новай зялёнай восі.

## Curve editing, dimensions

tool-undo-add-curve-point = Дадаць кропку крывой
tool-undo-edit-curve = Правіць крывую
tool-undo-remove-curve-point = Выдаліць кропку крывой
tool-undo-edit-segments = Змяніць сегменты
tool-curve-edit-too-few = Правіць крывую: замала кропак, каб выдаліць якую-небудзь
tool-curve-edit-pick = Пстрыкніце крывую, намаляваную інструментам «Крывая Безье», «Гладкая крывая» або «Ад рукі».
tool-curve-edit-drag = Цягніце, каб змяніць форму; адпусціце, каб ужыць, Esc скасоўвае.
tool-curve-edit-insert = Пстрыкніце, каб дадаць тут кропку (крывая захоўвае форму).
tool-curve-edit-idle = Цягніце кропкі і ручкі; пстрыкніце крывую, каб дадаць кропку, Delete выдаляе вылучаную кропку; двайная пстрычка па кропцы: вугал / гладкая. Enter завяршае.
tool-undo-dimension = Памер
tool-dimension-first = Выберыце пачатковую кропку памеру або пстрыкніце рабро, каб прастаўляць яго даўжыню.
tool-dimension-second = Выберыце канцавую кропку памеру.
tool-dimension-third = Рухайце, каб зрушыць памерную лінію, і пстрыкніце, каб размясціць яе.

## Line, rectangle, circle, polygon, arc

tool-undo-line = Лінія
tool-undo-rectangle = Прамавугольнік
tool-undo-polygon = Шматвугольнік
tool-undo-circle = Акружнасць
tool-line-first = Выберыце пачатковую кропку. Стрэлкі фіксуюць вось.
tool-line-next = Выберыце канцавую кропку або ўвядзіце даўжыню. Shift = зафіксаваць падказку, стрэлкі = зафіксаваць вось, Esc/двайная пстрычка = скончыць.
tool-rectangle-first = Выберыце першы вугал. Стрэлкі выбіраюць плоскасць (→ чырвоная, ← зялёная, ↑ сіняя нармаль).
tool-rectangle-second = Выберыце процілеглы вугал або ўвядзіце шырыня,вышыня. Esc = скасаваць.
tool-circle-what-polygon = шматвугольніка
tool-circle-what-circle = акружнасці
tool-circle-for-sides = бакоў
tool-circle-for-segments = сегментаў
# { $what } is tool-circle-what-*, { $kind } is tool-circle-for-*.
tool-circle-first = Выберыце цэнтр { $what } (грань пад курсорам арыентуе яго; стрэлкі фіксуюць нармаль). Увядзіце `{ $segs }s` для колькасці { $kind }.
tool-circle-radius = Выберыце кропку на радыусе або ўвядзіце радыус.
tool-arc-first = Выберыце пачатковую кропку дугі. Пачніце ў канцы рабра, каб маляваць датычна да яго.
tool-arc-end-tangent = Выберыце канцавую кропку (двайная пстрычка па ёй — для блакітнай датычнай дугі) або ўвядзіце даўжыню хорды.
tool-arc-end = Выберыце канцавую кропку або ўвядзіце даўжыню хорды.
tool-arc-bulge = Рухайце, каб задаць прагіб, і пстрыкніце, або ўвядзіце прагіб (`500r` для радыуса). Увядзіце `12s` для сегментаў.

## Eraser, Paint Bucket, Push/Pull, Offset, Sweep

tool-undo-hide = Схаваць
tool-undo-unsoften = Зрабіць рэзкім
tool-undo-soften = Згладзіць
tool-undo-erase = Сцерці
tool-eraser-status = Пстрыкайце або праводзьце па рэбрах, накіравальных, памерах ці групах, каб сцерці. Shift = схаваць, %CMD% = згладзіць, %CMD%+Shift = зрабіць рэзкім.
tool-paint-mods = Alt = ўзяць узор, Ctrl = звязаныя грані з тым жа матэрыялам, Shift = усе падобныя грані, Ctrl+Shift = падобныя звязаныя грані
    .macos = Cmd = ўзяць узор, Option = звязаныя грані з тым жа матэрыялам, Shift = усе падобныя грані, Option+Shift = падобныя звязаныя грані
tool-undo-paint = Заліўка
# { $mods } is tool-paint-mods.
tool-paint-status = Пстрыкніце грань або групу, каб зафарбаваць. { $mods }.
tool-paint-default-material = Прадвызначаны
tool-undo-push-pull = Цягні/Штурхай
# { $keep } is "on" or "off".
tool-push-pull-moving = Рухайце, каб задаць адлегласць, пстрыкніце або ўвядзіце значэнне. %COPY% = пакінуць зыходную грань ({ $keep ->
        [on] укл.
       *[off] выкл.
    }).
tool-push-pull-idle = Пстрыкніце грань, каб штурхаць або цягнуць яе. Двайная пстрычка паўтарае апошнюю адлегласць.
tool-undo-offset = Зрушэнне контуру
tool-offset-moving = Рухайце ўнутр або вонкі і пстрыкніце, або ўвядзіце адлегласць.
tool-offset-idle = Пстрыкніце грань, каб зрушыць яе рэбры. Двайная пстрычка паўтарае апошняе зрушэнне.
tool-undo-follow-me = Працягванне
tool-follow-me-profile = Пстрыкніце грань-профіль (загадзя вылучыце рэбры шляху, каб цягнуць адразу).
tool-follow-me-path = Пстрыкайце або праводзьце ўздоўж рэбраў шляху; двайная пстрычка/Enter — скончыць. Пстрыкніце грань, каб цягнуць вакол яе рабра. Esc = скасаваць.

## Select

tool-select-mods = Ctrl = дадаць, Shift = пераключыць, Ctrl+Shift = прыбраць
    .macos = Option (або Cmd) = дадаць, Shift = пераключыць, Shift+Option = прыбраць
tool-undo-aim-light = Навесці святло
tool-undo-active-cut = Актыўны зрэз
tool-select-aim = Навядзіце святло: пакіньце ручку на тым, на што яно павінна свяціць. Esc = скасаваць.
# { $mods } is tool-select-mods.
tool-select-box = Цягніце злева направа для вылучэння рамкай, справа налева — для вылучэння секушчай рамкай. { $mods }.
tool-select-idle = Вылучайце аб’екты. { $mods }. Цягніце, каб вылучыць некалькі. Двайная пстрычка па групе — рэдагаваць яе.

## Move, Rotate, Scale, Paste

tool-undo-copy = Капіраваць
tool-undo-move = Перамясціць
tool-undo-copy-array = Капіраваць масіў
tool-undo-rotate = Павярнуць
tool-undo-rotate-array = Павярнуць масіў
tool-undo-scale = Маштабаваць
tool-undo-paste = Уставіць
tool-suffix-copy = (копія)
tool-suffix-on = (укл.)
# { $copy } is empty or " " + tool-suffix-copy.
tool-move-from = Выберыце кропку, адкуль перамяшчаць{ $copy }. Перамяшчае вылучанае або аб’ект пад курсорам. %COPY% = копія; пасля копіі ўвядзіце `5x` або `/5` для масіваў.
tool-move-to = Выберыце пункт прызначэння або ўвядзіце адлегласць{ $copy }. Shift = зафіксаваць падказку, стрэлкі = зафіксаваць вось.
tool-rotate-center = Выберыце цэнтр павароту (грань арыентуе транспарцір; стрэлкі фіксуюць яго вось).
tool-rotate-zero = Выберыце нулявы напрамак.
# { $step } is tool-rotate-rounded or tool-rotate-snaps; { $on } is empty or " " + tool-suffix-on.
tool-rotate-angle = Выберыце вугал ({ $step }) або ўвядзіце градусы. %COPY% = копія{ $on }; пасля копіі ўвядзіце `6x` для радыяльнага масіва.
# { $step } is an angle.
tool-rotate-rounded = акругляецца да { $step }°
tool-rotate-snaps = прывязка праз кожныя 15°
tool-scale-uniformly = раўнамерна
tool-scale-two-axes = па дзвюх восях
tool-scale-one-axis = па адной восі
tool-scale-about-centre = ад цэнтра
tool-scale-about-opposite = ад процілеглай ручкі
# { $what } is tool-scale-uniformly / -two-axes / -one-axis, { $about } is tool-scale-about-*.
tool-scale-dragging = Рухайце, каб маштабаваць { $what } { $about }; пстрыкніце або ўвядзіце каэфіцыент (па восях: `1.5; 2`). %CMD% = ад цэнтра.
tool-scale-select = Вылучыце аб’екты (або навядзіце на адзін), потым цягніце ручку.
tool-scale-idle = Цягніце ручку: вуглавыя маштабуюць раўнамерна, сярэдзіны рэбраў — па дзвюх восях, цэнтры граней — па адной. %CMD% = ад цэнтра.
# { $factors } is the scale factor(s).
tool-scale-tooltip-uniform = Раўнамерны маштаб { $factors }
tool-scale-tooltip = Маштаб { $factors }
# { $name } is the name of the component being placed.
tool-paste-place-named = Пстрыкніце, каб размясціць «{ $name }» (Esc скасоўвае).
# { $count } is the number of entities.
tool-paste-place = Пстрыкніце, каб размясціць устаўляемыя аб’екты: { $count ->
        [one] { $count } аб’ект
        [few] { $count } аб’екты
        [many] { $count } аб’ектаў
       *[other] { $count } аб’екта
    } (Esc скасоўвае).

## Section, solids, text, texture, workplane, misc

tool-undo-section-plane = Секушчая плоскасць
tool-section-status = Пстрыкніце грань, каб размясціць секушчую плоскасць. Яна адсякае бок за стрэлкамі.

## Shapes (box, cylinder, cone, pyramid, sphere, torus)

tool-vcb-tube = Трубка
# { $what } is the lower-case shape name.
tool-prim-box-first = Фігура «{ $what }»: пстрыкніце першы вугал асновы (грань пад курсорам задае арыентацыю; стрэлкі задаюць плоскасць).
tool-prim-round-first = Фігура «{ $what }»: пстрыкніце цэнтр асновы (грань пад курсорам задае арыентацыю). Увядзіце `{ $segs }s` для сегментаў.
tool-prim-box-second = Пстрыкніце процілеглы вугал або ўвядзіце `шырыня, глыбіня` (`шырыня, глыбіня, вышыня` размяшчае адразу).
tool-prim-sphere-radius = Рухайце, каб задаць радыус, і пстрыкніце, або ўвядзіце яго.
tool-prim-torus-radius = Пстрыкніце радыус (да сярэдзіны трубкі) або ўвядзіце `радыус, трубка`.
tool-prim-round-radius = Пстрыкніце радыус або ўвядзіце `радыус` (`радыус, вышыня` размяшчае адразу).
tool-prim-torus-tube = Рухайце, каб задаць радыус трубкі, і пстрыкніце, або ўвядзіце яго.
tool-prim-height = Рухайце ўверх або ўніз, каб задаць вышыню, і пстрыкніце, або ўвядзіце яе.

# { $name } is the solid operation's name.
tool-solid-not-solid = { $name }: не цела (кожнае рабро павінна абмяжоўваць роўна дзве грані, укладзеных груп няма).
tool-solid-first = { $name }: пстрыкніце першую цвёрдую групу або кампанент.
tool-solid-next = { $name }: пстрыкніце наступнае цела, каб дадаць яго (Esc — пачаць нанова).
tool-solid-cut = { $name }: пстрыкніце цела, якое трэба выразаць першым (Esc — пачаць нанова).
tool-solid-second = { $name }: пстрыкніце другое цела (Esc — пачаць нанова).
tool-undo-text = Тэкст
tool-text-first = Пстрыкніце кропку, рабро або грань, каб прымацаваць тэкст, або пустое месца для тэксту без выноснай лініі.
tool-text-place = Рухайце, каб размясціць тэкст, і пстрыкніце. Рэдагуйце тэкст у карцы «Уласцівасці».
tool-text3d-status = Пстрыкніце, каб размясціць 3D-тэкст (Esc скасоўвае).
tool-undo-texture-position = Размяшчэнне тэкстуры
tool-texture-pick = Пстрыкніце грань з тэкстурай, каб размясціць яе тэкстуру.
tool-texture-move = Перамяшчэнне тэкстуры. Esc скасоўвае.
tool-texture-green = Паварот і маштабаванне тэкстуры вакол чырвонай шпількі.
tool-texture-blue = Маштабаванне вышыні тэкстуры.
tool-texture-idle = Цягніце грань або чырвоную шпільку для перамяшчэння, зялёную — для павароту/маштабавання, сінюю — для расцягвання. Enter або пстрычка па-за ёй, калі гатова.
tool-texture-idle-many = Граняў: { $count }, тэкстура працягваецца ад абранай. Shift+пстрычка дадае або прыбірае грань, Alt+пстрычка працягвае тэкстуру на іншую.
tool-texture-cant-continue = Тэкстуру нельга працягнуць на гэту грань (яна перпендыкулярная).
tool-texture-vcb = Плітка

tool-undo-add-guide-plane = Дадаць накіравальную плоскасць
tool-undo-edit-guide-plane = Рэдагаваць накіравальную плоскасць
tool-workplane-origin = Пстрыкніце пачатак плоскасці (або ўвядзіце x,y,z). Цягніце ручкі актыўнай плоскасці: пачатак, вастрыё нармалі, ручка павароту вакол чырвонай.
tool-workplane-red = Пстрыкніце кропку на новай чырвонай восі. Двайная пстрычка / Enter: плоскасць грані; стрэлкі: нармаль уздоўж чырвонай / зялёнай / сіняй / рабра.
tool-workplane-third = Пстрыкніце трэцюю кропку на плоскасці.
tool-workplane-drag-origin = Перамясціць пачатак плоскасці.
tool-workplane-drag-normal = Нахіліць плоскасць: перамясціць яе нармаль.
tool-workplane-drag-spin = Круціць плоскасць вакол яе нармалі.
tool-zoom-status = Цягніце ўверх/уніз для маштабавання. Увядзіце вугал агляду і націсніце Enter.
tool-orbit-status = Цягніце для вярчэння. Shift = панарама.
tool-pan-status = Цягніце для панарамы.
# { $prompt } is the plugin's prompt.
tool-plugin-select = { $prompt } — пстрыкніце, каб дадаць / прыбраць, Enter — працягнуць, Esc скасоўвае.
# { $name } is the tool's name.
tool-not-implemented = Інструмент «{ $name }» яшчэ не рэалізаваны.

## Tool Guide panel (long help per tool; a line starting with • is a list item)

tool-select-instructor =
    Пстрыкніце аб’ект, каб вылучыць яго.
    • Двайная пстрычка па грані: вылучыць яе з рэбрамі.
    • Трайная пстрычка: вылучыць усё звязанае.
    • Цягніце злева направа: вылучэнне рамкай (цалкам унутры).
    • Цягніце справа налева: вылучэнне секушчай рамкай (датыкаецца).
    • Ctrl = дадаць, Shift = пераключыць, Ctrl+Shift = прыбраць.
    • Двайная пстрычка па групе — рэдагаваць яе; Esc — закрыць.
    • Delete сцірае вылучанае.
    .macos =
        Пстрыкніце аб’ект, каб вылучыць яго.
        • Двайная пстрычка па грані: вылучыць яе з рэбрамі.
        • Трайная пстрычка: вылучыць усё звязанае.
        • Цягніце злева направа: вылучэнне рамкай (цалкам унутры).
        • Цягніце справа налева: вылучэнне секушчай рамкай (датыкаецца).
        • Option (або Cmd) = дадаць, Shift = пераключыць, Shift+Option = прыбраць.
        • Двайная пстрычка па групе — рэдагаваць яе; Esc — закрыць.
        • Delete сцірае вылучанае.
tool-eraser-instructor = Пстрыкайце або праводзьце па рэбрах, каб сцерці іх. Сціранне рабра прыбірае грані, якія яно абмяжоўвае.
tool-paint-instructor =
    Выберыце матэрыял на панэлі «Матэрыялы», потым пстрыкайце грані.
    • Alt: узяць узор матэрыялу.
    • Ctrl: заліць звязаныя грані з тым жа матэрыялам.
    • Shift: замяніць матэрыял на ўсіх падобных гранях.
    • Ctrl+Shift: замяніць яго на падобных звязаных гранях.
    .macos =
        Выберыце матэрыял на панэлі «Матэрыялы», потым пстрыкайце грані.
        • Cmd: узяць узор матэрыялу.
        • Option: заліць звязаныя грані з тым жа матэрыялам.
        • Shift: замяніць матэрыял на ўсіх падобных гранях.
        • Option+Shift: замяніць яго на падобных звязаных гранях.
tool-texture-position-instructor =
    Размясціце тэкстуру грані з тэкстурай (вылучыце яе або пстрыкніце па ёй).
    • Цягніце грань або чырвоную шпільку: перамясціць тэкстуру.
    • Зялёная шпілька: павярнуць і змяніць маштаб вакол чырвонай шпількі.
    • Сіняя шпілька: расцягнуць вышыню тэкстуры.
    • Шпількі прыліпаюць да вяршынь і рэбраў; падвойная пстрычка па рэбры выраўноўвае тэкстуру па ім.
    • Shift+пстрычка дадае грань, Alt+пстрычка працягвае на яе тэкстуру (праз вуглы).
    • Увядзіце памер пліткі на грані (`800` або `800;400`) або вугал (`30°`).
    • Esc скасоўвае перацягванне; Enter або пстрычка па-за гранню завяршае.
tool-line-instructor =
    Пстрыкніце, каб пачаць, пстрыкніце, каб скончыць. Кожны канец пачынае новы сегмент.
    Увядзіце даўжыню і націсніце Enter для дакладнасці.
    Shift фіксуе напрамак падказкі; стрэлкі фіксуюць восі.
tool-arc-instructor =
    Пстрыкніце пачатковую кропку, пстрыкніце канцавую, потым рухайце, каб задаць прагіб, і пстрыкніце.
    • Увядзіце даўжыню хорды, потым прагіб (або `500r` для радыуса); `12s` задае сегменты.
    • Калі пачаць у канцы рабра, блакітная дуга будзе датычнай да яго.
    • Двайная пстрычка малюе дугу, датычную да папярэдняй.
tool-arc-center-instructor =
    Пстрыкніце цэнтр, пстрыкніце пачатковую кропку (радыус), потым рухайцеся вакол і пстрыкніце канец.
    Пасля першай пстрычкі ўвядзіце радыус, потым вугал. `12s` задае сегменты.
tool-arc3-instructor =
    Пстрыкніце пачатковую кропку, пстрыкніце кропку, праз якую праходзіць дуга, потым пстрыкніце канец.
    Пасля першай пстрычкі ўвядзіце даўжыню, пасля другой — вугал.
tool-bezier-instructor =
    Пстрыкайце кропкі крывой; націсніце і цягніце на кропцы, каб выцягнуць яе ручкі (проста пстрычка стварае вугал).
    Enter або двайная пстрычка завяршае адкрытую крывую (толькі рэбры); пстрычка на першай кропцы замыкае яе ў грань.
    Backspace выдаляе апошнюю кропку. Увядзіце даўжыню для наступнай кропкі ў бок курсора; `12s` задае сегменты на ўчастак.
tool-edit-curve-instructor =
    Змяняйце форму крывой, намаляванай інструментам «Крывая Безье», «Гладкая крывая» або «Ад рукі» (двайная пстрычка па ёй інструментам «Вылучэнне» або вылучыце яе і абярыце «Правіць крывую»).
    • Цягніце кропку (квадрат), каб перамясціць яе, ручку (кружок), каб выгнуць крывую; гладкая кропка трымае свае ручкі на адной лініі.
    • Пстрыкніце крывую паміж кропкамі, каб дадаць кропку; Delete выдаляе вылучаную кропку.
    • Двайная пстрычка па кропцы: вугал ↔ гладкая. `12s` задае сегменты на ўчастак.
    • Enter, Esc або пстрычка ў пустым месцы завяршае.
tool-smooth-curve-instructor =
    Пстрыкайце кропкі; крывая гладка праходзіць праз іх.
    Enter або двайная пстрычка завяршае адкрытую крывую (толькі рэбры); пстрычка на першай кропцы замыкае яе ў грань.
    Backspace выдаляе апошнюю кропку. Увядзіце даўжыню для наступнай кропкі ў бок курсора; `12s` задае сегменты на ўчастак.
tool-freehand-instructor =
    Націсніце і цягніце, каб маляваць; адпусціце, каб скончыць. Штрых згладжваецца.
    Штрых, што канчаецца ў сваім пачатку, замыкаецца ў грань; інакш застаюцца рэбры.
tool-pie-instructor =
    Пстрыкніце цэнтр, пстрыкніце пачатковую кропку (радыус), потым пстрыкніце канец, каб атрымаць замкнёную грань-сектар.
    Увядзіце радыус, потым вугал.
tool-rectangle-instructor = Пстрыкніце першы вугал, потым процілеглы. Увядзіце шырыня,вышыня для дакладнасці.
tool-rotated-rectangle-instructor =
    Пстрыкніце першы вугал, пстрыкніце другі (першае рабро), потым рухайце, каб задаць шырыню, і пстрыкніце.
    Увядзіце даўжыню (або `l,w`) рабра, потым шырыню (або `w,вугал`).
tool-circle-instructor = Пстрыкніце цэнтр, потым рухайце і пстрыкніце, каб задаць радыус. Увядзіце `24s` перад пстрычкай, каб задаць сегменты.
tool-polygon-instructor = Пстрыкніце цэнтр, потым радыус. Увядзіце `6s`, каб задаць колькасць бакоў.
tool-box-instructor =
    Пстрыкніце першы вугал асновы, пстрыкніце процілеглы вугал, потым рухайце ўверх або ўніз і пстрыкніце, каб задаць вышыню.
    • Грань пад курсорам нясе аснову; стрэлкі пасля першай пстрычкі задаюць яе плоскасць.
    • Увядзіце `шырыня, глыбіня`, потым вышыню — або `шырыня, глыбіня, вышыня` адразу.
    • Вынік — кампанент: змяняйце яго памеры ў карцы «Уласцівасці»; «Разбіць» дае звычайную геаметрыю.
tool-cylinder-instructor =
    Пстрыкніце цэнтр асновы, пстрыкніце радыус, потым рухайце ўверх або ўніз і пстрыкніце, каб задаць вышыню.
    • Увядзіце радыус, потым вышыню — або `радыус, вышыня` адразу; `32s` задае сегменты.
    • Вынік — кампанент: змяняйце яго памеры ў карцы «Уласцівасці»; «Разбіць» дае звычайную геаметрыю.
tool-sphere-instructor =
    Пстрыкніце там, дзе сфера ляжыць, потым рухайце і пстрыкніце, каб задаць радыус (або ўвядзіце яго).
    • `32s` задае сегменты. Вынік — кампанент: змяняйце яго памеры ў карцы «Уласцівасці».
tool-torus-instructor =
    Пстрыкніце цэнтр, пстрыкніце радыус (да сярэдзіны трубкі), потым рухайце і пстрыкніце, каб задаць трубку.
    • Увядзіце `радыус, трубка` адразу; `32s` задае сегменты.
    • Вынік — кампанент: змяняйце яго памеры ў карцы «Уласцівасці».
tool-push-pull-instructor =
    Навядзіце на грань, пстрыкніце, рухайце, каб задаць адлегласць, пстрыкніце яшчэ раз.
    %COPY% пакідае зыходную грань. Двайная пстрычка паўтарае апошнюю адлегласць.
tool-move-instructor = Пстрыкніце кропку, каб узяць вылучанае, пстрыкніце пункт прызначэння. %COPY% робіць копію; увядзіце `5x` для масіва.
tool-rotate-instructor = Пстрыкніце цэнтр, пстрыкніце нулявы напрамак, пстрыкніце вугал. Увядзіце вугал у градусах.
tool-scale-instructor = Вылучыце аб’екты, потым цягніце ручку. Увядзіце каэфіцыент маштабу.
tool-offset-instructor = Пстрыкніце грань, рухайце ўнутр або вонкі, пстрыкніце. Увядзіце адлегласць.
tool-chamfer-instructor =
    Пстрыкніце рабро паміж дзвюма гранямі (або адно з вылучаных рэбраў: усе адразу), адсуньцеся ад яго, каб задаць памер, пстрыкніце яшчэ раз.
    • Увядзіце адлегласць падчас задання памеру; двайная пстрычка бярэ апошні памер.
    • Увод адразу пасля фаскі пераробляе яе з новым памерам. Esc скасоўвае.
tool-fillet-instructor =
    Пстрыкніце рабро паміж дзвюма гранямі (або адно з вылучаных рэбраў: усе адразу), адсуньцеся ад яго, каб задаць радыус, пстрыкніце яшчэ раз.
    • Увядзіце радыус, `12s` для колькасці сегментаў або абодва: `20,12s`.
    • Двайная пстрычка бярэ апошні радыус; увод адразу пасля згладжвання пераробляе яго. Esc скасоўвае.
tool-solid-tools-instructor =
    Інструменты → Цвёрдацельныя аперацыі: выберыце аперацыю, потым пстрыкніце першую цвёрдую групу, затым другую.
    • Калі цвёрдыя целы вылучаны, аперацыя выконваецца адразу.
    • Адніманне / Абрэзка рэжуць другое цела першым (Абрэзка пакідае першае).
    • Раздзяленне дае тры часткі; Знешняя абалонка / Аб’яднанне працягваюць дадаваць целы, па якіх пстрыкаюць.
    • Цвёрдае цела: кожнае рабро абмяжоўвае роўна дзве грані, укладзеных груп няма.
tool-follow-me-instructor = Вылучыце шлях, потым пстрыкніце грань-профіль, каб працягнуць яе.
tool-tape-instructor = Пстрыкніце дзве кропкі, каб вымераць. Пачатак на рабры стварае паралельную накіравальную.
tool-protractor-instructor = Пстрыкніце цэнтр, пстрыкніце базавы напрамак, пстрыкніце вугал, каб стварыць накіравальную.
tool-axes-instructor =
    Пстрыкніце новы пачатак, пстрыкніце кропку на чырвонай восі, потым кропку ў напрамку зялёнай восі.
    Увядзіце `x,y,z` для пачатку. Пстрыкніце восі правай кнопкай: Размясціць / Перамясціць / Скінуць.
tool-dimension-instructor =
    Пстрыкніце пачатковую і канцавую кропкі (або пстрыкніце рабро), потым рухайце, каб зрушыць памер, і пстрыкніце.
    Двайная пстрычка па тэксце памеру інструментам «Вылучэнне» рэдагуе яго; пусты тэкст вяртае вымераную даўжыню.
tool-text-instructor =
    Пстрыкніце кропку, рабро або грань, адсуньце тэкст і пстрыкніце яшчэ раз.
    • Тэкст па змаўчанні: каардынаты кропкі, даўжыня рабра, плошча грані.
    • Пстрыкніце пустое месца для тэксту без выноснай лініі.
    • Рэдагуйце тэкст у карцы «Уласцівасці»; перамяшчайце яго інструментам «Перамяшчэнне» або разам з геаметрыяй.
tool-section-instructor =
    Пстрыкніце грань, каб размясціць на ёй секушчую плоскасць; яна становіцца актыўным зрэзам.
    • Стрэлкі паказваюць на частку, што застаецца бачнай.
    • Вылучайце плоскасць за яе мяжу; перамяшчайце яе інструментам «Перамяшчэнне», выдаляйце клавішай Delete.
    • Правы клік па плоскасці: Актыўны зрэз, Разгарнуць, Выраўнаваць выгляд.
    • Выгляд → Секушчыя плоскасці / Зрэзы / Запаўненне зрэзаў.
tool-guide-plane-instructor =
    Пстрыкніце тры кропкі: пачатак, кропку на чырвонай восі, кропку на плоскасці.
    • Двайная пстрычка па грані (або Enter пасля першай пстрычкі): плоскасць гэтай грані.
    • Пасля першай пстрычкі стрэлкі задаюць нармаль: → чырвоная, ← зялёная, ↑ сіняя, ↓ апошняе рабро.
    • Новая плоскасць актыўная: кропкі ў паветры трапляюць на яе, восі і сетка прывязваюцца да яе.
    • Цягніце ручкі актыўнай плоскасці: пачатак, вастрыё нармалі (нахіл), чырвоная ручка (паварот).
    • Акно → Працоўныя плоскасці, каб актываваць, схаваць, перайменаваць, выдаліць або ўвесці вуглы.
tool-slice-instructor =
    Задайце плоскасць разрэзу, як накіравальную плоскасць: пстрыкніце тры кропкі.
    • Двайная пстрычка па грані (або Enter пасля першай пстрычкі): плоскасць гэтай грані.
    • Пасля першай пстрычкі стрэлкі задаюць нармаль: → чырвоная, ← зялёная, ↑ сіняя, ↓ апошняе рабро.
    • Рэжа вылучанае (грані, групы, кампаненты) або ўсё ў актыўным кантэксце.
    • Рэбры дадаюцца ўздоўж разрэзу; кампанент, агульны з іншымі копіямі, спачатку робіцца ўнікальным.
    • %COPY% пераключае раздзяленне (+ каля курсора): кожнае разрэзанае цела становіцца дзвюма групамі, закрытымі ў месцы разрэзу.
tool-orbit-instructor =
    Цягніце, каб круціць вакол кропкі пад курсорам.
    Сярэдняя кнопка мышы круціць у любым інструменце; Shift+сярэдняя робіць панараму.
    Трэкпад: пракрутка двума пальцамі круціць, Shift+пракрутка — панарама, шчыпок маштабуе.
tool-pan-instructor = Цягніце, каб зрушыць выгляд. Shift + сярэдняя кнопка мышы робіць панараму ў любым інструменце.
tool-zoom-instructor =
    Цягніце ўверх, каб наблізіць, уніз, каб аддаліць. Кола мышы маштабуе ў бок курсора ў любым інструменце.
    Увядзіце вугал агляду (напрыклад, 35) і націсніце Enter. Shift+Z паказвае ўсю мадэль.
tool-position-camera-instructor =
    Пстрыкніце, каб стаць у кропку, гледзячы туды, куды зараз глядзіць камера.
    • Цягніце з месца, дзе стаяць, туды, куды глядзець.
    • Увядзіце вышыню вачэй (па змаўчанні 1676 мм) перад пстрычкай.
    • Пасля гэтага пераключаецца на «Агледзецца».
tool-look-around-instructor = Цягніце, каб круціць выгляд вакол вока. Увядзіце вышыню вачэй і націсніце Enter.
tool-walk-instructor =
    Цягніце ўверх/уніз, каб ісці наперад/назад, улева/управа, каб паварочваць.
    • Shift: рух уверх/уніз і ўбок. %CMD%: бег.
    • Стрэлкі: крокі і павароты. Увядзіце вышыню вачэй і націсніце Enter.

# { $prompt } is the plugin's prompt, { $index } of { $total } the point being picked.
tool-plugin-pick = { $prompt } ({ $index } з { $total }; Esc скасоўвае).

## ======== Прывязка і падказкі ========

snap-endpoint = Канцавыя кропкі
snap-midpoint = Сярэдзіны
snap-center = Цэнтры
snap-intersection = Перасячэнні
snap-origin = Пачатак
snap-on-edge = На рабры
snap-on-face = На грані
snap-on-guide = На накіравальных лініях
snap-guide-point = Накіравальныя кропкі
snap-on-red-axis = Чырвоная вось
snap-on-green-axis = Зялёная вось
snap-on-blue-axis = Сіняя вось
snap-parallel-to-edge = Паралельна рабру
snap-perpendicular-to-edge = Перпендыкулярна рабру
snap-from-point = Ад кропкі
snap-tangent = Датычная ў вяршыні
snap-square = Квадрат
snap-golden-section = Залатое сячэнне
snap-half-circle = Паўкруг
snap-grid-point = Вузлы сеткі
snap-on-grid-line = Лініі сеткі

inference-endpoint = Канцавая кропка
inference-midpoint = Сярэдзіна
inference-center = Цэнтр
inference-intersection = Перасячэнне
inference-on-edge = На рабры
inference-on-red-axis = На чырвонай восі
inference-on-green-axis = На зялёнай восі
inference-on-blue-axis = На сіняй восі
inference-parallel-to-edge = Паралельна рабру
inference-perpendicular-to-edge = Перпендыкулярна рабру
inference-from-point = Ад кропкі
inference-on-face = На грані
inference-guide-point = Накіравальная кропка
inference-on-guide = На лініі
inference-origin = Пачатак
inference-square = Квадрат
inference-golden-section = Залатое сячэнне
inference-half-circle = Паўкруг
inference-tangent = Датычная ў вяршыні
inference-grid-point = Вузел сеткі
inference-on-grid-line = На лініі сеткі

## Snapping menu groups

snap-group-points = Кропкі
snap-group-on-objects = На аб’ектах
snap-group-directions = Напрамкі
snap-group-shapes = Фігуры
snap-group-grid = Сетка

## ======== Праўка ========

edit-nothing-selected = нічога не вылучана
edit-nothing-to-copy = няма чаго капіраваць
edit-component-file-empty = файл кампанента пусты
edit-internal-error = унутраная памылка
edit-untitled = Без назвы
# { $error } — тэкст памылкі.
edit-parameters-error = Параметры: { $error }
edit-undo-erase = Сцерці
edit-undo-delete-guides = Выдаліць накіравальныя

# { $label } — назва аперацыі для адмены, { $error } — прычына.
edit-failed = { $label }: не атрымалася: { $error }
edit-op-error = { $label }: { $error }

## ======== Віды ========

view-iso = Ізаметрыя
view-top = Зверху
view-bottom = Знізу
view-front = Спераду
view-back = Ззаду
view-left = Злева
view-right = Справа

## HUD выгляду (выпадальныя спісы бачнай вобласці)

view-style-shaded = Са зліўкай
view-style-textured = З тэкстурамі
view-style-textured-menu = Са зліўкай і тэкстурамі
view-style-monochrome = Манахром
view-style-hidden-line = Схаваныя лініі
view-style-wireframe = Каркас
view-style-realistic = Рэалістычны
# $label — назва стылю.
view-style-xray-suffix = { $label } · Рэнтген
view-parallel = Паралельная
view-perspective = Перспектыва
# $view — назва стандартнага выгляду, $projection — Паралельная / Перспектыва.
view-label = { $view } · { $projection }
view-parallel-projection = Паралельная праекцыя
view-fov = Кут агляду
view-fov-tip = Вертыкальны кут, які бачыць камера з перспектывай (яго можна ўвесці і ў інструменце «Маштаб»)
view-zoom-extents = Паказаць усё
view-viewports = Бачныя вобласці
view-panes-1 = Адзін выгляд
view-panes-2 = Два выгляды побач
view-panes-4 = Чатыры выгляды (2 × 2)
view-presets = Шаблоны бачных абласцей
# $label — бягучы выгляд, напрыклад «Зверху · Паралельная».
view-hud-view-tip = Выгляд: { $label } — стандартныя выгляды, перспектыўная ці паралельная праекцыя, кут агляду, бачныя вобласці
# $label — бягучы стыль.
view-hud-style-tip = Стыль: { $label } — як малююцца грані і рэбры (таксама Выгляд → Стыль граней)
view-xray = Рэнтген
view-edges = Рэбры
view-profiles = Контуры
view-show-axes = Восі
view-show-ground = Зямля
view-show-sky = Неба
view-show-grid = Сетка
view-show-guides = Накіравальныя
view-show-shadows = Цені
view-show-section-planes = Сякучыя плоскасці
view-show-section-cuts = Разрэзы сячэннем
view-show-section-fill = Зліўка сячэнняў
view-show-model-header = Мадэль (усе выгляды)

## Шаблоны бачных абласцей

view-preset-front-iso = Спераду + Ізаметрыя
view-preset-top-iso = Зверху + Ізаметрыя
view-preset-front-right = Спераду + Справа
view-preset-front-left-top-iso = Спераду, Злева, Зверху + Ізаметрыя
view-preset-top-front-right-iso = Зверху, Спераду, Справа + Ізаметрыя
view-preset-front-back-left-right = Спераду, Ззаду, Злева, Справа

## Паведамленні бачнай вобласці

view-nothing-rendered = нічога не намалявана
view-no-renderer = 3D-рэндарар недаступны
# $error — прычына.
view-renderer-unavailable-yet = 3D-рэндарар пакуль недаступны ({ $error })
view-renderer-unavailable = 3D-рэндарар недаступны (патрэбны бэкенд wgpu)
view-group = Група
# $path — ланцужок назваў груп, напрыклад «Стол › Ножка».
view-editing = Праўка: Мадэль › { $path }   (Esc — закрыць)
view-hud-show-tip = Паказаць: восі, зямлю, неба, сетку — у гэтым выглядзе; накіравальныя, цені, сякучыя плоскасці, разрэзы і зліўку — ва ўсіх выглядах (таксама меню Выгляд)

## ======== Панэль інструментаў ========

toolbar-size-small = Малая
toolbar-size-medium = Сярэдняя
toolbar-size-large = Вялікая

## Загалоўкі рэзерву

toolbar-group-draw = Маляваць
toolbar-group-tools = Інструменты
toolbar-group-camera = Камера
toolbar-group-file = Файл
toolbar-group-edit = Праўка
toolbar-group-snapping = Прывязка
toolbar-group-window = Акно

## Кнопкі

toolbar-snapping-menu = Меню прывязкі
toolbar-separator = Раздзяляльнік
toolbar-snapping = Прывязка
# $state — "off", "some" або "on".
toolbar-snapping-tip = Выберыце, да якіх кропак і напрамкаў прывязваецца курсор. { $state ->
        [off] Прывязкі выключаны.
        [some] Частка прывязак выключана.
       *[on] Усе прывязкі ўключаны.
    }

## ======== Радок стану ========

status-grid-step = Крок сеткі
status-measurements-tip = Вымярэнні: проста ўвядзіце значэнне і націсніце Enter (напрыклад, 2500, 2.5m, 300,200, 24s)

## ======== Рабочыя плоскасці ========

workplane-title = Рабочыя плоскасці
workplane-none = Рабочых плоскасцей няма. Дадайце плоскасць праз Інструменты → Накіравальная плоскасць.
workplane-active-tip = Актыўная: малюйце на гэтай плоскасці
workplane-visible = Бачная
workplane-deactivate = Дэактываваць
workplane-no-active-hint = Без актыўнай плоскасці выкарыстоўваюцца восі малявання.
# $name — назва плоскасці.
workplane-world = { $name } (сусветная)
workplane-origin = Пачатак
workplane-angles = Нахіл / азімут / паварот
workplane-tilt-tip = Нахіл: кут нармалі ад сіняй восі
workplane-azimuth-tip = Азімут: напрамак нахілу, ад чырвонай восі
workplane-spin-tip = Паварот вакол нармалі
workplane-undo-delete = Выдаліць рабочую плоскасць
workplane-undo-edit = Змяніць рабочыя плоскасці

## ======== Лісты макета і вектарны экспарт ========

layout-credits-none = Аўтарства: спампаваных мадэлей няма
# $lines — адзін радок на кожную спампаваную мадэль.
layout-credits = Аўтарства:
    { $lines }
layout-col-name = Назва
layout-col-qty = Колькасць
layout-col-size = Памер
layout-col-material = Матэрыял
layout-no-parts = Няма груп і кампанентаў
layout-scale-as-shown = Як на экране
# $n — назоўнік маштабу.
layout-scale-label = Маштаб 1:{ $n }
layout-tb-author = Аўтар
layout-tb-date = Дата
layout-tb-scale = Маштаб
layout-tb-sheet = Ліст
# $n — нумар ліста, $total — колькасць лістоў.
layout-tb-sheet-of = { $n } з { $total }

## ======== Налады ========

prefs-language = Мова
# { $language } — назва мовы сістэмы.
prefs-language-system = Як у сістэме ({ $language })
prefs-language-note = Уласныя пераклады: пакладзіце <code>.ftl у { DATA_DIR() }/locales/ (гл. docs/LOCALIZATION.md); так можна таксама перавызначыць асобныя тэкстуры ўбудаванай мовы.

prefs-section-general = Агульныя
prefs-section-drawing = Маляванне
prefs-section-libraries = Бібліятэкі
prefs-section-downloads = Загрузкі
prefs-section-view = Выгляд
prefs-section-navigation = Навігацыя
prefs-section-interface = Інтэрфейс
prefs-section-shortcuts = Спалучэнні клавіш
prefs-section-toolbar = Панэль інструментаў
prefs-section-render = Рэндар

## Пошукавыя словы раздзелаў (акрамя іх назваў; пераклад перадае свае)

prefs-words-general = аўтазахаванне хвіліны шаблон новы дакумент адзінкі адмена крокі гісторыя рэзервовая копія папярэдняя версія мост python mcp іі агент скрыпты
prefs-words-render = рэндар выява памер сэмплы адскокі якасць шумапрыгнечанне праграмы gif анімацыя oidn open image denoise
prefs-words-drawing = прывязкі прывязка канцавая кропка сярэдзіна рабро грань вось сетка крок крокі перамяшчэнне паварот маштаб кут
prefs-words-libraries = кампаненты матэрыялы папка прыклады ў мадэлі плітка малыя вялікія спіс
prefs-words-downloads = сачыць загрузкі папка браўзер
prefs-words-interface = тэма светлая цёмная сістэма знешні выгляд маштаб памер панэль інструментаў значкі мова пераклад запуск пачатковы экран прывітанне
prefs-words-navigation = мыш трэкпад арбіта маштаб панарама хуткасць інвертаваць кола курсор сярэдняя кнопка два пальцы жэст шчыпок паварот жэст нахіл мультытач тачпад
prefs-words-shortcuts = клавіятура спалучэнні гарачыя клавішы клавішы
prefs-words-toolbar = панэль інструментаў кнопкі парадак рэзерв раздзяляльнік наладзіць інструменты
prefs-words-view = новы дакумент стыль са зліўкай тэкстуры манахром схаваныя лініі каркас рэалістычны рэбры контуры восі зямля неба сетка цені перспектыва паралельная праекцыя кут агляду камера

## Акно

prefs-title = Налады
prefs-search = Пошук
prefs-nothing-found = Нічога не знойдзена
prefs-reset = Скінуць
prefs-reset-tip = Налады гэтага раздзела вяртаюцца да значэнняў па змаўчанні
# $section — назва раздзела.
prefs-defaults-flash = Налады → { $section }: па змаўчанні

## Радкі з папкай або праграмай

# $path — папка.
prefs-folder-default = { $path } (па змаўчанні)
prefs-file-found = { $path } (знойдзена)
prefs-choose = Выбраць…
prefs-default = Па змаўчанні
prefs-show-in-finder = Паказаць у файлавым менеджары
    .macos = Паказаць у Finder
    .windows = Паказаць у Правадыру

## Агульныя

prefs-data-folder = Тэчка даных
prefs-data-folder-note = Тут захоўваюцца плагіны, бібліятэкі, шаблоны, вашы пераклады і мост для Python-скрыптоў і ШІ-агентаў (bridge.json, python/).
prefs-autosave = Аўтазахаванне
prefs-autosave-every = Аўтазахоўваць кожныя
prefs-minutes = хвілін
prefs-autosave-note = Незахаваныя змены запісваюцца ў { DATA_DIR() }/autosave, а не побач з файлам.
prefs-new-document = Новы дакумент
prefs-template-note = Шаблон Файл → Новы (адзінкі і маштаб); яго выгляд задаецца ў раздзеле «Выгляд».
prefs-undo = Адмена
prefs-keep = Захоўваць
prefs-undo-steps = крокаў адмены на дакумент
prefs-undo-note = Кожны крок захоўвае копію мадэлі: вялікія мадэлі з вялікай колькасцю крокаў патрабуюць больш памяці.
prefs-bridge = Скрыпты і ІІ-агенты
prefs-bridge-external = Дазволіць падключэнне скрыптам і ІІ-агентам па-за Varstotas (MCP)
prefs-bridge-note = Выключана: да маста звяртаюцца толькі скрыпты, запушчаныя з Varstotas (Файл → Выканаць скрыпт…, плагіны). Уключана: у { DATA_DIR() }/bridge.json запісваюцца лакальны адрас і токен для іншых праграм, напрыклад MCP-сервера для ІІ-асістэнта. Мост слухае толькі 127.0.0.1.

## Рэндар

prefs-new-documents = Новыя дакументы
prefs-match-view = Падганяць пад выгляд
prefs-match-view-tip = Вышыня залежыць ад формы бачнай вобласці
prefs-denoise = Прыгнятаць шум пры спыненні рэндара
prefs-denoise-ai = Праз Open Image Denoise, калі ўсталяваны
prefs-use-current-doc = Узяць з бягучага дакумента
prefs-use-current-doc-tip = Узяць налады рэндара адкрытага дакумента
prefs-apply-current-doc = Ужыць да бягучага дакумента
prefs-apply-current-doc-tip = Адкрыты дакумент атрымае гэтыя налады (Адмяніць верне іх назад)
prefs-render-note = Файл → Новы пачынае з іх; Акно → Рэндар змяняе налады адкрытага дакумента.
prefs-programs = Праграмы
prefs-oidn = Бібліятэка Open Image Denoise (шумапрыгнятальнік рэндара)
prefs-oidn-missing = не знойдзена: шум прыгнятае ўбудаваны фільтр (усталюйце Open Image Denoise: brew install open-image-denoise або пакет вашай сістэмы)

## Маляванне

prefs-grid = Сетка
prefs-show-grid = Паказваць сетку
prefs-grid-step = крок
prefs-grid-major = асноўная праз
prefs-grid-note = Пры аддаленні крок множыцца на лік асноўных. Пры актыўным Вылучэнні ўвод даўжыні задае крок.
prefs-snapping = Прывязка і прырашчэнні
prefs-snapping-note = Тыя ж пераключальнікі, што ў меню «Прывязка».

## Бібліятэкі

prefs-components = Кампаненты
prefs-library-folder = Папка бібліятэкі
prefs-components-folder-tip = Кампаненты (.vrst) і генератары (.rhai) карткі «Кампаненты»
prefs-show-examples = Паказваць прыклады
prefs-show-examples-tip = Убудаваныя прыклады генератараў
prefs-show-in-model = Паказваць у мадэлі
prefs-show-in-model-tip = Кампаненты і генератары адкрытага дакумента
prefs-materials = Матэрыялы
prefs-materials-folder-tip = Матэрыялы (.vrstmat) карткі «Матэрыялы»; падпапкі — катэгорыі
prefs-show-mat-examples-tip = Матэрыялы, якія пастаўляюцца з Varstotas (дэма «Галерэя матэрыялаў»)
prefs-tiles = Пліткі
prefs-tiles-note = Для абедзвюх картак; таксама ў іх меню «…».

## Загрузкі

prefs-watch-downloads = Сачыць за загрузкамі
prefs-watch-offer = Прапаноўваць новыя файлы мадэляў для імпарту
prefs-watch-offer-tip = Кожны новы файл мадэлі або архіў у папцы прапануецца для імпарту (заўсёды ўключана, пакуль чакае старонка «Іншых сайтаў»)
prefs-watched-folder = Папка назірання
prefs-watched-folder-tip = Куды ваш браўзер захоўвае файлы
prefs-downloads-note = Таксама ў Акно → Анлайн-бібліятэка 3D → Іншыя сайты.

## Панэль інструментаў

prefs-toolbar-note = Кнопкі панэлі інструментаў па парадку; астатнія чакаюць у рэзерве (яны застаюцца ў меню).
prefs-toolbar-on = На панэлі інструментаў
prefs-toolbar-reserve = Рэзерв
prefs-toolbar-to-reserve = У рэзерв
prefs-toolbar-down = Ніжэй
prefs-toolbar-up = Вышэй
prefs-toolbar-to-end = У канец панэлі інструментаў

## Інтэрфейс

prefs-theme = Тэма
prefs-theme-light = Светлая
prefs-theme-dark = Цёмная
prefs-theme-system = Як у сістэме
prefs-theme-note = Бачная вобласць захоўвае свае колеры (неба, зямля); лісты і разгорткі застаюцца белай паперай.
prefs-scale = Маштаб
prefs-scale-note = Меню, панэлі і вокны; %CMDK%+ / %CMDK%− / %CMDK%0 таксама мяняюць яго.
prefs-toolbar-icons = Значкі панэлі інструментаў

## Спалучэнні клавіш

prefs-shortcuts-note = Націсніце спалучэнне, а потым новыя клавішы: Esc адмяняе, Backspace здымае спалучэнне. Клавіша, якая была ў іншай каманды, забіраецца ў яе.
# $command — назва каманды.
prefs-shortcut-removed = { $command }: без спалучэння
# $keys — націснутае спалучэнне, $why — прычына.
prefs-shortcut-reserved = { $keys } не можа быць спалучэннем: { $why }
# $other — каманда, у якой раней было спалучэнне.
prefs-shortcut-taken = { $command }: { $keys } (забрана ў каманды «{ $other }»)
prefs-shortcut-set = { $command }: { $keys }
prefs-shortcuts-tools = Інструменты
prefs-press-keys = Націсніце клавішы…
prefs-default-none = Па змаўчанні: няма
# $keys — спіс спалучэнняў па змаўчанні.
prefs-default-keys = Па змаўчанні: { $keys }

## Навігацыя

prefs-speed = Хуткасць
prefs-nav-orbit = Арбіта
prefs-nav-pan = Панарама
prefs-nav-zoom = Маштаб
prefs-invert-wheel = Інвертаваць кола
prefs-invert-wheel-tip = Кола ад сябе аддаляе
prefs-zoom-cursor = Маштабаваць да курсора
prefs-zoom-cursor-tip = Выключана: да цэнтра выгляду
prefs-mouse = Мыш
prefs-middle-drag = Цяганне сярэдняй кнопкай:
prefs-shift-other = Shift уключае другое.
prefs-trackpad = Трэкпад
prefs-two-fingers = Два пальцы:
prefs-trackpad-note = Shift мяняе кручэнне і панараму месцамі. Шчыпок і %CMD% + два пальцы мяняюць маштаб незалежна ад кола мышы.

## Выгляд (значэнні па змаўчанні для новага дакумента)

prefs-view-note = Як выглядае новы дакумент (Файл → Новы, Новы з шаблона). Меню «Выгляд» і кнопкі бачнай вобласці мяняюць бягучы выгляд.
prefs-camera = Камера
prefs-fov = Кут агляду
prefs-use-current-view = Узяць з бягучага выгляду
prefs-use-current-view-tip = Узяць стыль, пераключальнікі і камеру актыўнай бачнай вобласці
prefs-apply-current-view = Ужыць да бягучага выгляду
prefs-apply-current-view-tip = Наладзіць актыўную бачную вобласць як новы дакумент

## ======== Спалучэнні клавіш ========

key-new = Новы
key-open = Адкрыць…
key-save = Захаваць
key-save-as = Захаваць як…
key-quit = Выйсці
key-undo = Адмяніць
key-redo = Вярнуць
key-select-all = Вылучыць усё
key-select-none = Зняць вылучэнне
key-make-group = Стварыць групу
key-zoom-extents = Паказаць усё
key-toggle-snapping = Прывязка ўкл. / выкл.
key-next-scene = Наступная сцэна
key-prev-scene = Папярэдняя сцэна

## Назвы клавіш і зарэзерваваныя клавішы (Налады → Спалучэнні клавіш)

key-name-space = Прабел
key-name-pgdn = PgDn
key-name-pgup = PgUp
key-reserved-tools = гэту клавішу выкарыстоўвае кожны інструмент
key-reserved-arrows = стрэлкі фіксуюць восі прывязкі
key-reserved-f1 = F1 адкрывае даведку
key-reserved-digits = лічбы ўводзяцца ў «Вымярэнні»
key-reserved-measurements = гэтая клавіша ўводзіць тэкст у «Вымярэнні»
key-reserved-clipboard = Выразаць / Капіраваць / Уставіць

## ======== Шаблоны ========

template-architectural-mm = Архітэктурны — міліметры
template-architectural-m = Архітэктурны — метры
template-woodworking-mm = Сталярны — міліметры
template-imperial = Імперскі — цалі

template-architectural-mm-desc = Будынкі і інтэр’еры, даўжыні ў міліметрах.
template-architectural-m-desc = Будынкі і ўчасткі, даўжыні ў метрах.
template-woodworking-mm-desc = Мэбля і дробныя прадметы: набліжаны выгляд, даўжыні ў міліметрах.
template-imperial-desc = Даўжыні ў цалях.

## ======== Бакавая панэль ========

## Карткі (назвы — ідэнтыфікатары ў наладах; тут паказаныя назвы)

tray-card-entity-info = Уласцівасці
tray-card-materials = Матэрыялы
tray-card-material-editor = Рэдактар матэрыялу
tray-card-components = Кампаненты
tray-card-tags = Тэгі
tray-card-outliner = Структура
tray-card-scenes = Сцэны
tray-card-shadows = Цені
tray-card-environment = Асяроддзе
tray-card-soften-edges = Згладжванне рэбраў
tray-card-instructor = Падказкі
tray-card-model-info-tip = Звесткі пра дакумент: статыстыка дакумента, Выдаліць нявыкарыстанае
tray-cards-all-off = Усе карткі выключаны: Акно → Бакавая панэль

tray-untagged = Без тэга
tray-default = Па змаўчанні
tray-rename = Перайменаваць
tray-invalid-value = Няправільнае значэнне
# $error — прычына.
tray-kernel-not-ready = (ядро не гатовае: { $error })
tray-through = Наскрозь

## Цені

tray-shadows-show = Паказваць цені
tray-shadows-time = Час
tray-shadows-date = Дата
tray-shadows-light = Святло
tray-shadows-dark = Цемра
tray-shadows-display = Адлюстраванне
tray-shadows-on-faces = На граньях
tray-shadows-on-ground = На зямлі
tray-shadows-location = Месцазнаходжанне
tray-shadows-latitude = Шырата
tray-shadows-longitude = Даўгата
tray-shadows-utc-offset = Зрух UTC
tray-shadows-north-angle = Кут поўначы
# $alt — вышыня сонца ў градусах.
tray-sun-above = Сонца на { $alt }° над гарызонтам
tray-sun-below = Сонца пад гарызонтам

## Згладжванне рэбраў

tray-soften-allow-normals = Дапушчальны кут паміж нармалямі
tray-soften-smooth-normals = Згладзіць нармалі
tray-soften-coplanar = Згладжваць компланарныя
tray-soften-select = Вылучыце рэбры або грані
# $n — колькасць рэбраў.
tray-soften-edges-count = { $n ->
        [one] { $n } рабро
        [few] { $n } рабры
        [many] { $n } рэбраў
       *[other] { $n } рэбраў
    }

## Уласцівасці

tray-info-invalid-context = Недапушчальны кантэкст
tray-info-no-selection = Нічога не вылучана
tray-info-units = Адзінкі
tray-unit-millimeters = Міліметры
tray-unit-centimeters = Сантыметры
tray-unit-meters = Метры
tray-unit-inches = Цалі
tray-unit-feet = Футы
tray-info-faces = Грані
tray-info-edges = Рэбры
tray-info-groups = Групы
tray-info-components = Кампаненты
tray-info-face = Грань
tray-info-area = Плошча
tray-info-front = Пярэдняя
tray-info-back = Задняя
tray-info-tag = Тэг
tray-info-edge = Рабро
tray-info-length = Даўжыня
tray-info-soft = Мяккае
tray-info-smooth = Гладкае
tray-info-group = Група
tray-info-component = Кампанент
tray-info-parametric-group = Параметрычная група
# $kind — Група, Кампанент або Параметрычная група.
tray-info-solid = Цела: { $kind }
tray-info-volume = Аб’ём
tray-info-instance = Асобнік
tray-info-name-hint = назва
tray-info-definition = Азначэнне
tray-info-material = Матэрыял
tray-info-source = Крыніца
tray-info-licence-unknown = ліцэнзія невядомая
# $line — радок аўтарства мадэлі.
tray-info-source-hover =
    { $line }
    Файл → Экспарт → Атрыбуцыі паказвае аўтарства мадэлі
# $count — колькасць вылучаных элементаў.
tray-info-n-guides = { $count ->
        [one] { $count } накіравальная
        [few] { $count } накіравальныя
        [many] { $count } накіравальных
       *[other] { $count } накіравальнай
    }
tray-info-n-dimensions = { $count ->
        [one] { $count } памер
        [few] { $count } памеры
        [many] { $count } памераў
       *[other] { $count } памеру
    }
tray-info-n-entities = { $count ->
        [one] { $count } аб’ект
        [few] { $count } аб’екты
        [many] { $count } аб’ектаў
       *[other] { $count } аб’екта
    }
tray-info-contains = Змяшчае
tray-count-faces = { $n ->
        [one] { $n } грань
        [few] { $n } грані
        [many] { $n } граняў
       *[other] { $n } грані
    }
tray-count-edges = { $n ->
        [one] { $n } рабро
        [few] { $n } рабры
        [many] { $n } рэбраў
       *[other] { $n } рабра
    }
tray-count-groups = { $n ->
        [one] { $n } група
        [few] { $n } групы
        [many] { $n } груп
       *[other] { $n } групы
    }
tray-count-components = { $n ->
        [one] { $n } кампанент
        [few] { $n } кампаненты
        [many] { $n } кампанентаў
       *[other] { $n } кампанента
    }
tray-count-guides = { $n ->
        [one] { $n } накіравальная
        [few] { $n } накіравальныя
        [many] { $n } накіравальных
       *[other] { $n } накіравальнай
    }
tray-count-dimensions = { $n ->
        [one] { $n } памер
        [few] { $n } памеры
        [many] { $n } памераў
       *[other] { $n } памеру
    }

## Назвы крокаў адмены

tray-undo-rename = Перайменаваць
tray-undo-soften-edge = Згладзіць рабро
tray-undo-unsoften-edge = Зрабіць рабро рэзкім
tray-undo-set-tag = Задаць тэг
tray-undo-texture-position = Размяшчэнне тэкстуры
tray-undo-edit-material = Змяніць матэрыял
tray-undo-edit-segments = Змяніць сегменты
tray-undo-edit-radius = Змяніць радыус
tray-undo-edit-text = Змяніць тэкст
tray-undo-edit-dimension-text = Змяніць тэкст памеру
tray-undo-rename-section = Перайменаваць сячэнне
tray-undo-delete-tag = Выдаліць тэг
tray-undo-rename-tag = Перайменаваць тэг
tray-undo-add-tag = Дадаць тэг
tray-undo-scene-properties = Уласцівасці сцэны
tray-undo-rename-scene = Перайменаваць сцэну

## Параметрычная група

tray-params-make = Зрабіць параметрычнай
tray-params-make-hover = Памер і колькасць копій задаюцца ва «Уласцівасцях»
tray-params-title = Параметры
tray-params-length = Даўжыня (X)
tray-params-width = Шырыня (Y)
tray-params-height = Вышыня (Z)
tray-params-gap = Зазор
tray-params-copies = Копіі
tray-params-along = Уздоўж
tray-params-resize = Змяніць памер
tray-params-stretch = Расцягнуць
tray-params-stretch-hover = Зрушвае далёкую палову: канцы захоўваюць памер
tray-params-scale = Маштаб

## Тэкстура грані

tray-tex-title = Тэкстура
tray-tex-offset-x = Зрух X
tray-tex-offset-y = Зрух Y
tray-tex-rotation = Паварот
tray-tex-scale-x = Маштаб X
tray-tex-scale-y = Маштаб Y
tray-tex-reset = Скінуць
tray-tex-tile-w = Шырыня пліткі
tray-tex-tile-h = Вышыня пліткі
uv-title = UV-рэдактар
uv-undo = Праўка UV
uv-undo-mark-seams = Адзначыць швы
uv-undo-clear-seams = Зняць швы
uv-seams-marked = Швамі адзначана рэбраў: { $count }.
uv-seams-cleared = Знята швоў: { $count }.
uv-no-edges = Спачатку вылучыце рэбры (разам з гранямі).
uv-nothing = Няма чаго мяняць.
uv-empty = Вылучыце грані з тэкстурай (або пафарбуйце іх), каб правіць раскладку тэкстуры.
uv-unfold-hover = Раскласці кожны востраў плоска ў натуральную велічыню, разрэзаўшы па швах
uv-planar = Плоская
uv-box = Кубічная
uv-cylinder = Цыліндрычная
uv-sphere = Сферычная
uv-seams-mark-hover = Адзначыць вылучаныя рэбры як швы
uv-seams-clear-hover = Зняць швы з вылучаных рэбраў (інакш усе)
uv-seams-hard-hover = Адзначыць швом кожнае жорсткае (не змякчанае) рэбра паміж гранямі
uv-rotate-ccw = Павярнуць на 90° супраць гадзіннікавай
uv-rotate = Павярнуць на 90° па гадзіннікавай
uv-flip-u = Адлюстраваць злева ↔ направа
uv-flip-v = Адлюстраваць зверху ↕ уніз
uv-fit-hover = Маштабаваць кожны востраў на адну плітку
uv-pack-hover = Раскласці астравы ў адной плітцы ў адным маштабе
uv-align-hover = Павярнуць кожны востраў, каб яго самае доўгае рэбра ішло проста
uv-reset-hover = Скінуць становішча тэкстуры на астравах (вярнуць зыходнае накладанне)
uv-tool-select = Выбар: пстрычка — востраў (у рэжыме граняў — грань), Shift+пстрычка дадае або прыбірае, рамка — працягнуць па пустым месцы; перацягванне выбранага рухае яго
uv-tool-move = Рухаць выбраныя астравы або грані (чырвоная / зялёная стрэлка гізма — уздоўж адной восі)
uv-tool-rotate = Паварочваць выбраныя астравы або грані вакол апорнай кропкі
uv-tool-scale = Маштабаваць выбраныя астравы або грані ад апорнай кропкі (чырвоны / зялёны квадрат гізма — па адной восі)
uv-mode-island = Выбіраць астравы цалкам
uv-mode-face = Выбіраць асобныя грані
uv-select-all = Выбраць усё
uv-frame = Паказаць усе астравы
uv-project-hover = Праекцыя тэкстуры: плоская, скрыня, цыліндр або сфера
uv-reset-rotation = Скінуць паварот: развярнуць астравы так, каб тэкстура ішла роўна (форма разгорткі захоўваецца)
uv-reset-scale = Скінуць маштаб: вярнуць тэкстуры яе ўласны памер (форма разгорткі захоўваецца)
uv-hint-select = Пстрычка / рамка: выбар · пстрычка яшчэ раз: тая, што ніжэй · Shift: дадаць або прыбраць · цягнуць выбранае: рухаць · кола: маштаб · правая кнопка: панарама
uv-hint-move = Цягнуць: рухаць выбранае · стрэлкі гізма: па адной восі · прыліпае да вуглоў і сеткі плітак (Alt: без прыліпання; Shift: крокі ўкл/выкл) · увесці dx; dy
uv-hint-rotate = Цягнуць: паварот вакол апорнай кропкі · цэнтр гізма можна перацягнуць, двайная пстрычка — у цэнтр · Shift: крок вугла ўкл/выкл · увесці вугал
uv-hint-scale = Цягнуць: маштаб ад апорнай кропкі · чырвоны / зялёны квадрат: адна вось, аранжавы: абедзве · цэнтр гізма можна перацягнуць, двайная пстрычка — у цэнтр · Shift: крокі ўкл/выкл · увесці k або kx; ky
uv-value-move = Зрух
uv-value-rotate = Вугал
uv-value-scale = Маштаб
uv-value-hover = Дакладнае значэнне, Enter ужывае; адразу пасля перацягвання замяняе яго
uv-checker-hover = Паказаць шахматку замест тэкстур (тут і ў 3D-выглядзе), каб бачыць расцяжэнні
uv-density = Шчыльнасць тэкселяў
uv-px-per-m = пікс/м
uv-summary = Граняў: { $faces } · астравоў: { $islands } · выбрана: { $selected }
uv-material = Матэрыял: { $name }
uv-map-hover = Якая карта матэрыялу паказана пад разгорткай
uv-no-texture = Няма тэкстуры: у граняў і бягучага матэрыялу няма выявы
uv-tile-hover = Паўтараць тэкстуру па ўсім полі (інакш — адна плітка)
tray-tex-width = Шырыня тэкстуры
tray-tex-height = Вышыня тэкстуры
tray-tex-keep-aspect = Захоўваць прапорцыі

## Крывыя

tray-curve = Крывая
tray-curve-closed = Замкнёная крывая
tray-curve-points = Кропкі
# $count — колькасць кропак, $smooth — колькі з іх гладкія.
tray-curve-points-value = { $count } (гладкіх: { $smooth })
tray-curve-segments = Сегменты
tray-curve-segments-hover = Рэбраў на выгнуты ўчастак паміж дзвюма кропкамі
tray-curve-edit = Змяніць крывую
tray-curve-edit-hover = Цягніце яе кропкі і ручкі (або двойчы пстрыкніце крывую)
tray-curve-arc = Дуга
tray-curve-circle = Акружнасць
tray-curve-polygon = Шматкутнік
tray-curve-radius = Радыус
tray-curve-sides = Старон
tray-curve-circumference = Даўжыня акружнасці
tray-curve-arc-length = Даўжыня дугі
tray-curve-angle = Кут

## Памер, святло, сякучая плоскасць, накіравальныя

tray-dim-text = Тэкст
tray-dim-linear = Лінейны памер
tray-light-title = Святло
tray-light-render = Рэндар…
tray-light-render-hover = Акно → Рэндар
tray-light-hint = Перамясціць / Павярнуць перамяшчаюць і паварочваюць яго; цягніце квадратную ручку, каб накіраваць.
tray-section-title = Сякучая плоскасць
tray-section-name = Назва
tray-section-active-cut = Актыўны разрэз
tray-section-normal = Нармаль
tray-section-reverse = Перавярнуць
tray-section-align-view = Выраўнаваць выгляд
tray-guide-line = Накіравальная лінія
tray-guide-direction = Напрамак
tray-guide-point = Накіравальная кропка
tray-guide-position = Пазіцыя

## Картка матэрыялаў

tray-mat-current = Бягучы матэрыял для заліўкі (B)
tray-mat-custom = Свой
tray-mat-use = Ужыць

## Тэгі

tray-tags-add = Дадаць тэг
tray-tags-header = Бачнасць · Назва
tray-tags-untagged-hover = Пераключае бачнасць геаметрыі, груп і кампанентаў верхняга ўзроўню без тэга (іх змесціва належыць ім)
tray-tags-toggle = Пераключыць бачнасць
tray-tags-rename-hover = Двойчы пстрыкніце, каб перайменаваць; правая кнопка — больш
tray-tags-delete-hover = Выдаліць тэг (тое, што ім пазначана, стане без тэга)
# $n — нумар новага тэга.
tray-tag-new-name = Тэг { $n }
# $name — тэг, $uses — колькі элементаў ім пазначана.
tray-tag-deleted = Тэг «{ $name }» выдалены
tray-tag-deleted-uses = Тэг «{ $name }» выдалены: элементаў без тэга цяпер — { $uses }

## Структура

tray-outliner-empty = Груп і кампанентаў пакуль няма.
tray-outliner-hide-show = Схаваць / паказаць
tray-outliner-zoom = Паказаць
tray-outliner-zoom-hover = Вылучыць і паказаць у выглядзе (двайны пстрык)
tray-outliner-edit-group = Змяніць групу
tray-outliner-edit-component = Змяніць кампанент
tray-outliner-unhide = Паказаць
tray-outliner-hide = Схаваць
tray-outliner-explode = Раздзяліць
tray-outliner-make-unique = Зрабіць унікальным
tray-outliner-make-unique-hover = Сваё азначэнне: змена не закранае іншых копій
tray-outliner-move-out = Вынесці з групы
tray-outliner-move-out-hover = У групу / кампанент вакол гэтага (або перацягніце радок)

## Сцэны

tray-scenes-add = Дадаць сцэну з бягучага выгляду
tray-scenes-update = Абнавіць вылучаную сцэну па бягучым выглядзе
tray-scenes-delete = Выдаліць вылучаную сцэну
tray-scenes-up = Перасунуць сцэну вышэй
tray-scenes-down = Перасунуць сцэну ніжэй
tray-scenes-play = Прайграць
tray-scenes-pause = Паўза
tray-scenes-play-hover = Прайграць анімацыю сцэн (Выгляд → Анімацыя)
tray-scenes-empty = Сцэн няма. Націсніце +, каб захаваць бягучы выгляд.
tray-scenes-click-hover = Пстрыкніце, каб перайсці да гэтай сцэны, двойчы — каб перайменаваць
tray-scenes-include = Уключыць у анімацыю
tray-scenes-animation = Анімацыя
tray-scenes-transitions = Пераходы паміж сцэнамі
tray-scenes-transition = Пераход
tray-scenes-delay = Затрымка сцэны

## Падказкі

# $key — спалучэнне клавіш.
tray-instructor-shortcut = Спалучэнне: { $key }
tray-instructor-more = Больш — у Кіраўніцтве карыстальніка (F1)
tray-instructor-not-implemented = Гэты інструмент яшчэ не рэалізаваны.

## Карткі бібліятэк, агульны каркас (tray_lib)

tray-tiles-small = Малыя пліткі
tray-tiles-large = Вялікія пліткі
tray-tiles-list = Спіс
tray-lib-search = Пошук
tray-lib-refresh = Абнавіць: перачытаць папку бібліятэкі
tray-lib-more = Папка бібліятэкі, што паказваць, памер пліткі
tray-lib-section-show = Паказаць раздзел
tray-lib-section-fold = Згарнуць раздзел
# $unused — колькасць нявыкарыстаных элементаў.
tray-purge-materials = Выдаліць нявыкарыстанае: прыбраць матэрыялы, якіх не выкарыстоўвае мадэль (іх { $unused })
tray-purge-materials-none = Выдаліць нявыкарыстанае: нявыкарыстаных матэрыялаў няма
tray-purge-components = Выдаліць нявыкарыстанае: прыбраць кампаненты, якіх не выкарыстоўвае мадэль (іх { $unused })
tray-purge-components-none = Выдаліць нявыкарыстанае: нявыкарыстаных кампанентаў няма
tray-lib-fav-add = Дадаць у абранае
tray-lib-fav-remove = Прыбраць з абранага
tray-lib-move-to-folder = Перамясціць у папку
tray-my-library = Мая бібліятэка
# $path — файл, які перашкаджае.
tray-lib-exists = { $path } ужо існуе
tray-folder-new = Новая папка…
tray-folder-rename = Перайменаваць папку…
tray-folder-delete = Выдаліць папку…
tray-folder-delete-files = Выдаліць папку і яе файлы…

## ======== Бібліятэка кампанентаў ========

## Раздзелы

library-examples = Прыклады
library-examples-hover = Генератары, якія пастаўляюцца з Varstotas (не файлы)
library-in-model = У мадэлі
library-in-model-hover = Кампаненты і генератары адкрытага дакумента (Захаваць у бібліятэку стварае файл)
library-favourites = Абранае
library-favourites-hover = Пстрыкніце правай кнопкай плітку бібліятэкі → Дадаць у абранае
library-recent = Нядаўнія
library-recent-hover = Кампаненты бібліятэкі, размешчаныя апошнімі
library-hide-examples = Схаваць убудаваныя прыклады («…» → Паказваць прыклады вяртае іх)
library-hide-in-model = Схаваць кампаненты дакумента («…» → Паказваць у мадэлі вяртае іх)
# $path — папка.
library-folder-reveal-hover =
    { $path }
    Пстрыкніце, каб паказаць у Finder
    .windows =
    { $path }
    Пстрыкніце, каб паказаць у Правадыру
library-show-in-finder = Паказаць у Finder
    .windows = Паказаць у Правадыру
    .linux = Паказаць у файлавым менеджары

## Верхні радок і меню «…»

library-plus-hover = Захаваць вылучанае ў бібліятэку, новы генератар
library-online-hover = Анлайн-бібліятэка 3D: бясплатныя мадэлі з сеткі (Poly Haven, Smithsonian, NASA, Kenney, убудовы)
library-save-selection = Захаваць вылучанае ў бібліятэку…
library-select-first = Спачатку нешта вылучыце
library-new-generator = Новы генератар…
library-new-generator-hover = Кампанент, пабудаваны скрыптом з параметрамі (Акно → Рэдактар генератараў)
library-choose-folder = Выбраць папку…
library-default-folder = Папка па змаўчанні
library-default-folder-hover = Назад да { DATA_DIR() }/components
library-purge = Выдаліць нявыкарыстаныя кампаненты
library-purge-hover = Прыбраць кампаненты (і азначэнні груп), якіх не выкарыстоўвае мадэль
library-show-examples = Паказваць прыклады
library-show-examples-hover = Убудаваныя прыклады генератараў
library-show-in-model = Паказваць у мадэлі
library-show-in-model-hover = Кампаненты і генератары адкрытага дакумента
library-empty = Кампанентаў пакуль няма: вылучыце нешта і абярыце + → Захаваць вылучанае ў бібліятэку.
library-nothing-found = Нічога не знойдзена
library-image-filter = Выява
library-name = Назва
library-save = Захаваць
# $n — колькасць элементаў.
library-unreadable-files = Не ўдалося прачытаць файлаў: { $n }
library-unreadable-generators = Не ўдалося прачытаць генератараў: { $n }

## Пліткі і іх кантэкстныя меню

# $name — кампанент, $note — дадатковы радок (заканчваецца пераносам радка або пусты), $x $y $z — памер.
library-tile-hover =
    { $name }
    { $note }{ $x } × { $y } × { $z }
    Пстрыкніце, каб размясціць
# $uses — колькі копій размешчана.
library-in-document = У гэтым дакуменце: размешчана { $uses }
library-place = Размясціць
library-select-instances = Вылучыць асобнікі
library-preview-from-view = Задаць перадпрагляд з выгляду
library-preview-from-view-file-hover = Бачная вобласць у цяперашнім выглядзе стане выявай (захоўваецца ў файле): спачатку ўпішыце мадэль у кадр
library-preview-from-view-def-hover = Бачная вобласць у цяперашнім выглядзе стане выявай пліткі (захоўваецца разам з дакументам): спачатку ўпішыце кампанент у кадр
library-preview-image = Задаць выяву перадпрагляду…
library-preview-image-file-hover = Выява, якая паказваецца тут замест намаляванай мадэлі (захоўваецца ў файле)
library-preview-image-def-hover = Выява, якая паказваецца на плітцы замест адрэндэранай мадэлі
library-rename-ellipsis = Перайменаваць…
library-delete-ellipsis = Выдаліць…
library-save-to-library = Захаваць у бібліятэку
library-save-to-library-hover = Запісаць у папку бібліятэкі, для любога дакумента
library-parameters = Параметры…
library-parameters-hover = Акно → Параметры кампанента: значэнні, якія рухаюць, паказваюць або фарбуюць яго часткі
library-delete-from-document = Выдаліць з дакумента…
library-delete-from-document-hover = Кампанент і ўсе яго размешчаныя копіі

## Выявы перадпрагляду

library-undo-set-preview = Задаць выяву перадпрагляду
library-undo-remove-preview = Прыбраць выяву перадпрагляду
library-undo-set-def-preview = Задаць перадпрагляд кампанента
library-undo-remove-def-preview = Прыбраць перадпрагляд кампанента
library-preview-set-file = Выява перадпрагляду задана (захавана разам з файлам)
library-preview-removed = Выява перадпрагляду прыбрана
# $name — кампанент.
library-preview-set-def = Выява перадпрагляду для { $name } задана (захавана разам з дакументам)
library-preview-removed-def = Выява перадпрагляду для { $name } прыбрана
# $path — файл выявы, $error — прычына.
library-preview-error-file = Выява перадпрагляду: { $path }: { $error }
library-preview-error = Выява перадпрагляду: { $error }

## Захаванне, размяшчэнне, перамяшчэнне

# $path — файл, $name — кампанент, $error — прычына.
library-exists = { $path } ужо існуе
library-save-error = Захаваць у бібліятэку: { $error }
library-saved = Захавана ў бібліятэку: { $path }
library-save-failed = Не ўдалося захаваць у бібліятэку: { $error }
library-place-failed = Размясціць { $name }: { $error }
library-no-copies = На верхнім узроўні мадэлі няма копій гэтага кампанента
library-move-failed = Перамясціць у папку: { $error }

## Дыялогі папак

library-folder-new-title = Новая папка
library-folder-rename-title = Перайменаваць папку
library-folder-name = Назва папкі
library-create = Стварыць
library-rename = Перайменаваць
# $heading — загаловак дыялогу, $name — папка, $error — прычына.
library-folder-done = { $heading }: { $name }
library-folder-failed = { $heading }: { $error }
library-folder-delete-title = Выдаліць папку?
# $name — папка.
library-folder-delete-text-materials = Папка «{ $name }» будзе выдалена з бібліятэкі матэрыялаў:
library-folder-delete-text-components = Папка «{ $name }» будзе выдалена з бібліятэкі кампанентаў:
library-folder-empty = Папка пустая.
# $files — колькасць файлаў, $dirs — колькасць падпапак.
library-folder-what = Файлаў: { $files }
library-folder-what-dirs = Файлаў: { $files }, падпапак: { $dirs }
# $what — тэкст library-folder-what.
library-folder-not-empty = ⚠ Папка не пустая, разам з ёй будзе выдалена: { $what }.
# $n — колькі файлаў не пералічана.
library-folder-more = … і яшчэ { $n }
library-undone-copies-stay = Гэта нельга адмяніць. Копіі, ужо размешчаныя ў дакументах, застаюцца.
library-folder-delete-with = Выдаліць разам з файламі ({ $files })
library-folder-deleted = Папка { $name } выдалена
library-folder-deleted-files = Папка { $name } выдалена разам з файламі ({ $files })
library-folder-delete-failed = Выдаліць папку: { $error }

## Дыялогі выдалення

library-delete-title = Выдаліць з бібліятэкі?
library-delete-text = «{ $name }» будзе выдалена з папкі бібліятэкі:
library-delete-doc-title = Выдаліць з дакумента?
library-delete-gen-text = Генератар «{ $name }» будзе прыбраны з гэтага дакумента.
# $uses — колькасць размешчаных кампанентаў.
library-delete-gen-uses = Размешчаныя кампаненты ({ $uses }) стануць звычайнымі, без параметраў.
library-undo-brings-back = Праўка → Адмяніць верне яго.
library-undo-brings-them-back = Праўка → Адмяніць верне іх.
library-undo-delete-generator = Выдаліць генератар { $name }
# $n — колькасць кампанентаў, якія засталіся без параметраў.
library-generator-removed-uses = Генератар { $name } прыбраны: кампанентаў без параметраў — { $n }
library-generator-removed = Генератар { $name } прыбраны
library-deleted = Выдалена з бібліятэкі: { $name }
library-delete-failed = Выдаліць { $path }: { $error }
library-undo-delete-component = Выдаліць кампанент { $name }
# $n — колькасць выдаленых копій.
library-component-deleted-copies = Выдалены { $name } і размешчаных копій: { $n }
library-component-deleted = Кампанент { $name } выдалены з дакумента
library-delete-def-text = Размяшчэнняў «{ $name }»: { $uses }. Кампанент і ўсе яго копіі будуць выдалены.

## Дыялог перайменавання

library-rename-material = Перайменаваць матэрыял
library-rename-component = Перайменаваць кампанент
# $file — файл бібліятэкі.
library-rename-file-too = Файл бібліятэкі { $file } таксама перайменуецца.
library-undo-rename-component = Перайменаваць кампанент { $name }
library-renamed = Перайменавана ў { $name }
library-rename-failed = Перайменаваць { $path }: { $error }

## ======== Матэрыялы ========

## Картка бібліятэкі

mat-plus-hover = Новы матэрыял з выявы, імпарт файлаў матэрыялаў
mat-online-hover = Анлайн-бібліятэка матэрыялаў: бясплатныя PBR-матэрыялы (Poly Haven, ambientCG, убудовы)
mat-new-from-texture = Новы з тэкстуры…
mat-new-from-texture-hover = Матэрыял з файла выявы (PNG, JPEG)
mat-new-procedural = Новы працэдурны…
mat-new-procedural-hover = Бясшвовая тэкстура па шаблоне: цэгла, плітка, дошкі, шум або шахматка
proc-title = Працэдурны матэрыял
proc-bricks = Цэгла
proc-tiles = Плітка
proc-planks = Дошкі
proc-noise = Шум
proc-checker = Шахматка
proc-main = Колер
proc-joint = Швы
proc-second = Другі колер
proc-count = Элементаў упоперак × уверх
proc-scale = Маштаб
proc-joint-width = Шырыня шва
proc-variation = Разкід
proc-seed = Варыянт
proc-another = Іншы
proc-width = Шырыня пліткі
proc-name = Назва
proc-hint = Плітка паўтараецца без швоў; у папярэднім праглядзе — 2 × 2 пліткі.
proc-create = Стварыць матэрыял
proc-plugin-hover = Узор з плагіна (@texture_generator)
proc-generating = Генерацыя…
proc-created = Створаны матэрыял { $name }: пстрыкайце па гранях, каб пафарбаваць
mat-import = Імпарт…
mat-import-hover = Дадаць файлы матэрыялаў (.vrstmat, фарматы ўбудоў) у дакумент
mat-choose-folder = Выбраць папку…
mat-default-folder = Папка па змаўчанні
mat-default-folder-hover = Назад да { DATA_DIR() }/materials
mat-purge = Выдаліць нявыкарыстаныя матэрыялы
mat-purge-hover = Прыбраць матэрыялы, якіх не выкарыстоўвае мадэль
mat-show-examples-hover = Матэрыялы, якія пастаўляюцца з Varstotas (дэма «Галерэя матэрыялаў»)
mat-in-model-hover = Матэрыялы адкрытага дакумента
mat-favourites-hover = Пстрыкніце правай кнопкай матэрыял бібліятэкі → Дадаць у абранае
mat-recent-hover = Матэрыялы бібліятэкі, выкарыстаныя апошнімі
mat-examples-hover = Матэрыялы, якія пастаўляюцца з Varstotas (дэма «Галерэя матэрыялаў»)
mat-empty = Пуста: пстрыкніце правай кнопкай матэрыял мадэлі → Захаваць у бібліятэку
mat-use = Ужыць
mat-use-hover = Фарбаваць ім (ён дадаецца ў мадэль, калі нешта афарбавана)
mat-export = Экспарт…
mat-save-hover = Запісаць у папку бібліятэкі матэрыялаў, для любога дакумента
mat-copy-hover = Скапіяваць у вашу бібліятэку матэрыялаў
mat-delete-model-hover = Прыбраць з мадэлі: тое, што ім афарбавана, атрымае матэрыял па змаўчанні (Адмяніць верне яго)
mat-replace = Замяніць бягучым матэрыялам
mat-replace-hover = Перафарбаваць усё, што пафарбавана гэтым матэрыялам, бягучым матэрыялам і выдаліць яго з мадэлі (Адмяніць верне яго)
mat-undo-replace = Замяніць { $name } на { $with }
mat-replaced = { $name } → { $with }: перафарбавана { $n }
mat-file-filter = Матэрыял Varstotas
mat-all-filter = Усе матэрыялы

## Апісанне пліткі (падказка)

mat-kind-metal = метал
mat-kind-glass = шкло
mat-kind-emissive = свеціцца
mat-kind-texture = тэкстура
# $name — шэйдарны шаблон.
mat-kind-shader = шэйдар { $name }
mat-kind-shader-custom = свой
# $value — шурпатасць, ад 0 да 1.
mat-kind-roughness = шурпатасць { $value }

## Паведамленні

mat-undo-add = Дадаць матэрыял { $name }
mat-undo-delete = Выдаліць матэрыял { $name }
mat-undo-rename = Перайменаваць матэрыял { $name }
mat-saved = Захавана ў бібліятэку матэрыялаў: { $name }
mat-save-failed = Не ўдалося захаваць матэрыял: { $error }
mat-exported = Экспартавана { $path }
mat-export-failed = Не ўдалося экспартаваць матэрыял: { $error }
# $names — імпартаваныя матэрыялы, $plugin — імпартар.
mat-imported-with = Імпартавана { $names } праз { $plugin }
mat-import-empty = { $plugin }: матэрыялаў у { $path } няма
mat-import-failed-with = Не ўдаецца імпартаваць { $path } праз { $plugin }: { $error }
mat-imported = Імпартаваны матэрыял { $name }
mat-import-failed = Не ўдаецца імпартаваць { $path }: { $error }
# $n — колькасць афарбаваных элементаў, вернутых да матэрыялу па змаўчанні.
mat-deleted-uses = Матэрыял { $name } выдалены, выкарыстанняў вернута да матэрыялу па змаўчанні: { $n } (Адмяніць верне)
mat-deleted = Матэрыял { $name } выдалены (Адмяніць верне)
mat-delete-failed = Не ўдаецца выдаліць матэрыял { $name }
mat-rename-exists = Матэрыял «{ $name }» ужо існуе
mat-file-exists = { $path } ужо існуе
mat-delete-title = Выдаліць з бібліятэкі матэрыялаў?
mat-delete-undone = Гэта нельга адмяніць. Дакументы, якія яго выкарыстоўваюць, захоўваюць сваю копію.
mat-deleted-from-library = Выдалена з бібліятэкі матэрыялаў: { $name }

## Матэрыялы анлайн (акно і каталогі)

mat-online-title = Анлайн-бібліятэка матэрыялаў
mat-online-plugins-hint =
    Убудовы з @material_source
    паказаны тут
mat-online-search = Пошук: цэгла, дуб, бетон…
mat-online-description = Бясплатныя фотаскананы CC0 (колер, нармалі, шурпатасць, аклюзія, карты металу ў рэальным памеры). Загрузкі ідуць у бібліятэку матэрыялаў.
mat-online-click-hint = Пстрыкніце матэрыял, каб паглядзець; двайны пстрык — спампаваць і фарбаваць ім.
# $width — даўжыня.
mat-online-wide = шырыня { $width }
mat-online-use-hover = Ужо ў бібліятэцы: фарбаваць ім
mat-online-download-use = Спампаваць і ўжыць
mat-online-download-use-hover = Захаваць у бібліятэку, дадаць у мадэль і фарбаваць
mat-online-download-hover = Захаваць у бібліятэку матэрыялаў
# $source — назва крыніцы.
mat-online-loading = Загружаецца каталог { $source }…
mat-online-load-failed = Не ўдаецца загрузіць каталог: { $error }
# $n — колькасць паказаных матэрыялаў.
mat-online-count = { $n ->
        [one] { $n } матэрыял
        [few] { $n } матэрыялы
        [many] { $n } матэрыялаў
       *[other] { $n } матэрыялу
    }
mat-online-load-hover = Запытаць у ўбудовы яе матэрыялы з гэтымі значэннямі
mat-online-downloaded = { $name } спампаваны ў бібліятэку матэрыялаў
mat-terms-polyhaven = Працуе на Poly Haven (polyhaven.com). Тэкстуры пад CC0: бясплатна для любога выкарыстання, у тым ліку камерцыйнага; указваць аўтара не трэба.
mat-terms-ambientcg = Матэрыялы ambientCG пад CC0: бясплатна для любога выкарыстання, у тым ліку камерцыйнага; указанне аўтара неабавязковае.
mat-err-not-catalogue = гэта не каталог
mat-err-no-colour-map = няма карты колеру

## Рэдактар матэрыялу
mated-window-title = Рэдактар матэрыялу
mated-none = Няма матэрыялу: выберыце ўзор або матэрыял з бібліятэкі ці вазьміце яго з вылучэння.
mated-not-in-model = Яшчэ не ў мадэлі: дадасца, калі вы пафарбуеце ім або зменіце яго.
mated-used = { $count ->
        [one] Выкарыстоўваецца на { $count } грані або аб'екце
       *[other] Выкарыстоўваецца на { $count } гранях і аб'ектах
    }
mated-pick = Узяць з вылучэння
mated-pick-tip = Піпетка: зрабіць бягучым матэрыял вылучанай грані або аб'екта
mated-pick-none = Вылучыце пафарбаваную грань або аб'ект, каб узяць яго матэрыял
mated-picked = Бягучы матэрыял: { $name }
mated-select-tip = Вылучыць грані і аб'екты гэтага кантэксту, пафарбаваныя гэтым матэрыялам
mated-selected = { $count ->
        [one] Вылучаны { $count } элемент
        [few] Вылучана { $count } элементы
        [many] Вылучана { $count } элементаў
       *[other] Вылучана { $count } элемента
    }
mated-window-tip = Адкрыць рэдактар у акне
mated-basic = Асноўнае
mated-color = Колер
mated-opacity = Непразрыстасць
mated-texture = Тэкстура
mated-no-texture = Без тэкстуры
mated-look = Выгляд
mated-hue = Адценне
mated-saturation = Насычанасць
mated-lightness = Яркасць
mated-cutoff = Парог празрыстасці
mated-colorize = Таніраваць
mated-colorize-tip = Афарбаваць выяву колерам матэрыялу, захаваўшы яе яркасць
mated-decal = Налепка
mated-decal-tip = Паказаць адну плітку без паўтораў: вакол яе відаць колер матэрыялу
mated-world = Сусветныя каардынаты
mated-world-tip = Каардынаты ад становішча ў свеце (кубічная праекцыя): без уліку пазіцыі тэкстуры, узор працягваецца праз грані
mated-look-reset = Скінуць выгляд
mated-undo-look = Выгляд тэкстуры
mated-undo-color = Змяніць колер матэрыялу
mated-textures = Тэкстуры
mated-slot-base = Асноўная тэкстура
mated-slot-empty = Пуста
mated-view-tip = Адкрыць буйна
mated-downscale = Паменшыць
mated-downscale-tip = Вялікая выява: рэндэр выкарыстоўвае не больш за 1024 px. Перазахаваць у { $side } px, каб дакумент быў меншы
mated-load-tip = Загрузіць выяву…
mated-replace-tip = Замяніць іншай выявай… (памер пліткі захаваецца)
mated-clear-tip = Прыбраць гэту тэкстуру
mated-export-tip = Захаваць выяву ў файл…
mated-edit-tip = Рэдагаваць у графічным рэдактары па змаўчанні для файла: захаванне там абновіць матэрыял
mated-map-set = Загрузіць набор карт з папкі…
mated-map-set-tip = Запоўніць слоты з папкі па суфіксах імёнаў файлаў: _color / _albedo / _diff, _normal (лепей OpenGL), _rough, _metal, _ao, _emission
mated-watching = { $count ->
        [one] Адсочваецца { $count } тэкстура, адкрытая ў рэдактары
        [few] Адсочваюцца { $count } тэкстуры, адкрытыя ў рэдактары
        [many] Адсочваецца { $count } тэкстур, адкрытых у рэдактары
       *[other] Адсочваецца { $count } тэкстуры, адкрытай у рэдактары
    }
mated-view-title = { $material }: { $slot }
mated-view-tiled = Паўтарыць 3 × 3
mated-undo-texture = Загрузіць тэкстуру
mated-undo-map-set = Загрузіць набор карт
mated-undo-clear = Прыбраць тэкстуру
mated-undo-downscale = Паменшыць тэкстуру
mated-undo-reload = Перазагрузіць тэкстуру
mated-no-maps = У { $path } няма выяў з суфіксамі карт
mated-maps-loaded = { $count ->
        [one] Загружана { $count } карта
        [few] Загружаны { $count } карты
        [many] Загружана { $count } карт
       *[other] Загружана { $count } карты
    }
mated-exported = Захавана: { $path }
mated-editing = Рэдагуецца { $file }: захавайце яго, каб абнавіць матэрыял
mated-cant-open-editor = Не ўдалося адкрыць графічны рэдактар: { $error }
mated-reloaded = { $slot }: перазагружана з графічнага рэдактара

## ======== Анлайн-бібліятэка 3D ========

model-lib-licence-any = Любая ліцэнзія
model-lib-licence-commercial = Камерцыйнае выкарыстанне дазволена
model-lib-licence-cc0 = Толькі CC0
model-lib-licence-known = Вядомая ліцэнзія

## Акно і фільтры

model-lib-title = Анлайн-бібліятэка 3D
model-lib-source = Крыніца
model-lib-source-count = { $label } ({ $n })
model-lib-plugins-hint =
    Убудовы з @model_source
    паказаны тут
model-lib-search-enter = Пошук, потым Enter: крэсла, чэрап…
model-lib-search = Пошук: крэсла, камень, расліна…
model-lib-all-categories = Усе катэгорыі
# $description — апісанне крыніцы.
model-lib-downloads-go = { $description } Загрузкі ідуць у бібліятэку кампанентаў.
model-lib-from-url = Па URL
model-lib-import = Імпартаваць
model-lib-import-hover = Спампаваць файл у бібліятэку кампанентаў і размясціць
model-lib-click-hint = Пстрыкніце мадэль, каб паглядзець; двайны пстрык — спампаваць і размясціць.
model-lib-type-search = Увядзіце запыт і націсніце Enter.
model-lib-loading = Загружаюцца мадэлі { $source }…
model-lib-load-failed = Не ўдаецца загрузіць мадэлі: { $error }
model-lib-retry = Паўтарыць
# $n — колькасць паказаных мадэляў.
model-lib-count = { $n ->
        [one] { $n } мадэль
        [few] { $n } мадэлі
        [many] { $n } мадэляў
       *[other] { $n } мадэлі
    }
model-lib-show = Паказаць
model-lib-format-any = Любы фармат
model-lib-size-any = Любы памер
model-lib-size-10k = ≤ 10 тыс. трохкутнікаў
model-lib-size-50k = ≤ 50 тыс. трохкутнікаў
model-lib-size-100k = ≤ 100 тыс. трохкутнікаў
model-lib-clear = Ачысціць
model-lib-reload = Перазагрузіць
model-lib-load = Загрузіць
model-lib-choose-with = выберыце праз …
model-lib-choose-it = Выберыце
model-lib-plugin-apply-hover = Ужыць гэтыя значэнні (потым шукаць)
model-lib-plugin-load-hover = Запытаць у ўбудовы яе мадэлі з гэтымі значэннямі

## Падрабязнасці мадэлі

# $n — колькасць трохкутнікаў.
model-lib-triangles = { $n ->
        [one] { $n } трохкутнік
        [few] { $n } трохкутнікі
        [many] { $n } трохкутнікаў
       *[other] { $n } трохкутніка
    }
# $value — памер у мегабайтах.
model-lib-megabytes = { $value } МБ
model-lib-heavy = ⚠ Цяжкая мадэль: яна можа запавольваць выгляд.
model-lib-downloading = Спампоўваецца…
# $n — колькасць мадэляў пакета ў бібліятэцы.
model-lib-in-components = ✔ Мадэляў у «Кампанентах»: { $n }
model-lib-show-in-components = Паказаць у «Кампанентах»
model-lib-download-browser = Спампаваць у браўзеры
model-lib-download-browser-hover = Адкрыць загрузку пакета ў браўзеры; архіў імпартуецца з «Загрузак» разам з указаннем аўтарства
model-lib-download-again = Спампаваць зноў
model-lib-download-pack = Спампаваць пакет
model-lib-download-pack-hover = Спампаваць пакет: кожная мадэль стане кампанентам бібліятэкі
model-lib-place-hover = Ужо ў бібліятэцы кампанентаў: размясціць у мадэлі
model-lib-download-place = Спампаваць і размясціць
model-lib-download-place-hover = Захаваць у бібліятэку кампанентаў і размясціць у мадэлі
model-lib-download = Спампаваць
model-lib-download-hover = Захаваць у бібліятэку кампанентаў
model-lib-web-page = Вэб-старонка

## Паведамленні

# $name — мадэль.
model-lib-cancelled = Спампоўванне { $name } скасавана
model-lib-fav-added = ☆ { $name } дададзена ў абранае
model-lib-fav-removed = { $name } прыбрана з абранага
model-lib-paste-url = Устаўце адрас файла мадэлі (https://…/model.glb)
model-lib-browser-download = Спампуйце { $name } у браўзеры: Varstotas імпартуе архіў з «Загрузак»
# $n — колькасць мадэляў, $folder — папка бібліятэкі, $pack — папка пакета.
model-lib-downloaded-pack = Спампавана мадэляў: { $n } — у Кампаненты → { $folder } → { $pack }
model-lib-downloaded = Спампавана { $name } (граняў: { $faces }) — у Кампаненты → { $folder }
model-lib-not-in-library = { $path } не ў бібліятэцы кампанентаў
model-lib-download-failed = Не ўдалося спампаваць: { $error }
model-lib-import-failed = Імпартаваць { $path }: { $error }
model-lib-undo-import = Імпартаваць { $name }
model-lib-imported = Імпартавана { $path }

## Крыніцы (online_models)

model-lib-source-favourites = Абранае
model-lib-source-downloaded = Спампаванае
model-lib-terms-polyhaven = Працуе на Poly Haven (polyhaven.com). Мадэлі пад CC0: бясплатна для любога выкарыстання, у тым ліку камерцыйнага; указваць аўтара не трэба.
model-lib-terms-smithsonian = Большасць аб’ектаў пад CC0, але некаторыя пазначаны «Usage conditions apply» (толькі некамерцыйнае выкарыстанне). Перад камерцыйным выкарыстаннем праверце старонку аб’екта; таварныя знакі і іншыя правы на самі аб’екты не ахопліваюцца.
model-lib-terms-nasa = Матэрыялы NASA звычайна не абаронены аўтарскім правам, але эмблема і лагатыпы NASA на мадэлях абароненыя, і нішто не павінна ўказваць на тое, што NASA ухваляе ваш прадукт.
model-lib-terms-kenney = Пакеты Kenney пад CC0: бясплатна для любога выкарыстання, у тым ліку камерцыйнага; указанне аўтара неабавязковае. Лагатып Kenney сюды не ўваходзіць.
model-lib-terms-quaternius = Пакеты Quaternius пад CC0: бясплатна для любога выкарыстання, у тым ліку камерцыйнага; указанне аўтара неабавязковае.
model-lib-terms-saved = Кожная мадэль захоўвае ліцэнзію, з якой прыйшла (у яе падрабязнасцях; Файл → Экспарт → Атрыбуцыі). Varstotas папярэджвае, калі ліцэнзія размешчанай або экспартаванай мадэлі абмяжоўвае яе выкарыстанне.
model-lib-desc-polyhaven = Бясплатныя мадэлі CC0 (glTF, тэкстуры 1K) у рэальным памеры.
model-lib-desc-smithsonian = Smithsonian Open Access 3D: сканы музейных прадметаў. Увядзіце запыт і націсніце Enter.
model-lib-desc-nasa = NASA 3D Resources: касмічныя апараты, роверы, планеты (GLB).
model-lib-desc-kenney = Пакеты гульнёвых рэсурсаў Kenney (CC0): пакет спампоўваецца цалкам, кожная мадэль становіцца кампанентам у Кампаненты → Kenney → <пакет>.
model-lib-desc-quaternius = Пакеты Quaternius з нізкай колькасцю палігонаў (CC0). Яны спампоўваюцца ў вашым браўзеры (Google Drive): Varstotas адкрывае старонку і імпартуе архіў з «Загрузак».
model-lib-desc-favourites = Мадэлі, якія вы пазначылі ☆ (пстрыкніце ☆ у падрабязнасцях мадэлі).
model-lib-desc-history = Усё, што спампавана або імпартавана з «Загрузак», спачатку новае. Пішыце, каб шукаць.

## Памылкі каталогаў і імпарту

model-lib-err-not-catalogue = гэта не каталог
model-lib-err-not-search = гэта не адказ на пошук
model-lib-err-kenney-packs = на kenney.nl пакетаў не знойдзена (старонка змянілася?)
model-lib-err-quaternius-packs = на quaternius.com пакетаў не знойдзена (старонка змянілася?)
model-lib-err-plugin-array = убудова павінна вярнуць масіў мадэляў
model-lib-err-no-download = на старонцы пакета няма загрузкі
model-lib-err-quaternius-browser = Пакеты Quaternius спампоўваюцца ў браўзеры: выкарыстайце «Спампаваць у браўзеры»
model-lib-err-importer-depth = убудовы-імпартары выклікаюць адна адну занадта глыбока
model-lib-err-empty-answer = пусты адказ
model-lib-err-plugin-nothing = убудова нічога не вярнула
model-lib-err-spec-object = апісанне мадэлі павінна быць аб’ектам
model-lib-err-spec-empty = у апісанні мадэлі няма file, url, zip або meshes
model-lib-err-archive-no-model = у архіве няма файла мадэлі (glb, gltf, fbx, dae, obj, 3mf, ply, stl, dxf)
# $k — нумар сеткі ў адказе.
model-lib-err-mesh-vertices = сетка { $k }: няма вяршынь
model-lib-err-mesh-faces = сетка { $k }: няма граняў
model-lib-err-no-faces = у сетках няма граняў
model-lib-err-model-empty = мадэль пустая
model-lib-err-no-size = у мадэлі няма памеру
model-lib-err-no-home = няма хатняй папкі
model-lib-err-archive-none = у архіве няма файлаў мадэляў
model-lib-err-none-read = не ўдалося прачытаць ніводнай мадэлі

## ======== Загрузкі ========

## Паведамленні
# $path is a file.
downloads-no-models = { $path }: у ім няма файлаў мадэляў
downloads-no-address = { $path }: у ім няма вэб-адраса
downloads-not-model = { $path }: не файл мадэлі (glb, gltf, fbx, dae, obj, 3mf, ply, stl, dxf, varstotas, zip)
# $site is the site, $title the model.
downloads-waiting-site = Чакаем файл з { $site }: спампуйце яго ў браўзеры
downloads-waiting-file = Чакаем файл «{ $title }» ({ $site }): спампуйце яго ў браўзеры
downloads-no-library = Няма папкі бібліятэкі кампанентаў
# $name is the file, $folder the library folder, $n the number of models.
downloads-imported = Імпартавана { $name } у Кампаненты → { $folder }
downloads-imported-many = Імпартавана мадэляў: { $n } з { $name } у Кампаненты → { $folder }
downloads-import-failed = Імпарт { $name }: { $error }
downloads-err-no-file-name = няма імя файла
downloads-err-no-model-chosen = мадэль не выбрана
# $label is a licence, $names the first models under it, $more " +N" or empty, $note what it asks.
downloads-licence-part = { $label } ({ $names }{ $more }) — { $note }
# $list is the licence-part texts.
downloads-licences = Ліцэнзіі: { $list }. Спіс аўтараў: Файл → Экспарт → Атрыбуцыі.

## Пошук на, Іншыя сайты

downloads-search-on = Шукаць на:
# $site is the site, $query the search.
downloads-search-hover = Адкрыць у браўзеры пошук на { $site } па запыце «{ $query }»
downloads-edit = Змяніць…
downloads-edit-hover = Змяніць сайты ({ DATA_DIR() }/sources.toml); спіс будзе прачытаны нанова
downloads-other = Іншыя сайты
# $waiting is the number of waiting downloads.
downloads-other-waiting = Іншыя сайты — чакаем загрузак: { $waiting }
downloads-other-text = Спампуйце з любога сайта ў браўзеры: Varstotas сам на сайты не заходзіць. Ён бярэ файл з вашай папкі Загрузак і захоўвае старонку як крыніцу.
downloads-watch = Сачыць за Загрузкамі
downloads-watch-hover = Прапаноўваць да імпарту кожны новы файл мадэлі або архіў з папкі Загрузак (заўсёды ўключана, пакуль старонка чакае)
# $dir is the watched folder.
downloads-watching = сочым за { $dir }
downloads-off = выключана
downloads-import-file = Імпартаваць файл…
downloads-import-file-hover = Імпартаваць файл мадэлі або архіў, які вы спампавалі
downloads-models-filter = Мадэлі
# $title is the model, $site the site, $licence its licence.
downloads-waiting-card = Чакаем файл з гэтай старонкі: { $title } ({ $site }, { $licence })
downloads-page = Старонка
# $file is a file address.
downloads-browser-file = Спампуйце ў браўзеры: { $file }
downloads-stop = Не чакаць

## Акно Імпарт загрузкі

downloads-import-title = Імпарт загрузкі
downloads-importing = Імпартуецца…
# $name is the file, $mb its size in megabytes.
downloads-file-size = { $name }  ({ $mb } МБ)
downloads-credit-from-page = ✔ Крыніца ўзята са старонкі, якую вы адправілі
downloads-duplicate = ⚠ Гэты ж файл ужо імпартаваўся раней
downloads-field-title = Назва
downloads-field-author = Аўтар
downloads-field-site = Сайт
downloads-licence = Ліцэнзія
downloads-licence-hint = як на старонцы
downloads-place = Змясціць у мадэль
# $on models are chosen of $n.
downloads-models-count = { $on } з { $n } мадэляў (кожная стане кампанентам)
downloads-all = Усе
downloads-none = Нічога
# $folder is the library folder of the site.
downloads-destination = Кампаненты → { $folder }
downloads-ignore = Прапусціць
# $n is the number of other offers.
downloads-more-waiting = Чакае яшчэ: { $n }

## ======== Рэндэр ========

render-dof-view = Як у выглядзе
render-dof-off = Выкл.
render-dof-on = Укл.

## Прасеты якасці і памеры відарысаў (ідэнтыфікатары застаюцца англійскімі)

render-preset-draft = Чарнавік
render-preset-preview = Прагляд
render-preset-final = Фінальная
render-preset-best = Найлепшая
render-size-hd = HD 1280 × 720
render-size-full-hd = Full HD 1920 × 1080
render-size-qhd = QHD 2560 × 1440
render-size-4k = 4K 3840 × 2160
render-size-square = Квадрат 1080
render-size-portrait = Партрэт 1080 × 1350

## Месяцы (дата сонца)

render-month-jan = студз.
render-month-feb = лют.
render-month-mar = сак.
render-month-apr = крас.
render-month-may = трав.
render-month-jun = чэрв.
render-month-jul = ліп.
render-month-aug = жн.
render-month-sep = вер.
render-month-oct = кастр.
render-month-nov = ліст.
render-month-dec = снеж.

## Памылкі экспарту і шэйдараў

render-export-empty = памер відарыса павінен быць не менш за 1 x 1 піксель
# $w and $h are the requested size, $max the limit per side.
render-export-too-large = памер відарыса { $w } x { $h } занадта вялікі (не больш за { $max } пікселяў на бок)
render-export-gpu = Памылка GPU пры экспарце відарыса: { $error }
render-shader-clash = не кампілюецца разам з іншымі шэйдарамі: { $error }

## Акно рэндэра

render-title = Рэндэр
render-current-view = Адрэндэрыць бягучы выгляд
render-stop = Спыніць
render-continue = Працягнуць
render-more-samples = Больш сэмплаў
render-more-samples-tip = Падвоіць колькасць сэмплаў
render-save-png = Захаваць PNG…
render-save-exr = Захаваць EXR…
render-save-exr-tip = Лінейнае святло, 32-бітныя лікі з плаваючай кропкай
render-all-scenes = Адрэндэрыць усе сцэны…
# $error is the reason.
render-failed = Рэндэр не ўдаўся: { $error }
render-error = Рэндэр: { $error }
# $n is the number of scenes, $folder where they were saved.
render-scenes-done = Адрэндэрана сцэн: { $n }, у { $folder }
# $written scenes are saved, scene $next is being rendered.
render-scenes-progress = Рэндэр сцэн: захавана { $written }, у працы сцэна { $next }
# $path is the saved file.
render-saved = Захавана { $path }
# $secs is a number with one decimal.
render-status-running = { $samples } / { $max } сэмплаў · { $secs } с
# $denoised is "yes" or "no".
render-status-done = Гатова: { $samples } сэмплаў за { $secs } с{ $denoised ->
        [yes] , з шумапрыглушэннем
       *[no] {""}
    }
render-status-stopped = Спынена на { $samples } сэмплах · { $secs } с
render-undo-lights = Праўка крыніц святла
render-undo-settings = Налады рэндэра

## Святло

render-lights = Святло ({ $n })
render-add-point = + Кропка
render-add-spot = + Пражэктар
render-add-area = + Плоскасць
render-on = Укл.
render-light-point = кропка
render-light-spot = пражэктар
render-light-area = плоскасць
render-light-name = Імя
render-light-type = Тып
render-light-position = Пазіцыя
render-light-to-centre = У цэнтр выгляду
render-light-to-centre-tip = Перамясціць на 2 м над мэтай камеры
render-light-to-camera = Да камеры
render-light-aim-at-target = Накіраваць на мэту
render-light-aim-at-target-tip = Свяціць на мэту выгляду
render-light-aim = Накіраванне
render-colour = Колер
render-light-power = Моц
render-light-radius = Радыус
render-light-radius-tip = Памер лямпы: больш мяккія цені
render-light-cone = Конус
render-light-cone-angle = Вугал конуса
render-light-soft = Мяккасць
render-light-soft-edge = Мяккі край
render-size = Памер

## Налады

render-sec-image = Відарыс
render-sec-sampling = Сэмплаванне
render-sec-camera = Камера
render-sec-light = Святло
render-sec-colour = Колер
render-quality = Якасць
# $samples and $bounces are numbers.
render-quality-tip = сэмплаў: { $samples }, адбіццяў: { $bounces }
render-match-view = Як выгляд
render-match-view-tip = Прапорцыі выгляду пры гэтай вышыні
render-size-custom = Свой
render-transparent = Празрысты фон
render-transparent-tip = Неба і зямля за мадэллю празрыстыя ў PNG (але працягваюць яе асвятляць)
render-samples = Сэмплы
render-time-limit = Ліміт часу
render-time-limit-tip = 0 = пакуль не скончацца сэмплы
render-bounces = Адбіцці
render-clamp = Абмежаванне
render-clamp-tip = Найярчэйшае ўскоснае святло сэмпла: ніжэй — менш светлавых шумоў, вышэй — захоўваецца яркае адбітае святло; 0 = выкл.
render-denoiser = Шумапрыглушэнне
render-denoiser-off = Выкл.
render-denoiser-builtin = Убудаванае
render-denoiser-builtin-tip = Фільтр з улікам краёў
render-denoiser-ai = ШІ (Open Image Denoise)
render-denoiser-install = Усталюйце Open Image Denoise: brew install open-image-denoise
render-dof = Глыбіня рэзкасці
render-dof-tip = Як у выглядзе: глыбіня рэзкасці з Эфектаў, калі яна ўключана
# $mm is the equivalent focal length.
render-lens = Аб’ектыў { $mm } мм
render-lens-tip = Поле зроку выгляду для камеры 35 мм
render-aperture = Дыяфрагма
render-aperture-tip = Меншая дыяфрагмавая лічба = меншая глыбіня рэзкасці
render-focus = Фокус
render-focus-target = Мэта
render-focus-target-tip = Факусіравацца на мэце выгляду (цэнтр арбіты)
render-sun = Сонца
render-sun-tip = Моц сонца (яго кірунак: Цені)
render-sky = Неба
render-sky-tip = Святло асяроддзя
render-rotation = Паварот
render-rotation-tip = Паварочвае неба / HDRI
render-exposure = Экспазіцыя
render-env-exposure-tip = Экспазіцыя асяроддзя (выгляд і рэндэр)
render-exposure-tip = Дадаецца да экспазіцыі асяроддзя (толькі рэндэр)
render-tone-curve = Танальная крывая
# $name is the environment's tone curve.
render-tone-environment = Асяроддзе: { $name }
render-white-balance = Баланс белага
render-white-balance-tip = 6500 K = нейтральны; вышэй — цяплей
render-tint = Адценне
render-tint-tip = Зялёны (−) .. пурпурны (+)
render-saturation = Насычанасць
render-contrast = Кантраст
render-vignette = Віньетка
render-reset = Скінуць
render-reset-tip = Вярнуць налады колеру па змаўчанні
render-shader-define-surface = define fn surface(s: Surface, p: Params) -> Surface
# $line is the line in the user's shader code, $msg the compiler's message.
render-shader-line = радок { $line }: { $msg }

## ======== Рэалістычны стыль ========

realistic-undo-material = Праўка матэрыялу
# $path is a file, $error the reason.
realistic-cant-use = Немагчыма выкарыстаць { $path }: { $error }
realistic-cant-read = Немагчыма прачытаць { $path }: { $error }
realistic-used-by = Выкарыстоўваецца Рэалістычным стылем
realistic-load = Загрузіць…
realistic-clear = Ачысціць
realistic-reset = Скінуць
realistic-preset = Прасет

## Асяроддзе

realistic-background = Фон
realistic-sky-sky = Неба
realistic-sky-color = Колер
realistic-sky-hdri = HDRI
realistic-sky-physical = Фізічнае неба
realistic-sky-physical-tip = Дзённае святло для становішча сонца з карткі Цені
realistic-haze = Смуга
realistic-haze-tip = Мутнасць: 2 = чыстае блакітнае неба, 10 = смуга; час і дата сонца — у картцы Цені
realistic-hdri-none = няма (неба)
realistic-hdri-tip = Эквірэктангулярная панарама (.hdr)
realistic-rotation = Паварот
realistic-intensity = Інтэнсіўнасць
realistic-sun = Сонца
realistic-sun-tip = Моц сонца; яго кірунак бярэцца з карткі Цені, калі цені ўключаны
realistic-exposure = Экспазіцыя
realistic-tone-mapping = Танальнае адлюстраванне
realistic-tonemap-aces = ACES (кінематаграфічнае)
realistic-tonemap-agx = AgX (мяккае)
realistic-tonemap-none = Няма
realistic-show-env = Паказваць асяроддзе за мадэллю
realistic-effects = Эфекты

## Фізічныя ўласцівасці

realistic-physical = Фізічныя (Рэалістычны)
realistic-preset-matte = Матавы
realistic-preset-satin = Сатынавы
realistic-preset-glossy = Глянцавы
realistic-preset-metal = Метал
realistic-preset-chrome = Хром
realistic-preset-glass = Шкло
realistic-preset-light = Святло
realistic-metallic = Металічнасць
realistic-metallic-tip = 0 = фарба, пластык, дрэва; 1 = голы метал
realistic-roughness = Шурпатасць
realistic-roughness-tip = 0 = люстэркавая, 1 = матавая
realistic-glass = Шкло
realistic-glass-tip = Колькі святла прапускаецца
realistic-ior = IOR
realistic-ior-tip = Паказчык пераламлення (шкло 1,5, вада 1,33)
realistic-emission = Выпраменьванне
realistic-emission-tip = Святло, якое выдае паверхня: колер і моц
realistic-coat-sheen = Пакрыццё, бляск, краі
realistic-clearcoat = Празрыстае пакрыццё
realistic-clearcoat-tip = Глянцавы слой лаку: лакіраванае дрэва, аўтамабільная фарба, глазураваная плітка
realistic-coat-rough = Шурпатасць пакрыцця
realistic-coat-rough-tip = Шурпатасць лаку (0 = глянцавы)
realistic-sheen = Бляск тканіны
realistic-sheen-tip = Мяккае свячэнне пад пакатым вуглом: аксаміт, тканіна, абіўка
realistic-sheen-tint = Адценне бляску
realistic-sheen-tint-tip = 0 = белы бляск, 1 = колер матэрыялу
realistic-anisotropy = Анізатропія
realistic-anisotropy-tip = Выцягнутыя блікі ўздоўж тэкстуры (+) або поперак яе (−): шліфаваны метал
realistic-anti-tiling = Супраць мазаічнасці
realistic-anti-tiling-tip = Разбівае бачнае паўтарэнне тэкстур
realistic-bevel = Фаска, мм
realistic-bevel-tip = Закругляе рэбры граней у рэндэры (толькі трасіроўка шляху)
realistic-shader = Шэйдар
realistic-shader-tip = Праграма паверхні на WGSL (docs/SHADERS.md): працэдурныя ўзоры, хвалі, свячэнне
realistic-shader-none = Няма
realistic-shader-custom = Свой
realistic-shader-edit = Змяніць…
realistic-shader-edit-tip = Напісаць або змяніць шэйдар
realistic-map-normal = Нармалі
realistic-map-roughness = Шурпатасць
realistic-map-metallic = Металічнасць
realistic-map-occlusion = Аклюзія
realistic-map-emission = Выпраменьванне
realistic-normal-strength = Моц нармаляў
realistic-remove-physical = Выдаліць фізічныя ўласцівасці

## Рэдактар шэйдараў

# $material is the material's name.
realistic-shader-window = Шэйдар — { $material }
realistic-start-from = Пачаць з…
realistic-start-from-tip = Замяніць код гэтым шэйдарам
realistic-checker-new = Шахматка (новая)
realistic-reference = Даведнік
realistic-reference-tip = Палі паверхні, дапаможныя функцыі і ўбудаваныя сродкі
realistic-compiles = Кампілюецца; матэрыял яго выкарыстоўвае
realistic-ref-surface = Паверхня (s, o)
realistic-ref-parameters = Параметры
realistic-ref-functions = Функцыі

## Эфекты

realistic-fx-off = Выкл.
realistic-fx-fast = Хутка
realistic-fx-balanced = Збалансавана
realistic-fx-quality = Якасць
realistic-fx-soft-shadows = Мяккія цені
realistic-fx-soft-shadows-tip = Цені размываюцца ад таго, што іх адкідвае
realistic-fx-sun-size = Памер сонца
realistic-fx-ao = Навакольная аклюзія
realistic-fx-ao-tip = Цямнейшыя складкі і кантактныя цені
realistic-fx-radius = Радыус, мм
realistic-fx-strength = Моц
realistic-fx-reflections = Адбіткі
realistic-fx-reflections-tip = Мадэль, што адбіваецца ў глянцавых паверхнях (у прасторы экрана)
realistic-fx-bloom = Свячэнне
realistic-fx-bloom-tip = Яркае святло свеціцца
realistic-fx-threshold = Парог
realistic-fx-dof = Глыбіня рэзкасці
realistic-fx-dof-tip = Размыццё перад адлегласцю фокуса і за ёй
realistic-fx-blur = Размыццё
realistic-fx-focus = Фокус, мм
realistic-fx-focus-tip = 0 = кропка ў цэнтры выгляду
realistic-fx-fog = Туман
realistic-fx-fog-tip = Смуга, што згушчаецца з адлегласцю
realistic-fx-fog-distance = Адлегласць, мм
realistic-fx-fog-height = Вышыня, мм
realistic-fx-fog-height-tip = 0 = раўнамерны туман; інакш ён радзее на гэтай вышыні
realistic-vignette = Віньетка
realistic-style = Стыль
realistic-stylize-photo = Фота
realistic-stylize-pencil = Аловак
realistic-stylize-watercolor = Акварэль
realistic-stylize-technical = Тэхнічны
realistic-fx-clay = Гліна
realistic-fx-clay-tip = Усе матэрыялы як звычайная гліна (макет аб’ёмаў)
realistic-fx-high-quality = Высокая якасць
realistic-fx-high-quality-tip = Больш сэмплаў для мяккіх ценяў, аклюзіі і адбіткаў

## ======== Анімацыя ========

anim-title = Экспарт анімацыі
anim-no-scenes = у анімацыі няма сцэн
anim-no-scenes-flash = У анімацыі няма сцэн
# $error is the reason.
anim-error = Анімацыя: { $error }
# $what is a file path, or "N frames in <folder>".
anim-saved = Захавана { $what }
anim-render-dropped = рэндэр быў адкінуты
# $n is the frame count, $dir the folder with the PNG frames.
anim-frames-in = Кадраў: { $n }, у { $dir }
anim-cancelled = скасавана
anim-animate = Анімаваць
anim-scenes = Сцэны
anim-add-scenes = Спачатку дадайце сцэны (з адзнакай «Уключыць у анімацыю»)
anim-orbit = Аблёт вакол выгляду
anim-length = Працягласць
anim-turns-suffix = {" "}абаротаў
anim-holds-hint = Вытрымкі і пераходы: картка Сцэны → Анімацыя
anim-size = Памер
anim-fps = Частата кадраў
anim-frames = Кадры
anim-as-view = Як выгляд
anim-path-traced = Трасіроўка шляху
anim-output = Вывад
anim-gif = Аніміраваны GIF
anim-png-sequence = Паслядоўнасць PNG
# $done of $total frames.
anim-frame-progress = Кадр { $done } / { $total }
anim-encoding = Запіс GIF…
anim-export = Экспарт…

## ======== Разгортка ========

unfold-paper-fit = Па дэталях
unfold-paper-a4 = A4
unfold-paper-a3 = A3
unfold-paper-a2 = A2

## Акно разгортвання і экспарт

unfold-title = Разгортка
unfold-selection = Вылучэнне
unfold-everything = Усё ў кантэксце
unfold-nothing = Разгортка: няма чаго разгортваць (вылучыце грані або групы)
# $name is the document name, $scale like "1:10".
unfold-footer = { $name } - разгортка { $scale }
# $n is the sheet number out of $total.
unfold-footer-sheet = { $name } - разгортка { $scale } - ліст { $n } з { $total }
# $oversize is the number of pieces larger than the sheet (0 = none).
unfold-done = Разгорнута: граней — { $faces }, дэталей — { $pieces }, лістоў — { $sheets }{ $oversize ->
        [0] {""}
       *[other] {" "}(большыя за ліст: { $oversize })
    }: { $path }
unfold-export-failed = Не ўдалося экспартаваць: { $error }
# $what is "Selection" or "Everything in the context".
unfold-no-faces = { $what }: няма граней. Вылучыце грані, групы або кампаненты.
unfold-summary = { $what }: граней — { $faces }, дэталей — { $pieces }, лістоў — { $sheets }
unfold-oversize = { $n ->
        [one] { $n } дэталь большая за ліст
        [few] { $n } дэталі большыя за ліст
        [many] { $n } дэталяў большыя за ліст
       *[other] { $n } дэталі большыя за ліст
    }: абярыце большы фармат паперы або маштаб.
unfold-scale = Маштаб 1:
unfold-paper = Папера
unfold-landscape = Альбомная арыентацыя
unfold-glue-tabs = Клеевыя клапаны
unfold-edge-numbers = Нумары рэбраў
unfold-soft-folds = Згібы па мяккіх рэбрах
unfold-sheet-of = Ліст { $n } з { $total }
unfold-legend-cut = ━ разрэз
unfold-legend-valley = ╍ даліна
unfold-legend-mountain = ━·━ гара
unfold-export = Экспарт:

## ======== Рэдактар шэйдараў ========

## Убудаваныя прасеты
shader-preset-wood = Дрэва
shader-preset-water = Вада
shader-preset-reeded-glass = Рэбрыстае шкло
shader-preset-glow = Свячэнне

## Падказкі да аўтадапаўнення і выклікаў (апісанні дапаможных функцый, убудаваных сродкаў WGSL і палёў Surface)
shader-fn-surface = шэйдар: вяртае змененую паверхню
shader-fn-hash3 = псеўдавыпадковае значэнне 0..1 для кропкі
shader-fn-noise = гладкі шум значэнняў 0..1
shader-fn-fbm = шматслойны шум 0..1 (аблокі, мармур, скажэнне)
shader-fn-srgb = колер, абраны ў sRGB (0..1) → лінейны
shader-fn-hsv = адценне / насычанасць / яркасць (0..1) → лінейны колер
shader-fn-tangent_of = адзінкавы вектар упоперак n
shader-fn-bump = n, нахілены па ўхіле палявы вышыні
shader-fn-abs = абсалютнае значэнне
shader-fn-min = меншае
shader-fn-max = большае
shader-fn-clamp = x, абмежаваны ад lo да hi
shader-fn-saturate = x, абмежаваны ад 0 да 1
shader-fn-mix = a + (b − a)·t
shader-fn-step = 0 ніжэй за edge, 1 — з яго
shader-fn-smoothstep = гладкі пераход 0→1 паміж lo і hi
shader-fn-fract = x − floor(x)
shader-fn-floor = акругленне ўніз
shader-fn-ceil = акругленне ўверх
shader-fn-round = бліжэйшае цэлае
shader-fn-sign = −1, 0 або 1
shader-fn-sqrt = квадратны корань
shader-fn-pow = x у ступені y
shader-fn-exp = e у ступені x
shader-fn-log = натуральны лагарыфм
shader-fn-sin = сінус (радыяны)
shader-fn-cos = косінус (радыяны)
shader-fn-tan = тангенс (радыяны)
shader-fn-atan2 = вугал (x, y), радыяны
shader-fn-length = даўжыня вектара
shader-fn-distance = length(a − b)
shader-fn-normalize = адзінкавы вектар уздоўж v
shader-fn-dot = скалярны здабытак
shader-fn-cross = вектарны здабытак двух vec3
shader-fn-reflect = i, люстэркава адбіты ад нармалі n
shader-fn-select = t, калі cond, інакш f
shader-fn-vec2 = 2-вектар
shader-fn-vec3 = 3-вектар
shader-fn-vec4 = 4-вектар
shader-surface-base = лінейны асноўны колер
shader-surface-alpha = непразрыстасць
shader-surface-metallic = 0 = дыэлектрык, 1 = метал
shader-surface-roughness = 0 = люстэрка, 1 = матавая
shader-surface-transmission = шкло: 1 = святло прапускаецца
shader-surface-emission = святло, якое выдаецца, лінейнае, з уліком моцы
shader-surface-normal = сусветная нармаль, адзінкавая, павернутая да гледача
shader-surface-pos = пазіцыя ў мадэлі, мм (z = уверх)
shader-surface-uv = плоскія тэкстурныя каардынаты, мм
shader-surface-view = адзінкавы кірунак да вока
shader-surface-time = секунды
shader-param-tag = паўзунок рэдактара матэрыялу, чытаецца як p.name (не больш за 8)

## ======== Дыялогі ========

## Аднаўленне аўтазахавання
dialog-untitled = Без назвы
dialog-age-recently = нядаўна
dialog-age-under-minute = менш за хвіліну таму
# $n is a number
dialog-age-minutes = { $n ->
        [one] { $n } хвіліну таму
        [few] { $n } хвіліны таму
        [many] { $n } хвілін таму
       *[other] { $n } хвіліны таму
    }
dialog-age-hours = { $n ->
        [one] { $n } гадзіну таму
        [few] { $n } гадзіны таму
        [many] { $n } гадзін таму
       *[other] { $n } гадзіны таму
    }
dialog-age-days = { $n ->
        [one] { $n } дзень таму
        [few] { $n } дні таму
        [many] { $n } дзён таму
       *[other] { $n } дня таму
    }

## Захаванне змен
dialog-save-title = Захаваць змены?
# $name is the document name
dialog-save-question = Захаваць змены, якія вы ўнеслі ў «{ $name }»?
dialog-save-warning = Калі не захаваць, змены будуць страчаны.
dialog-dont-save = Не захоўваць
dialog-save = Захаваць

## Аднаўленне аўтазахаванай мадэлі
dialog-recover-title = Аднавіць аўтазахаваную мадэль?
# $name is a file name; $age is dialog-age-* text ("5 minutes ago")
dialog-recover-copy = Varstotas знайшоў аўтазахаваную копію «{ $name }» ад { $age }, навейшую за захаваны файл.
dialog-recover-untitled = Varstotas знайшоў аўтазахаваную мадэль без назвы ад { $age }.
dialog-recover-note = Яна можа ўтрымліваць змены, якія не былі захаваны, напрыклад пасля збою.
dialog-discard = Адкінуць
dialog-discard-hint = Выдаліць аўтазахаваную копію
dialog-recover = Аднавіць
dialog-not-now = Не зараз
dialog-not-now-hint = Пакінуць аўтазахаваную копію на потым

## Змяненне памеру
dialog-resize-title = Змяніць памер?
dialog-resize-model = Змяніць памер мадэлі?
dialog-resize-group = Змяніць памер актыўнай групы або кампанента?
# $factor is the scale factor, e.g. 1.2500
dialog-resize-scaled = Усё ў ёй будзе пераменштабавана ў { $factor } раза адносна яе пачатку каардынат.
dialog-resize-keys = Enter = Так · Esc = Не
dialog-yes = Так
dialog-no = Не

## Запыт значэння
dialog-invalid-value = Няправільнае значэнне

## Агульныя подпісы
dialog-width = Шырыня
dialog-height = Вышыня

## Экспарт 2D-графікі
dialog-export-title = Экспарт 2D-графікі
dialog-export-aspect = Выкарыстоўваць суадносіны старон выгляду
dialog-export-viewport = Памер выглядавага акна
# $w and $h are pixel counts
dialog-export-viewport-hint = { $w } × { $h } пікс. (экранных пікселяў)
dialog-export-double = Падвоіць памер відарыса
dialog-export-transparent = Празрысты фон
dialog-export-axes = Восі
dialog-export-guides = Накіроўвальныя
dialog-export-dims = Памеры
dialog-export-note = Вылучэнне і падгляд інструментаў ніколі не экспартуюцца.
dialog-export = Экспарт…
dialog-filter-png = Відарыс PNG

## Экспарт 2D-чарцяжа
dialog-drawing-title = Экспарт 2D-чарцяжа
dialog-drawing-intro = Бягучы выгляд лініямі: SVG або PDF.
dialog-drawing-scale = У маштабе 1:
dialog-drawing-perspective = Перспектыўны выгляд: упісваецца ў старонку A4. Для маштабу — Камера → Паралельная праекцыя.
dialog-drawing-fitted = Упісана ў старонку A4.
dialog-drawing-profiles = Профілі (тоўстыя контуры)
dialog-drawing-hidden = Схаваныя лініі (пункцірныя)
dialog-drawing-dims = Памеры і тэкст
dialog-drawing-sections = Раз’ём
dialog-drawing-pdf = PDF…
dialog-drawing-svg = SVG…

## Размяшчэнне 3D-тэксту
dialog-text3d-title = Размясціць 3D-тэкст
dialog-text3d-font = Шрыфт
dialog-text3d-filled = Запоўнены
dialog-text3d-extruded = Выцягнуты
dialog-text3d-hint = Enter пачынае новы радок, %CMD%+Enter размяшчае тэкст.
dialog-text3d-place = Размясціць

## Выбар шаблона
dialog-template-title = Выберыце шаблон
dialog-template-intro = Новыя мадэлі пачынаюцца з гэтымі адзінкамі. Іх можна змяніць па змаўчанні ў Наладах.

## Пра праграму
dialog-about-tagline = Лёгкі натыўны 3D-рэдактар для малявання і пабудовы.
# $version is the app version
dialog-about-version = Версія { $version }
app-start-failed-title = Не ўдалося запусціць Varstotas
app-start-failed = Varstotas не змог адкрыць акно:

    { $error }

    Для 3D-выгляду патрэбны Vulkan, DirectX 12, Metal або OpenGL 3.3. Абнавіце драйвер відэакарты; у віртуальнай машыне ўключыце 3D-паскарэнне.
dialog-about-build = Зборка { $build } · { $commit }
dialog-about-dev-build = Зборка для распрацоўкі · { $commit }
dialog-about-licence = Свабоднае праграмнае забеспячэнне пад ліцэнзіяй MIT.
dialog-about-licences = Ліцэнзіі…

## ======== Лісты ========

sheets-title = Лісты
sheets-custom = Свой
sheets-none-yet = Лістоў пакуль няма.
sheets-none-first = Няма ліста: спачатку дадайце (Акно → Лісты)
sheets-nothing-to-export = Лісты: няма чаго экспартаваць (спачатку дадайце ліст)
# $i is the sheet's number, $n the number of sheets
sheets-exported-one = Экспартаваны ліст { $i } з { $n }: { $path }
sheets-exported-all = Экспартавана лістоў: { $n }: { $path }
sheets-export-failed = Не ўдалося экспартаваць: { $error }
sheets-add-first = Дадайце ліст, каб пачаць

## Крокі адмены
sheets-undo-add = Дадаць ліст
sheets-undo-edit = Правіць ліст
sheets-undo-duplicate = Дубліраваць ліст
sheets-undo-delete = Выдаліць ліст

## Спіс лістоў і ўласцівасці
sheets-add = + Ліст
sheets-duplicate = Дубліраваць
sheets-name = Імя
sheets-paper = Папера
sheets-landscape = Альбомная арыентацыя
sheets-title-block = Штамп
sheets-tb-title = Назва
sheets-tb-author = Аўтар
sheets-tb-date = Дата

## Даданне элементаў
sheets-add-viewport = + Выгляд
sheets-add-viewport-hint = Бягучы выгляд мадэлі
sheets-add-text = + Тэкст
sheets-add-credits = + Атрыбуцыі
sheets-add-credits-hint = Атрыбуцыі спампаваных мадэляў (назва, аўтар, сайт, ліцэнзія): тэкст з {"{credits}"}, запаўняецца пры малюнку
sheets-remove = Прыбраць
sheets-add-views = + Стандартныя выгляды
sheets-add-views-hint = Спераду, зверху і злева ў стандартным маштабе з габарытнымі памерамі, а таксама ізаметрыя
sheets-add-table = + Табліца дэталей
sheets-add-table-hint = Групы і кампаненты з памерамі

## Выбраны элемент
# $n is the item's number
sheets-viewport-n = Выгляд { $n }
sheets-text-n = Тэкст { $n }
sheets-table-n = Табліца дэталей { $n }
sheets-shows = Паказвае
sheets-captured = Захоплены выгляд
sheets-update-view = Абнавіць да бягучага выгляду
sheets-scale = Маштаб
sheets-scale-hint = Для маштабу патрэбна Паралельная праекцыя
sheets-position = Пазіцыя
sheets-size = Памер
sheets-height = Вышыня
sheets-text-height = Вышыня тэксту
sheets-profiles = Профілі
sheets-hidden = Схаваныя лініі
sheets-dims = Памеры
sheets-sections = Разрэзы
sheets-frame = Рамка
sheets-caption = Подпіс
sheets-overall = Габарытныя памеры
sheets-hint = Пстрыкніце элемент у падглядзе, каб правіць яго; перацягвайце, каб перамясціць, цягніце за кут, каб змяніць памер.

## Ніжні калантытул
# $paper is a paper name; the others are counts
sheets-summary = { $paper } — выглядаў: { $viewports }, у маштабе: { $scaled }, тэкстаў: { $texts }
sheets-svg-hint = Гэты ліст
sheets-pdf-hint = Усе лісты, па старонцы на кожны
sheets-export = Экспарт:

## ======== Параметры кампанента ========

rig-title = Параметры кампанента
rig-select-hint = Вылучыце кампанент (не групу) і абярыце Акно → Параметры кампанента або Правіць параметры… ва Уласцівасцях.
rig-state-has = — з параметрамі
rig-state-none = — параметраў пакуль няма: іх дадасць Ужыць
rig-no-parts = У кампанента няма частак для руху: згрупуйце яго дэталі ўсярэдзіне (дзверы, шуфляду…) — адкрыйце яго і выкарыстайце Стварыць групу.

## Блок «Уласцівасці»
# $name is the component's name; $n the number of copies
rig-info-copies = { $name } — копій: { $n }
rig-no-params = Параметраў пакуль няма
rig-reset = Скінуць
rig-reset-hint = Вярнуць значэнні па змаўчанні
rig-edit = Правіць параметры…
rig-edit-hint = Акно → Параметры кампанента: параметры, формулы, рухі частак, скрыпт
# $n is the number of motions
rig-info-summary = Кампанент з параметрамі (рухаў: { $n })

## Крокі адмены і паведамленні
rig-undo-reset = Скінуць параметры
rig-undo-edit = Правіць параметры
rig-undo-add = Дадаць параметры
rig-undo-remove = Выдаліць параметры
rig-select-rigged = Вылучыце кампаненты з параметрамі
rig-select-component = Вылучыце кампанент, каб даць яму параметры
rig-err-group = Групы не могуць мець параметраў: спачатку стварыце з яе кампанент (Праўка → Стварыць кампанент)
rig-err-generated = Гэты кампанент пабудаваны генератарам або параметрычнай групай
rig-applied = Параметры ўжыты
rig-removed = Параметры выдалены
rig-err-place-copy = Размесціце копію кампанента, каб выбраць у мадэлі
rig-pick-axis = Пстрыкніце дзве кропкі на восі павароту
rig-pick-slide = Пстрыкніце дзве кропкі: кірунак слізгання
rig-err-same-points = Дзве кропкі супадаюць

## Табліца параметраў
rig-parameters = Параметры
rig-parameters-hint = Значэнні, якія мае кожная копія (іх паказвае Уласцівасці)
rig-col-name = Імя
rig-col-label = Подпіс
rig-col-type = Тып
rig-col-min = Мін.
rig-col-max = Макс.
rig-col-step = Крок
rig-col-default = Па змаўчанні
rig-col-options = Варыянты
rig-as-drawn = (як намалявана)
rig-remove-param = Выдаліць параметр
rig-add-param = + Параметр

## Формулы
rig-formulas = Формулы
rig-formulas-hint = Значэнні, вылічаныя з параметраў (выразы Rhai), якія можна выкарыстоўваць у рухах і скрыпце
rig-add-formula = + Формула

## Рухі
rig-motions = Рухі
rig-motions-hint = Частка (група / кампанент непасрэдна ўнутры), якую параметр паварочвае, зрушвае, паказвае, фарбуе або якой задае значэнне параметра (Задаць параметр: частка, што сама мае параметры — створаны генератарам кампанент, напрыклад цыліндр з Маляваць → Фігуры, або кампанент з параметрамі). Паварот / зрух: дыяпазон параметра адлюстроўваецца на Ад..Да (градусы / даўжыня); без дыяпазону або з выразам значэнне і ёсць рух
rig-motion-rotate = Павярнуць
rig-motion-slide = Зрушыць
rig-motion-show = Паказаць / схаваць
rig-motion-paint = Пафарбаваць
rig-motion-set = Задаць параметр
rig-parameter = (параметр)
rig-part-no-params = У часткі няма параметраў
rig-part-param-hint = Уласны параметр часткі, якому задаецца значэнне
rig-by = па
rig-expression = (выраз)
rig-from = ад
rig-to = да
rig-remove-motion = Выдаліць рух
rig-axis-through = Вось праз
rig-along = уздоўж
rig-direction = Кірунак
rig-pick = Выбраць…
rig-pick-hint = Пстрыкніце дзве кропкі ў мадэлі
rig-axes-note = мм, восі кампанента
rig-add-motion = + Рух

## Скрыпт
rig-script = Скрыпт
rig-script-hint = Неабавязковы код Rhai, што выконваецца пасля рухаў унутры кампанента: fn update(p) {"{"} … {"}"} з поўным API скрыптаў (rotate, move, paint, show / hide, set_params, box…); child(name) / child(name, true) знаходзіць частку па імені (углыб), part.min / max / size / bounds — яе межы; p — значэнні і формулы
rig-add-script = + Скрыпт

## Кнопкі
rig-add = Дадаць параметры
rig-revert = Вярнуць
rig-remove = Выдаліць параметры
rig-remove-hint = Копіі зноў паказваюць кампанент як намаляваны

## ======== Генератары ========

## Рэдактар генератараў
gen-editor-title = Рэдактар генератараў
gen-new = Новы
gen-new-hint = Пачаць з шаблона
gen-open = Адкрыць…
gen-filter = Генератар
gen-save = Захаваць у бібліятэку
gen-save-hint = Запісаць генератар у { DATA_DIR() }/generators (ён з’явіцца ў латку Кампаненты)
gen-apply-hint = Абнавіць гэты генератар дакумента: усе яго кампаненты будуць перабудаваны (адзін крок адмены)
gen-apply-doc = Ужыць да дакумента
gen-apply-doc-hint = Выкарыстаць гэтую крыніцу ў дакуменце: яго кампаненты будуць перабудаваны (адзін крок адмены)
gen-place = Размясціць
gen-place-hint = Размясціць кампанент са значэннямі з падгляду
# $name is the generator's name in the document
gen-in-model = У мадэлі: { $name }
gen-not-saved = не захавана
gen-build-failed = Збой пабудовы
gen-drag-turn = Цягніце, каб павярнуць
gen-defaults = Па змаўчанні
gen-no-home = Няма хатняй папкі
# $path is a file path; $error the system's message
gen-saved = Захавана ў { $path }
gen-cannot-save = Не ўдалося захаваць { $path }: { $error }
gen-name-taken = У дакуменце ўжо ёсць генератар з імем { $name }
gen-undo-update = Абнавіць генератар { $name }
# $old and $name are generator names
gen-renamed-applied = { $old } → { $name }: ужыта да дакумента
gen-same-source = { $name }: дакумент ужо выкарыстоўвае гэту крыніцу
gen-applied = { $name }: ужыта да дакумента

## Спіс генератараў і пліткі
# $label is a parameter label
gen-not-length = { $label }: не даўжыня
gen-formulas = Формулы
gen-origin-bundled = Убудаваны прыклад (Праўка → Захаваць у бібліятэку робіць вашу копію)
gen-origin-document = У гэтым дакуменце
gen-tile-error = Памылка: { $error }
# $note is the description and origin lines
gen-tile-tip =
    Генератар — параметры ў Уласцівасцях
    { $note }
gen-edit = Правіць генератар…
gen-show-in-finder = Паказаць у файлавым менеджары
    .macos = Паказаць у Finder
    .windows = Паказаць у Правадыры
gen-delete = Выдаліць…
gen-error = Генератар: { $error }

## Блок «Уласцівасці»
# $name is the generator's name; $n the number of components
gen-info-components = { $name } — кампанентаў: { $n }
gen-info-generator = Генератар: { $name }
gen-reset-hint = Вярнуць значэнні генератара па змаўчанні
gen-unique-generated = Копія яго генератара для гэтага кампанента («Паліца 2» у раздзеле У мадэлі): праўка гэтага скрыпта змяняе толькі яго
gen-unique-hint = Даць гэтаму кампаненту ўласнае вызначэнне: яго праўка болей не змяняе іншыя копіі
gen-unique-already = Гэта копія ўжо ўнікальная: ніхто больш не падзяляе яе вызначэнне або генератар
gen-unique-select = Вылучыце адзін кампанент
gen-make-unique = Зрабіць унікальным
gen-mixed = Вылучаныя кампаненты адрозніваюцца; змена задае ўсім
gen-select-generated = Вылучыце створаныя кампаненты

## ======== Звесткі пра дакумент ========

model-info-title = Звесткі пра дакумент
model-info-scene = У сцэне
model-info-scene-hint = Што намалявана: улічваецца змест кожнай размешчанай копіі
model-info-file = У файле
model-info-file-hint = Што захоўвае дакумент: кожнае вызначэнне адзін раз, выкарыстанае ці не
model-info-libraries = Бібліятэкі
# $n is a number, already formatted
model-info-files = файлаў: { $n }
# $used, $all and $unused are formatted numbers
model-info-used-of = { $used } з { $all } (не выкарыстана: { $unused })

## Радкі
model-info-row-polygons = Шматкутнікі (грані)
model-info-row-triangles = Трохкутнікі
model-info-row-edges = Рэбры
model-info-row-groups-placed = Размешчана груп
model-info-row-components-placed = Размешчана кампанентаў
model-info-row-components = Кампаненты
model-info-row-groups = Групы
model-info-row-materials = Матэрыялы
model-info-row-textures = Тэкстуры
model-info-row-tags = Тэгі
model-info-row-scenes = Сцэны
model-info-row-sheets = Лісты макета
model-info-row-generators = Генератары
model-info-row-lights = Святло
model-info-row-guides = Накіроўвальныя / памеры
model-info-row-file-size = Памер файла (захаванага)
model-info-row-lib-components = Бібліятэка кампанентаў
model-info-row-lib-materials = Бібліятэка матэрыялаў
model-info-preview-image = Відарыс падгляду
model-info-none = няма
model-info-show = Паказаць…
model-info-show-hint = Файл → Відарыс падгляду → Паказаць…

## Акно «Відарыс падгляду»
model-info-preview-title = Відарыс падгляду
model-info-no-preview = Відарыса падгляду няма — бібліятэка кампанентаў малюе мадэль
model-info-preview-note = Захоўваецца ў файле; бібліятэка кампанентаў паказвае яго для гэтага файла.
model-info-from-file = З файла відарыса…
model-info-from-view = З бягучага выгляду

## Ачыстка нявыкарыстанага
model-info-purge = Ачысціць нявыкарыстанае
model-info-purge-note = Выдаляе тое, што нічога ў сцэне не выкарыстоўвае (Праўка → Адмяніць вяртае назад).
# $n is the number of unused definitions / materials
model-info-purge-defs = Кампаненты і групы ({ $n })
model-info-purge-mats = Матэрыялы ({ $n })
model-info-nothing-to-purge = Нявыкарыстанага для ачысткі няма
model-info-purge-failed = Ачыстка не ўдалася
# $n is a count
model-info-purged-components = кампанентаў: { $n }
model-info-purged-groups = груп: { $n }
model-info-purged-materials = матэрыялаў: { $n }
# $parts is a comma-separated list of the three texts above
model-info-purged = Ачышчана: { $parts }

## ======== Плагіны ========

## Памылкі загалоўка плагіна (бачаць аўтары плагінаў)
# $label is a parameter label; $value is the text entered
ext-param-invalid = { $label }: недапушчальнае значэнне: { $value }
# $text is the @param line
ext-param-needs-default = @param патрабуе `name = default`: { $text }
ext-param-bad-name = дрэннае імя параметра `{ $key }`
ext-param-unclosed-quote = незакрытае двукоссе: { $text }
ext-param-unknown-type = невядомы тып параметра `{ $ty }` (length, int, number, text, bool)
ext-pick-needs-number = @pick патрабуе колькасць кропак: { $text }
ext-pick-range = @pick: ад 1 да 16 кропак
ext-pick-default = Пстрыкніце кропак: { $n }
ext-select-default = Вылучыце аб’екты
ext-empty-name = пусты @name

## Плагіны матэрыялаў
ext-mat-plugin-failed = плагін завяршыўся са збоем
ext-mat-no-answer = плагін не вывеў адказу
# $error is the JSON parser's message; $answer is the start of the plugin's answer
ext-mat-not-json = яго адказ не JSON ({ $error }): { $answer }
ext-mat-catalogue-array = catalogue() павінна вяртаць масіў матэрыялаў
ext-mat-no-home = няма хатняй папкі
ext-mat-default-name = Матэрыял
ext-mat-import-nothing = import_material() нічога не вярнула
ext-tex-bad-size = Убудаваная выява: "size" — адзін бок або [шырыня, вышыня] ад 1 да 8192 пікселяў
ext-tex-no-pixels = Убудаваная выява: перадайце пікселі ў "rgb", "rgba" або "grey" (масіў 0..255, base64 або blob)
ext-tex-pixel-count = Убудаваная выява: чакалася байтаў: { $want }, перададзена: { $got }
ext-mat-spec-object = апісанне матэрыялу павінна быць аб’ектам

## Меню Плагіны і дыялогі плагінаў
ext-script-undo = Скрыпт
# $error is the plugin header parser's message
ext-header-error = загаловак: { $error }
ext-no-plugins = Плагінаў няма
# $n is the number of plugins
ext-failed-to-load = Не ўдалося загрузіць плагінаў: { $n }…
ext-reload = Перазагрузіць плагіны
ext-open-folder = Адкрыць папку плагінаў
ext-script-gone = скрыпт знік
# $name is the plugin's or script's name (log lines)
ext-run-script = скрыпт { $name }
ext-run-plugin = плагін { $name }
ext-plugin-failed = плагін завяршыўся са збоем
ext-done = { $name } гатова
ext-exited = { $name }: завяршыўся з памылкай ({ $status })
ext-python-no-bridge = Python-скрыптам патрэбны мост (выключаны праз VARSTOTAS_NO_BRIDGE)
ext-python-busy = Python-скрыпт ужо выконваецца
ext-python-cant-start = Не ўдалося запусціць { $python }: { $error } (задайце VARSTOTAS_PYTHON)
ext-pick-points = Выбар кропак
ext-select-objects = Выбар аб’ектаў
ext-run = Запусціць

## ======== Кансоль скрыптоў і даведнік па скрыптах ========

## Кансоль скрыптоў
console-title = Кансоль скрыптоў
console-script-failed = скрыпт завяршыўся з памылкай
# $error is the script's error text
console-script-error = Памылка скрыпту: { $error }
# $name is the script file name; this is the undo step name
console-script-undo = Скрыпт { $name }
# $path is a file path; $error is the system's message
console-cannot-read = Не ўдалося прачытаць { $path }: { $error }
console-hint = Скрыпты Rhai над мадэллю. Паспрабуйце: box(ORIGIN, 1000, 600, 18)  — %CMD%+Enter запускае
console-run = Запусціць
console-run-hint = %CMD%+Enter
console-open = Адкрыць…
console-save = Захаваць…
console-stop-python = Спыніць Python
console-clear = Ачысціць вывад
# $port is a TCP port number
console-bridge = Мост Python / MCP: 127.0.0.1:{ $port }
console-bridge-hint = import varstotas; m = varstotas.connect()  — глядзіце { DATA_DIR() }/python
console-filter-rhai = Скрыпт Rhai

## Акно «Даведнік па скрыптах»
console-ref-title = Даведнік па скрыптах
console-search = Пошук
console-search-hint = функцыя, уласцівасць або слова
console-fulltext = Поўны тэкст: docs/SCRIPTING.md, docs/GENERATORS.md
console-copy-example = Скапіяваць прыклад
console-click-copy = Націсніце, каб скапіяваць
console-topic-basics = Асновы
console-topic-console = Кансоль і плагіны
console-topic-generators = Генератары
console-topic-component = Скрыпт кампанента
console-topic-functions = Функцыі
console-topic-properties = Уласцівасці
console-param-types = Тыпы параметраў
console-generators-note = Унутры `build` працуе ўвесь API (гл. Функцыі). Скрыпт выконваецца ў пустой мадэлі з матэрыяламі і тэгамі дакумента; усё, што ён пабудуе, становіцца змесцівам кампанента.
console-functions-intro = Усе функцыі API па раздзелах. Націсніце на сігнатуру, каб скапіяваць яе. `x` — аб’ект, масіў аб’ектаў або імя даччынай групы.
console-properties-intro = Пасля кропкі: `g.name`, `f.normal.x`, `door.size.z`, `[1, 2].len()`. Уласцівасці з пазнакай «прызначаецца» можна задаваць: `g.name = "Crate"`, `g.hidden = true`.
console-kind-objects = Аб’екты (грані, рэбры, групы, кампаненты)
console-kind-vectors = Вектары
console-kind-bounds = Межы — bounds(x), x.bounds
console-kind-arrays = Масівы
console-kind-strings = Радкі
console-nothing-found = Нічога не знойдзена
console-members = Уласцівасці і метады
console-tags = Загалоўныя тэгі
console-reference = ? Даведнік
console-reference-hint = Даведка → Даведнік па скрыптах: функцыі, уласцівасці, прыклады (F1 у рэдактары: слова пад курсорам)

## Старонка: асновы
console-ref-basics-1 = Скрыпты пішуцца на Rhai — невялікай мове з сінтаксісам, блізкім да Rust / JavaScript. Каардынаты — сусветныя міліметры; даўжыні з адзінкамі — радкі: `length("1.2m")`.
console-ref-basics-2 = Мова
console-ref-basics-4 = Радкі: `"text"`, злучаюцца праз `+`: `"w = " + w`. Метады радкоў `trim()`, `to_upper()` мяняюць радок на месцы і нічога не вяртаюць: `s.trim(); if s == "" {"{"} … {"}"}`.
console-ref-basics-5 = Канстанты: `ORIGIN`, `X`, `Y`, `Z` (адзінкавыя восі). Вектары складваюцца, адымаюцца і маштабуюцца: `ORIGIN + X * 100`; `.x .y .z`, `length()`, `normalize()`, `dot(b)`, `cross(b)`, `distance(b)`.
console-ref-basics-6 = Аб’екты
console-ref-basics-7 = Грані, рэбры, групы і кампаненты — гэта спасылкі: `let g = box(ORIGIN, 100, 100, 100); g.name = "Crate";`. Пасля аперацый, якія зліваюць або рэжуць геаметрыю, старая спасылка можа стаць няўдалай — правярайце `e.valid`.
console-ref-basics-8 = Усюды, дзе чакаецца аб’ект, радок называе групу / кампанент бягучага кантэксту: `move("Door", vec3(0, 10, 0))`. `child("Knob", true)` шукае і ў укладзеных групах.
console-ref-basics-9 = Кожны аб’ект мае `kind`, `bounds`, `size`, `min`, `max`, `center`, `hidden`; групы — яшчэ `name`, `definition`, `material`, `tag` (гл. Уласцівасці).
console-ref-basics-10 = Кантэкст і адмена
console-ref-basics-11 = Новая геаметрыя трапляе ў бягучы кантэкст: групу, адкрытую на пачатку скрыпту, або тую, у якую ўвайшлі праз `enter(g)` (выхад — `exit()`).
console-ref-basics-12 = Адзін запуск — адзін крок адмены. Скрыпт, які завяршыўся з памылкай, нічога не мяняе; памылка з нумарам радка паказваецца і захоўваецца ў Акно → Журнал.
console-ref-basics-13 = `print(x)` піша ў вывад; значэнне апошняга выразу таксама друкуецца. Скрыпты спыняюцца праз 30 с (генератары — праз 10 с).
## Старонка: кансоль
console-ref-console-1 = Акно → Кансоль скрыптоў запускае код над адкрытай мадэллю. %CMD%+Enter запускае, %CMD%+↑ / ↓ перамяшчаецца па ранейшых запусках, Адкрыць… / Захаваць… працуюць з файламі `.rhai`. Файл → Запусціць скрыпт… запускае файл.
console-ref-console-2 = Падказкі паказваюць функцыі API падчас набору, уласцівасці пасля кропкі (па тыпе злева) і сігнатуру выкліканай функцыі над курсорам. F1 адкрывае гэты даведнік на слове пад курсорам.
console-ref-console-3 = Загаловак: дыялогі і пункты меню
console-ref-console-4 = Радкі `// @tag …` уверсе ператвараюць скрыпт у плагін: Запуск спачатку пытае параметры, кропкі і аб’екты. Файл у `{ DATA_DIR() }/plugins` становіцца пунктам меню Плагіны.
console-ref-console-6 = Тэгі
## Старонка: генератары
console-ref-generators-1 = Генератар — гэта кампанент з параметрамі: скрыпт, чый `build(p)` будуе змесціва кампанента ў яго ўласных восях (пачатак каардынат — кропка ўстаўкі). Кожная копія мае свае значэнні ў Уласцівасці; копіі з аднолькавымі значэннямі выкарыстоўваюць адно азначэнне.
console-ref-generators-2 = Акно → Рэдактар генератараў (або Правіць генератар… на плітцы): зыходны код злева, жывы прагляд з палямі параметраў справа; Ужыць абнаўляе дакумент, Захаваць у бібліятэку піша файл.
console-ref-generators-4 = Загаловак
console-ref-generators-5 = абавязкова; ключ генератара ў дакуменце
console-ref-generators-6 = значэнне ў `p`; кожная частка дыяпазону неабавязковая
console-ref-generators-7 = значэнне, вылічанае з параметраў (толькі для чытання ў Уласцівасці)
console-ref-generators-8 = значэнні, для якіх ён ілжывы, адхіляюцца
console-ref-generators-9 = Тыпы параметраў
## Старонка: кампанент
console-ref-component-1 = Акно → Параметры кампанента дае любому кампаненту параметры: радкі значэнняў, формулы, рухі яго частак (павярнуць, зрушыць, паказаць, афарбаваць, задаць параметр часткі) і неабавязковы скрыпт.
console-ref-component-2 = Скрыпт вызначае `update(p)`. Ён працуе пасля рухаў, унутры кампанента, з усім API скрыптоў — тымі ж функцыямі, што ў кансолі і генератарах. `p` змяшчае значэнні і формулы.
console-ref-component-4 = Пошук частак
console-ref-component-5 = група / кампанент непасрэдна ўнутры кампанента
console-ref-component-6 = шукаецца і ў укладзеных групах (перамагае бліжэйшая)
console-ref-component-7 = унутры часткі
console-ref-component-8 = усе часткі (і ўкладзеныя)
console-ref-component-9 = бягучыя межы часткі (сусветныя)
console-ref-component-10 = схаваць, як hide(part)
console-ref-component-11 = Як гэта працуе
console-ref-component-12 = Скрыпт мяняе копію (варыянт для гэтых значэнняў), але не асноўны ўзор: часткі, якія ён змяніў, капіююцца ў варыянт, а незакранутыя застаюцца агульнымі. Копіі з аднолькавымі значэннямі выкарыстоўваюць адзін варыянт.
console-ref-component-13 = Праўкі, зробленыя ўнутры копіі (двайны націск), вяртаюцца ў асноўны ўзор без таго, што зрабіў скрыпт: дададзенае адкідаецца, перамяшчэнні / схаванне / афарбоўка скасоўваюцца.
console-ref-component-14 = Памылкі паказваюцца пры націсканні Ужыць і захоўваюцца ў Акно → Журнал.

## Рэдактар кода
# $sel is the selected completion's number, $n the number of completions
console-completion-count = { $sel } з { $n }

## ======== Журнал ========

log-level-error = памылка
log-level-warning = папярэджанне
log-title = Журнал
# $n is a count
log-errors = Памылкі { $n }
log-warnings = Папярэджанні { $n }
log-filter = фільтр
log-clear = Ачысціць
log-copy = Капіяваць
log-copy-hint = Скапіяваць паказаныя запісы
log-empty = У гэтым сеансе няма памылак і папярэджанняў.
log-no-match = Нічога не адпавядае фільтрам.
# $n is the number of errors not seen yet
log-badge-errors = ● { $n ->
        [one] { $n } памылка
        [few] { $n } памылкі
        [many] { $n } памылак
       *[other] { $n } памылкі
    }
log-badge = Журнал
log-badge-hint = Акно → Журнал: памылкі і папярэджанні адусюль (скрыпты, імпарт / экспарт, бібліятэкі …)

## ======== Ліцэнзіі ========

licences-tab-varstotas = Varstotas
licences-tab-components = Адкрытая праграмная частка
licences-tab-formats = Фарматы файлаў
licences-tab-libraries = Анлайн-бібліятэкі

licence-cc0 = CC0
licence-cc-by = CC-BY
licence-cc-by-sa = CC-BY-SA
licence-cc-by-nc = CC-BY-NC
licence-cc-by-nd = CC-BY-ND
licence-standard = Стандартная / асабістае карыстанне
licence-unknown = Невядома

licence-cc-by-nc-note = некамерцыйная: не прадавайце тое, што зроблена з яе дапамогай
licence-cc-by-nd-note = без вытворных: не распаўсюджвайце змененую копію
licence-cc-by-sa-note = на тых жа ўмовах: распаўсюджвайце змененыя копіі пад той жа ліцэнзіяй
licence-standard-note = уласная ліцэнзія сайта (бясплатна / асабістае карыстанне / умовы выкарыстання): часта без камерцыйнага выкарыстання або без распаўсюджвання самога файла мадэлі — праверце яе
licence-unknown-note = ліцэнзія невядомая: толькі для ўласнага карыстання

# Licences window: labels and the explanations of formats and online libraries (the licence texts themselves are not translated).

## Акно
licences-title = Ліцэнзіі
licences-count = { $n ->
        [one] { $n } кампанент
        [few] { $n } кампаненты
        [many] { $n } кампанентаў
       *[other] { $n } кампанента
    }
licences-filter = Фільтр: назва або ліцэнзія
licences-save = Захаваць як тэкст…
licences-save-hint = Усе кампаненты і тэксты іх ліцэнзій у адным файле
licences-saved = Захавана { $path }
licences-pick = Абярыце кампанент, каб убачыць яго ліцэнзію.
licences-all-text-title = Varstotas выкарыстоўвае наступныя кампаненты з адкрытым кодам.
licences-applies-to = Датычыцца: { $users }

## Укладка Varstotas
licences-own = Varstotas — свабоднае праграмнае забеспячэнне пад ліцэнзіяй MIT: можна выкарыстоўваць для асабістай і камерцыйнай працы, капіяваць, змяняць і распаўсюджваць.
licences-programs = Праграмы, што выкарыстоўваюцца, калі яны ў вас ёсць
licences-yours = Вашы мадэлі — вашы. Мадэлі і матэрыялы, спампаваныя з анлайн-бібліятэк, захоўваюць свае ліцэнзіі (Анлайн-бібліятэкі; Файл → Экспарт → Указанне аўтарства).

## Праграмы


licence-program-oidn-use = ШІ-шумапрыглушальнік рэндара
licence-program-oidn-licence = Apache-2.0 — не пастаўляецца, загружаецца з вашай устаноўкі

licence-program-python-use = плагіны і скрыпты Python
licence-program-python-licence = ліцэнзія PSF — не пастаўляецца, ваша ўстаноўка

## Укладка фарматаў файлаў
licence-formats-col-format = Фармат
licence-formats-col-varstotas = Varstotas
licence-formats-col-how = Як
licence-formats-col-note = Фармат

licence-format-varstotas-what = адкрыццё, захаванне
licence-format-varstotas-how = уласны фармат Varstotas
licence-format-varstotas-note = MIT, як і Varstotas

licence-format-gltf-what = імпарт, экспарт
licence-format-gltf-how = уласны чытач / запісвальнік; сеткі Draco праз draco-core (Apache-2.0)
licence-format-gltf-note = адкрыты стандарт Khronos, без ліцэнзійных выплат

licence-format-obj-what = імпарт, экспарт
licence-format-obj-how = уласны чытач / запісвальнік
licence-format-obj-note = адкрытая публічная спецыфікацыя

licence-format-stl-what = імпарт, экспарт
licence-format-stl-how = уласны чытач / запісвальнік
licence-format-stl-note = адкрытая публічная спецыфікацыя

licence-format-ply-what = імпарт, экспарт
licence-format-ply-how = уласны чытач / запісвальнік
licence-format-ply-note = адкрытая публічная спецыфікацыя

licence-format-3mf-what = імпарт, экспарт
licence-format-3mf-how = уласны чытач / запісвальнік (zip + XML)
licence-format-3mf-note = спецыфікацыя 3MF Consortium, без ліцэнзійных выплат

licence-format-collada-what = імпарт, экспарт
licence-format-collada-how = уласны чытач / запісвальнік (відарысы файламі побач)
licence-format-collada-note = адкрыты стандарт Khronos, без ліцэнзійных выплат

licence-format-fbx-what = імпарт
licence-format-fbx-how = ufbx (MIT / Unlicense), незалежны чытач
licence-format-fbx-note = фармат Autodesk; FBX — гандлёвая марка Autodesk, Varstotas не звязаны з ёй

licence-format-dxf-what = імпарт, экспарт
licence-format-dxf-how = уласны чытач / запісвальнік
licence-format-dxf-note = адкрыта апублікаваны фармат Autodesk; DXF — гандлёвая марка Autodesk

licence-format-svg-pdf-what = экспарт (чарцяжы, разгорткі, лісты)
licence-format-svg-pdf-how = уласныя запісвальнікі
licence-format-svg-pdf-note = адкрытыя стандарты W3C / ISO

licence-format-images-what = тэкстуры, відарысы, асяроддзі
licence-format-images-how = бібліятэка image (MIT / Apache-2.0)
licence-format-images-note = адкрытыя фарматы

licence-format-openexr-what = экспарт рэндара
licence-format-openexr-how = уласны запісвальнік
licence-format-openexr-note = Academy Software Foundation, BSD-3-Clause

licence-format-csv-json-what = імпарт кропак, экспарт граней (плагіны)
licence-format-csv-json-how = уласны код
licence-format-csv-json-note = адкрытыя фарматы

licence-format-gif-what = экспарт анімацыі
licence-format-gif-how = бібліятэка image (MIT / Apache-2.0)
licence-format-gif-note = адкрыты фармат

## Укладка анлайн-бібліятэк
licence-libs-intro = Кожная спампаваная мадэль захоўвае крыніцу, аўтара і ліцэнзію; Varstotas папярэджвае, калі ліцэнзія размешчанай або экспартаванай мадэлі абмяжоўвае яе выкарыстанне (некамерцыйная, без вытворных, на тых жа ўмовах, уласная ліцэнзія сайта, невядомая).
licence-libs-col-library = Бібліятэка
licence-libs-col-assets = Рэсурсы
licence-libs-col-licence = Ліцэнзія
licence-libs-col-terms = Умовы

licence-lib-poly-haven-what = мадэлі, матэрыялы
licence-lib-poly-haven-licence = CC0
licence-lib-poly-haven-terms = Бясплатны публічны API, таксама для камерцыйных праграм; просяць паказваць «Powered by Poly Haven» (паказваецца над рэсурсамі) і ўнікальны User-Agent (Varstotas).

licence-lib-ambientcg-what = матэрыялы
licence-lib-ambientcg-licence = CC0
licence-lib-ambientcg-terms = Бясплатны API і спампоўкі; указанне аўтарства вітаецца, але не абавязковае.

licence-lib-smithsonian-what = мадэлі
licence-lib-smithsonian-licence = пераважна CC0; некаторыя «Usage conditions apply» (некамерцыйныя)
licence-lib-smithsonian-terms = API не паведамляе, якія гэта: трэба праверыць старонку мадэлі. Varstotas пазначае такія мадэлі як «праверце ўмовы выкарыстання» і папярэджвае пры іх размяшчэнні або экспарце.

licence-lib-nasa-what = мадэлі
licence-lib-nasa-licence = не абаронена аўтарскім правам (грамадскі набытак ЗША)
licence-lib-nasa-terms = Эмблемы і лагатыпы NASA абаронены; нельга выкарыстоўваць так, нібы NASA гэта ўхваляе.

licence-lib-kenney-what = пакеты мадэляў
licence-lib-kenney-licence = CC0
licence-lib-kenney-terms = Чытаецца з публічных старонак рэсурсаў kenney.nl; лагатып Kenney не пад CC0.

licence-lib-quaternius-what = пакеты мадэляў
licence-lib-quaternius-licence = CC0
licence-lib-quaternius-terms = Спампоўваецца вамі ў браўзеры.

licence-lib-search-name = Пошук на сайтах (Printables, Thingiverse, Sketchfab, …)

licence-lib-search-what = мадэлі
licence-lib-search-licence = у кожнай мадэлі свая
licence-lib-search-terms = Varstotas толькі адкрывае пошук сайта ў вашым браўзеры; вы спампоўваеце самі і задаеце ліцэнзію ў картцы імпарту.

licence-lib-plugins-name = Плагіны

licence-lib-plugins-what = мадэлі, матэрыялы
licence-lib-plugins-licence = залежыць ад плагіна
licence-lib-plugins-terms = Плагін-крыніца паказвае свае ўмовы (радок @notice) над рэсурсамі; напрыклад, Sketchfab патрабуе ваш уласны токен акаўнта і ўказання аўтара і Sketchfab.

## ======== Кіраўніцтва карыстальніка ========

help-topic-start = Пачатак працы
help-topic-navigation = Навігацыя па выглядзе
help-topic-selecting = Вылучэнне
help-topic-drawing = Маляванне
help-topic-shapes = 3D-фігуры
help-topic-modifying = Змяненне
help-topic-inference = Падказкі і прывязка
help-topic-measurements = Поле памераў
help-topic-groups = Групы і кампаненты
help-topic-library = Бібліятэка кампанентаў
help-topic-materials = Матэрыялы і тэкстуры
help-topic-organizing = Тэгі, Структура, Уласцівасці
help-topic-construction = Дапаможныя лініі, памеры, сячэнні
help-topic-styles = Стылі і рэндар
help-topic-scenes = Сцэны, лісты, чарцяжы
help-topic-files = Файлы, імпарт і экспарт
help-topic-scripts = Скрыпты і плагіны
help-topic-shortcuts = Клавіятура і мыш
help-topic-problems = Калі нешта пайшло не так
## start
help-start-1 = Varstotas — 3D-рэдактар: вы малюеце рэбры, замкнёныя рэбры ўтвараюць грані, грані націскаюцца і выцягваюцца ў цвёрдыя целы. Усё счэплена, пакуль вы не згрупуеце.
help-start-2 = Акно
help-start-3 = Радок меню
help-start-4 = Файл, Праўка, Выгляд, Камера, Маляваць, Інструменты, Плагіны, Акно, Даведка
help-start-5 = Панэль інструментаў
help-start-6 = інструменты (падказка называе кожны і яго клавішу), навігацыя і Прывязка; выгляды і стылі — у правым верхнім куце бачнай вобласці
help-start-7 = Бачная вобласць
help-start-8 = мадэль; чырвоная, зялёная і сіняя лініі — восі, пад імі зямля
help-start-9 = Бакавая панэль
help-start-10 = карткі справа: Уласцівасці, Матэрыялы, Кампаненты, Тэгі, Структура, Сцэны, Цені, Асяроддзе, Згладжванне рэбраў, Падказкі (Акно → Бакавая панэль уключае і выключае карткі)
help-start-11 = Радок стану
help-start-12 = што чакае актыўны інструмент, паведамленні і памылкі, кнопка Журнал, поле памераў
help-start-13 = Першая мадэль
help-start-14 = 1. Націсніце `R` (Прамавугольнік), пстрыкніце кут на зямлі, пстрыкніце процілеглы кут — або набярыце `1000,600` і Enter.
help-start-15 = 2. Націсніце `P` (Націснуць/Выцягнуць), пстрыкніце прамавугольнік, рушце ўверх і пстрыкніце — або набярыце `750` і Enter.
help-start-16 = 3. Націсніце `Space` (Вылучэнне), патройчы пстрыкніце скрынку, каб вылучыць усё, націсніце `G`, каб зрабіць групу.
help-start-17 = 4. Націсніце `B` (Вядро з фарбай), абярыце матэрыял у картцы Матэрыялы, пстрыкайце па гранях.
help-start-18 = 5. `Shift+Z` змяшчае мадэль у вакно; Файл → Захаваць піша файл `.vrst`.
help-start-19 = Даведка падчас працы
help-start-20 = Картка «Падказкі» тлумачыць актыўны інструмент; F1 над мадэллю адкрывае гэтае кіраўніцтва на ім. Гатовыя мадэлі для агляду (абстаўлены дом, галерэя матэрыялаў) ляжаць у папцы { DATA_DIR() }/examples: Файл → Адкрыць.
## navigation
help-navigation-1 = Мыш і трэкпад
help-navigation-2 = Перацягванне сярэдняй кнопкай
help-navigation-3 = арбіта вакол кропкі пад курсорам
help-navigation-4 = Shift + перацягванне сярэдняй
help-navigation-5 = зрух
help-navigation-6 = Кола
help-navigation-7 = маштаб да курсора
help-navigation-8 = Трэкпад: правядзенне двума пальцамі
help-navigation-9 = арбіта; з Shift — зрух
help-navigation-10 = Трэкпад: шчыпок або %CMD% + правядзенне
help-navigation-11 = маштаб
help-navigation-12 = Гэта працуе ў кожным інструменце, таму не трэба кідаць аперацыю малявання, каб агледзецца.
help-navigation-13 = Акно → Налады → Навігацыя мяняе іх: хуткасць арбіты і маштабу, перавернутае кола, маштаб да цэнтра выгляду замест курсора, зрух сярэдняй кнопкай (Shift круціць), два пальцы для зруху або маштабу.
help-navigation-14 = Выгляды
help-navigation-15 = Камера → Стандартныя выгляды (і выпадны спіс выгляду ў куце бачнай вобласці): Ізаметрыя, Зверху, Знізу, Спераду, Ззаду, Злева, Справа. Камера → Паралельная праекцыя / Перспектыва. `Shift+Z` — Паказаць усё: уся мадэль у полі зроку.
help-navigation-16 = Захаваныя выгляды — гэта сцэны, гл. Сцэны, лісты, чарцяжы.
help-navigation-17 = Уласныя элементы кіравання бачнай вобласці
help-navigation-18 = Тры выпадныя спісы-значкі ў правым верхнім куце бачнай вобласці змяшчаюць усё пра тое, як яна паказвае мадэль (навядзіце курсор, каб убачыць бягучую наладу):
help-navigation-19 = Выгляд
help-navigation-20 = стандартныя выгляды, Перспектыва / Паралельная праекцыя, Поле зроку, Паказаць усё, Бачныя вобласці 1 / 2 / 4
help-navigation-21 = Стыль
help-navigation-22 = Зацененая … Рэалістычная; Рэнтген, Рэбры, Контуры
help-navigation-23 = Паказ (вока)
help-navigation-24 = восі, зямля, неба (выкл. = звычайны светлы фон), сетка гэтага выгляду; дапаможныя лініі, цені, плоскасці сячэння / сячэнні / запаўненне мадэлі (для кожнага выгляду)
help-navigation-25 = Пераключальнікі пакідаюць меню адкрытым; пстрычка па-за ім закрывае яго.
help-navigation-26 = Некалькі бачных абласцей
help-navigation-27 = Выгляд → Бачныя вобласці (або выпадны спіс выгляду) паказвае 1, 2 (побач) або 4 (2 × 2) выгляды мадэлі. Кожны мае сваю камеру, стыль і пераключальнікі выгляду; новыя пачынаюцца зверху, спераду і справа ў паралельнай праекцыі. Пстрычка ў выглядзе робіць яго актыўным (сіняя рамка): меню, панэлі і інструменты дзейнічаюць на яго, і сама пстрычка ўжо працуе там.
help-navigation-28 = Набор (Выгляд → Бачныя вобласці або Наборы бачных абласцей у выпадным спісе выгляду) адразу задаюць звычайныя раскладкі, напрыклад, Спераду + Ізаметрыя або Спераду, Злева, Зверху + Ізаметрыя: плоскія выгляды ў паралельнай праекцыі, ізаметрычны — у перспектыве.
## selecting
help-selecting-1 = Праўка → Вылучыць усё (`%CMDK%A`), Зняць вылучэнне (`%CMDK%T`). Delete або Backspace сціраюць вылучанае; Праўка → Схаваць хавае яго, Паказаць усё вяртае ўсё назад.
help-selecting-2 = Пстрыкніце мадэль правай кнопкай для кантэкстнага меню: Сцерці, Схаваць, Стварыць групу, Стварыць кампанент…, Расчляніць, Перасячы грані, Перавярнуць / Арыентаваць грані, Адлюстраваць уздоўж, Тэкстура → Пазіцыя, Разгарнуць…, і каманды плоскасцей сячэння і восяў.
help-selecting-3 = Картка Структура паказвае групы і кампаненты дрэвам: пстрычка там вылучае, двайная — наводзіць на аб’ект.
## drawing
help-drawing-1 = Інструменты малявання працуюць на грані пад курсорам або на зямлі / актыўнай рабочай плоскасці. Замкнёныя контуры рэбраў становяцца гранямі; лінія праз грань дзеліць яе.
help-drawing-2 = Пакуль інструмент працуе, можна ў любы момант набраць значэнне (не трэба пстрыкаць у поле памераў) і націснуць Enter — гл. Поле памераў.
## shapes
help-shapes-1 = Маляваць → Фігуры будуе цэлае цвёрдае цела за адну аперацыю. Вынік — створаны кампанент: вылучыце яго і змяняйце памеры і сегменты ў Уласцівасці; Расчляніць ператварае яго ў звычайную геаметрыю.
## modifying
help-modifying-1 = Перамяшчэнне, Паварот і Маштабаванне працуюць на вылучаным або на тым, па чым вы пстрыкнулі, калі нічога не вылучана. Перамяшчэнне вяршыні, рабра або грані расцягвае геаметрыю вакол.
help-modifying-2 = Копія
help-modifying-3 = Перамяшчэнне / Паварот з %COPY% робіць копію
help-modifying-4 = Масівы
help-modifying-5 = пасля копіі набярыце `5x` (пяць копій з тым жа крокам) або `/5` (пяць копій паміж)
help-modifying-6 = Меню Праўка і кантэкстнае меню
help-modifying-7 = Перасячы грані
help-modifying-8 = з мадэллю, вылучэннем або кантэкстам: рэбры, дзе грані перасякаюцца
help-modifying-9 = Сячэнне
help-modifying-10 = плоскасць рэжа вылучанае або кантэкст; з %COPY% (Падзел) кожнае разрэзанае цела становіцца дзвюма замкнёнымі групамі
help-modifying-11 = Перавярнуць / Арыентаваць грані
help-modifying-12 = павярнуць пярэдні бок (белы) вонкі
help-modifying-13 = Адлюстраваць уздоўж
help-modifying-14 = люстэрка вылучанага ўздоўж чырвонага, зялёнага або сіняга напрамку
help-modifying-15 = Картка Згладжванне рэбраў
help-modifying-16 = змякчаць / згладжваць рэбры па вугле
help-modifying-17 = Акно → Разгарнуць
help-modifying-18 = раскласці грані або групы ў выкрайку (SVG / PDF / DXF)
## inference
help-inference-1 = Падчас малявання курсор прыляпляецца да кропак і напрамкаў мадэлі; маркер і падказка кажуць, што знойдзена.
help-inference-2 = Канцавая кропка, Сярэдзіна
help-inference-3 = зялёная / блакітная кропка
help-inference-4 = На рабры, На грані
help-inference-5 = чырвоны квадрат / сіні ромб
help-inference-6 = Цэнтр, Перасячэнне
help-inference-7 = акружнасцей і дуг, рэбраў
help-inference-8 = На чырвонай / зялёнай / сіняй восі
help-inference-9 = лінія колеру восі
help-inference-10 = Паралельна / Перпендыкулярна
help-inference-11 = пурпурная лінія ўздоўж рабра
help-inference-12 = Ад кропкі
help-inference-13 = пункцірная лінія ад кропкі ўздоўж восі
help-inference-14 = Фіксацыя
help-inference-15 = Утрымлівайце `Shift`, каб зафіксаваць знойдзены напрамак; стрэлкі фіксуюць вось: → чырвоная, ← зялёная, ↑ сіняя, ↓ паралельна / перпендыкулярна апошняму рабру.
help-inference-16 = Меню Прывязка
help-inference-17 = Кнопка-магніт (і меню Прывязка) уключае і выключае кожную падказку; `Shift+S` выключае і ўключае прывязку цалкам. Крокі акругляюць вольныя перамяшчэнні (10 мм), вуглы (15°) і каэфіцыенты маштабу — прывязка да геаметрыі і набраныя значэнні маюць перавагу.
help-inference-18 = Выгляд → Сетка паказвае сетку са сваёй прывязкай; крок — у Акно → Налады → Маляванне, а пры актыўным Вылучэнні набор даўжыні задае яго.
help-inference-19 = Рабочыя плоскасці
help-inference-20 = Інструмент Накіроўвалая плоскасць (меню Інструменты) і Акно → Рабочыя плоскасці ствараюць плоскасць для малявання ў паветры: кропкі ў пустой прасторы ляжаць на ёй, восі і сетка ідуць за ёй.
## measurements
help-measurements-1 = Поле памераў справа ў радку стану паказвае бягучую даўжыню, вугал або памер. Проста набірайце, пакуль інструмент працуе, і націсніце Enter; Backspace выпраўляе, Esc ачышчае.
help-measurements-2 = 1200, 1.2m, 120cm, 4', 1'6"
help-measurements-3 = даўжыні: адзінка мадэлі або названая вамі
help-measurements-4 = 1000,600
help-measurements-5 = два значэнні: бакі прамавугольніка, аснова скрынкі
help-measurements-6 = 45
help-measurements-7 = вугал (Паварот, Транспарцір, дугі)
help-measurements-8 = 24s
help-measurements-9 = сегменты акружнасці, дугі або фігуры; бакі многавугольніка
help-measurements-10 = 5x, /5
help-measurements-11 = копіі ў радзе пасля копіі Перамяшчэння / Павароту
help-measurements-12 = 2, 0.5
help-measurements-13 = каэфіцыент маштабу
help-measurements-14 = Паўторны набор адразу пасля аперацыі мяняе гэтую аперацыю (іншая даўжыня, больш копій).
help-measurements-15 = Адзінка мадэлі бярэцца з шаблона (Файл → Новы з шаблона); `,` і `.` абодва працуюць як дзесятковы падзельнік.
## groups
help-groups-1 = Геаметрыя счэпліваецца: рэбры і грані, што датыкаюцца, зліваюцца і дзеляць адно аднаго. Каб частка была асобнай, зрабіце яе групай або кампанентам.
help-groups-2 = Стварыць групу (G)
help-groups-3 = група: адзін аб’ект; копіі незалежныя
help-groups-4 = Стварыць кампанент…
help-groups-5 = кампанент: копіі маюць адно азначэнне — праўка адной праўляе ўсе
help-groups-6 = Двайны націск
help-groups-7 = увайсці ў групу / кампанент для праўкі (астатняе цямнее); Esc або Праўка → Закрыць групу выходзіць
help-groups-8 = Расчляніць
help-groups-9 = назад да вольнай геаметрыі
help-groups-10 = Зрабіць унікальным (правы націск, Уласцівасці)
help-groups-11 = копія кампанента атрымлівае сваё азначэнне («Box 2»): яе праўка больш не мяняе іншыя копіі; створаны генератарам атрымлівае копію свайго генератара («Shelf 2» у Ў мадэлі)
help-groups-12 = Паўторнае размяшчэнне з бібліятэкі
help-groups-13 = копія ў мадэлі выкарыстоўваецца паўторна, пакуль не зменена; пасля праўкі арыгінал з бібліятэкі прыходзіць як новае азначэнне («Chair 2»)
help-groups-14 = Кампаненты з параметрамі
help-groups-15 = Праўка → Зрабіць параметрычным дае кампаненту памеры X / Y / Z (расцягванне або маштаб) і шэраг копій у Уласцівасці.
help-groups-16 = Акно → Параметры кампанента дае любому кампаненту параметры: значэнні і формулы, што паварочваюць, зрушваюць, паказваюць або фарбуюць яго часткі, неабавязковы скрыпт; кожная копія мае свае значэнні (паўзункі ў Уласцівасці).
help-groups-17 = Генератары — гэта кампаненты, пабудаваныя скрыптам з тыпізаванымі параметрамі (Акно → Рэдактар генератараў). Целы з Маляваць → Фігуры — таксама генератары.
help-groups-18 = Скрыпт кампанента ў Даведніку па скрыптах
help-groups-19 = Генератары ў Даведніку па скрыптах
## library
help-library-1 = Карткі Кампаненты і Матэрыялы выглядаюць аднолькава: верхні рад (Пошук, Абнавіць, + для дадання, кнопка воблака для анлайн-бібліятэкі, … для тэчкі бібліятэкі, што паказваць і памер пліткі: Малая / Вялікая / Спіс), потым раздзелы Ў мадэлі (адкрыты дакумент), Абранае, Нядаўнія, Мая бібліятэка са сваімі тэчкамі і Прыклады. Раздзел згортваецца стрэлкай; назва тэчкі адкрывае яе ў Finder.
help-library-2 = Правы націск на пліце → Дадаць у абранае
help-library-3 = пліта з ☆ таксама паказваецца ў Абраным уверсе (Прыбраць з абранага здымае яе); Нядаўнія пералічваюць тое, што вы нядаўна размясцілі або абралі для афарбоўкі
help-library-4 = Правы націск на назве тэчкі
help-library-5 = Новая тэчка…; для падтэчкі яшчэ Перайменаваць тэчку… і Выдаліць тэчку (толькі калі яна пустая)
help-library-6 = Правы націск на пліце → Перамясціць у тэчку
help-library-7 = перамясціць файл у іншую тэчку бібліятэкі (абранае ідзе за ім)
help-library-8 = Перацягнуць пліту ў бачную вобласць
help-library-9 = кампанент размяшчаецца там, дзе вы адпусцілі; матэрыял фарбуе грань пад курсорам
help-library-10 = Кампаненты паказваюць вашу бібліятэку (`{ DATA_DIR() }/components` або тэчку, абраную праз …), генератары, убудаваныя Прыклады і кампаненты адкрытага дакумента.
help-library-11 = Пстрычка па пліце
help-library-12 = размясціць яе: копія ідзе за курсорам, пстрычка — паставіць
help-library-13 = + → Захаваць вылучанае ў бібліятэку…
help-library-14 = запісаць вылучанае як новы файл `.vrst`
help-library-15 = Правы націск на пліце
help-library-16 = Размясціць, Задаць прагляд з выгляду, Задаць відарыс прагляду…, Прыбраць відарыс прагляду, Перайменаваць…, Паказаць у Finder, Выдаліць…; для кампанента дакумента яшчэ Вылучыць экзэмпляры, Захаваць у бібліятэку, Параметры…, Выдаліць з дакумента…
help-library-17 = Малюнкі
help-library-18 = Пліта паказвае малюнак файла, калі ён ёсць, інакш намаляваны эскіз. Задаць прагляд з выгляду бярэ бачную вобласць такой, якая яна цяпер — спачатку скадруйце кампанент. Кампанент, размешчаны з бібліятэкі, бярэ малюнак файла з сабой; Файл → Відарыс прагляду задае малюнак адкрытага дакумента.
help-library-19 = Анлайн-бібліятэка 3D
help-library-20 = Кнопка воблака ў картцы Кампаненты (або Акно → Анлайн-бібліятэка 3D) спампоўвае мадэлі: Poly Haven, Smithsonian, NASA, пакеты Kenney, па URL або з крыніц плагінаў. Спампаваная мадэль становіцца кампанентам у бібліятэцы з указаннем аўтарства (Уласцівасці → Крыніца, Файл → Экспарт → Указанне аўтарства). Фільтры па ліцэнзіі, фармаце і памеры; абранае і гісторыя спамповак.
help-library-21 = Кожная крыніца паказвае свае ўмовы над мадэлямі (радок @notice плагіна). Мадэль, ліцэнзія якой абмяжоўвае выкарыстанне (некамерцыйная, без вытворных, на тых жа ўмовах, уласная ліцэнзія сайта, невядомая), атрымлівае папярэджанне пры размяшчэнні і пры экспарце дакумента. Даведка → Ліцэнзіі… пералічвае ліцэнзію Varstotas, кампаненты з адкрытым кодам, фарматы файлаў і анлайн-бібліятэкі з іх умовамі.
## materials
help-materials-1 = Картка Матэрыялы паказвае матэрыялы мадэлі (Ў мадэлі), вашу бібліятэку (`{ DATA_DIR() }/materials` або тэчку, абраную праз …) і Прыклады; правы націск на пліце дае Выкарыстаць, Перайменаваць…, Захаваць у бібліятэку, Паказаць у Finder, Экспарт…, Выдаліць…. Выбар матэрыялу бібліятэкі толькі ўзбройвае Вядро з фарбай: матэрыял далучаецца да мадэлі (Ў мадэлі) з першай афарбаванай граняй. Выдаліць на пліце Ў мадэлі прыбірае матэрыял з мадэлі, яго грані вяртаюцца да прадвызначанага (Праўка → Адрабіць вяртае яго). Там жа праўце матэрыял: колер, непразрыстасць, відарыс тэкстуры і яго памер, фізічныя ўласцівасці для стылю Рэалістычная (метал, шурпатасць, шкло, выпраменьванне, карты, пакрыццё, бляск).
help-materials-2 = Захаваць у бібліятэку / Экспарт… / + → Імпарт…
help-materials-3 = абмен матэрыялам паміж дакументамі (`.vrstmat`)
help-materials-4 = Кнопка воблака (картка Матэрыялы)
help-materials-5 = спампаваць PBR-сканы з Poly Haven і ambientCG або з крыніц плагінаў
help-materials-6 = Шэйдар
help-materials-7 = матэрыял можа мець свой WGSL-шэйдар з параметрамі (у рэдактара шэйдараў свой даведнік)
help-materials-8 = Грані маюць пярэдні (белы) і зваротны (шэра-сіні) бок; афарбоўка групы фарбуе грані ўнутры, што не маюць свайго матэрыялу.
## organizing
help-organizing-1 = Уласцівасці
help-organizing-2 = уласцівасці вылучанага: плошча, даўжыня, аб’ём цел, тэг, матэрыял, імя, параметры кампанентаў і генератараў
help-organizing-3 = Тэгі
help-organizing-4 = бачнасць, колер і імя тэгаў (слаёў); выдаленне тэга пераносіць яго змесціва ў Без тэга
help-organizing-5 = Структура
help-organizing-6 = дрэва груп і кампанентаў: Навесці, Правіць, Схаваць, Расчляніць, Вынесці з групы, Выдаліць; перацягвайце радкі, каб перамяшчаць аб’екты паміж групамі
help-organizing-7 = Схаваць / Паказаць усё
help-organizing-8 = Акно → Звесткі пра дакумент
help-organizing-9 = статыстыка дакумента (грані, трохкутнікі, кампаненты, матэрыялы, тэкстуры …) і Ачысціць нявыкарыстанае
## construction
help-construction-1 = Дапаможныя лініі вылучаюцца, перамяшчаюцца і выдаляюцца як геаметрыя; Праўка → Выдаліць дапаможныя лініі прыбірае іх усе, Выгляд → Дапаможныя лініі хавае іх. Інструменты → 3D-тэкст… робіць тэкст геаметрыяй.
help-construction-2 = Выгляд → Плоскасці сячэння / Сячэнні / Запаўненне сячэння кіруюць паказам сячэнняў; актыўнае сячэнне захоўваецца ў сцэнах.
## styles
help-styles-1 = Меню Выгляд (і выпадны спіс Стыль бачнай вобласці)
help-styles-2 = Зацененая, Зацененая з тэкстурамі, Манахромная
help-styles-3 = грані ў колерах матэрыялаў, з тэкстурамі, адным колерам
help-styles-4 = Схаваныя лініі, Каркас
help-styles-5 = толькі рэбры (схаваныя прыбраны або не)
help-styles-6 = Рэалістычная
help-styles-7 = фізічныя матэрыялы, асвятленне ад асяроддзя, сонца, эфекты — картка Асяроддзе задае неба, HDRI, экспазіцыю і эфекты
help-styles-8 = Рэнтген, Рэбры, Контуры
help-styles-9 = праглядванне праз грані; рэбры і тоўстыя контуры ўключаюцца і выключаюцца
help-styles-10 = Цені
help-styles-11 = цені ад сонца; час і дата — у картцы Цені
help-styles-12 = Рэндар
help-styles-13 = Акно → Рэндар рэндзіць выгляд трасіроўшчыкам шляхоў на GPU: шкло, адбіткі, святло, мяккія цені; перадустаноўкі якасці, памер кадра, глыбіня рэзкасці, шумапрыглушэнне (калі ўстаноўлены Intel OIDN), каляровая карэкцыя; PNG / EXR або ўсе сцэны адразу.
help-styles-14 = Крыніцы святла (кропкавыя, пражэктарныя, плоскія) размяшчаюцца з вакна Рэндар; яны вылучаюцца пстрычкай па маркеры і перамяшчаюцца і паварочваюцца як іншыя аб’екты.
help-styles-15 = Файл → Экспарт → 2D-графіка (PNG) захоўвае выгляд такім, які ён ёсць; Файл → Экспарт → Анімацыя… робіць аніміраваны GIF або кадры PNG сцэн або аблёту.
## scenes
help-scenes-1 = Картка Сцэны
help-scenes-2 = захаваныя выгляды са стылем, бачнымі тэгамі і сячэннем; укладкі над бачнай вобласцю пераключаюць іх
help-scenes-3 = Выгляд → Анімацыя
help-scenes-4 = Прайграць сцэны; PgDn / PgUp — наступная / папярэдняя сцэна
help-scenes-5 = Акно → Лісты
help-scenes-6 = старонкі кампаноўкі: бачныя вобласці сцэн у маштабе 1:N, тэксты, рамка, стандартныя выгляды з памерамі, табліца дэталяў; шматстаронкавы PDF
help-scenes-7 = Файл → Экспарт → 2D-чарцёж
help-scenes-8 = бягучы выгляд як вектарны SVG / PDF з контурамі, схаванымі лініямі, сячэннямі і памерамі
help-scenes-9 = Файл → Экспарт → Справаздача
help-scenes-10 = дэталі з памерамі для распілу, плошчы па матэрыялах (CSV)
## files
help-files-1 = Дакументы — гэта файлы `.vrst` (тэкстуры і малюнкі захоўваюцца ўнутры). Файл → Новы з шаблона выбірае адзінкі; Акно → Налады задае шаблон па змаўчанні і інтэрвал аўтазахавання.
help-files-2 = Акно → Налады мае раздзелы злева і пошук: Агульныя (аўтазахаванне, шаблон Файл → Новы, колькі крокаў адмены захоўваецца), Інтэрфейс (светлая, цёмная або сістэмная тэма, маштаб меню і панэляў, памер значкоў панэлі інструментаў), Малюнак (сетка, прывязка і крокі — тыя ж пераключальнікі, што ў меню Прывязка), Бібліятэкі (тэчкі кампанентаў і матэрыялаў, Прыклады, памер пліткі), Спампоўкі (тэчка пад наглядам), Навігацыя (мыш і трэкпад), Спалучэнні клавіш (пстрыкніце спалучэнне і націсніце новыя клавішы; Backspace здымае яго, ↺ вяртае прадвызначанае), Рэндар (налады рэндара новых дакументаў; дзе бібліятэка Open Image Denoise) і Выгляд (як выглядае новы дакумент: стыль, рэбры, восі, зямля, неба, сетка, цені, праекцыя, поле зроку; Выкарыстаць бягучы выгляд / Ужыць да бягучага выгляду). Змены ўжываюцца адразу; Скінуць вяртае раздзел да прадвызначаных.
help-files-3 = Аўтазахаванне піша незахаваныя змены ў `{ DATA_DIR() }/autosave`; пасля збою Varstotas прапануе аднавіць іх.
help-files-4 = Імпарт
help-files-5 = OBJ, STL, DXF, glTF / GLB, 3MF, COLLADA (DAE), FBX, PLY, кропкі з CSV і фарматы, дададзеныя плагінамі; файлы можна і пераносіць у вакно
help-files-6 = Экспарт
help-files-7 = OBJ, STL, DXF, glTF (GLB), 3MF, COLLADA (DAE), PLY, грані як JSON, PNG, чарцяжы SVG / PDF, лісты, выкрайкі, анімацыя, справаздача, указанне аўтарства
help-files-8 = Праўка → Выразаць / Капіяваць / Уставіць / Уставіць на месца
help-files-9 = Уставіць на месца захоўвае пазіцыю капіявання
help-files-10 = Файл → Відарыс прагляду
help-files-11 = малюнак, які бібліятэка паказвае для гэтага файла
## scripts
help-scripts-1 = Акно → Кансоль скрыптоў запускае скрыпты Rhai над мадэллю: будаваць, змяняць, вылучаць, вымяраць. Файл → Запусціць скрыпт… запускае файл `.rhai` (або `.py`). Адзін запуск — адзін крок адмены.
help-scripts-2 = Плагіны — гэта скрыпты ў `{ DATA_DIR() }/plugins`: яны становяцца пунктамі меню Плагіны з дыялогамі для параметраў і выбару кропак; яны могуць дадаваць імпартёры, экспартёры і крыніцы матэрыялаў / мадэляў.
help-scripts-3 = Python і ІІ-агенты могуць кіраваць Varstotas праз лакальны мост (кліент Python, сервер MCP).
help-scripts-4 = Адкрыць Даведнік па скрыптах
help-scripts-5 = Кансоль і плагіны
help-scripts-6 = Усе функцыі
## shortcuts
help-shortcuts-1 = Клавішы інструментаў працуюць, калі ні ў адным тэкставым полі няма фокуса. Літары, набраныя падчас аперацыі, ідуць у поле памераў.
## problems
help-problems-1 = Акно → Журнал
help-problems-2 = кожная памылка і папярэджанне сеанса — скрыпты, імпарт / экспарт, бібліятэкі, спампоўкі; чырвоны лічыльнік у радку стану азначае новыя памылкі
help-problems-3 = Праўка → Адрабіць
help-problems-4 = кожную аперацыю можна адрабіць; няўдалая аперацыя нічога не мяняе
help-problems-5 = Акно → Звесткі пра дакумент → Ачысціць нявыкарыстанае
help-problems-6 = прыбраць нявыкарыстаныя кампаненты, матэрыялы і тэгі, каб файл стаў меншы
help-problems-7 = Грані выглядаюць шэра-сінімі
help-problems-8 = вы бачыце іх адваротны бок: вылучыце іх, правы націск → Перавярнуць грані або Арыентаваць грані
help-problems-9 = Не атрымліваецца маляваць там, дзе чакаецца
help-problems-10 = праверце фіксацыі Shift / стрэлкамі, актыўную рабочую плоскасць (Акно → Рабочыя плоскасці) і ці не знаходзіцеся вы ўнутры групы (Esc выходзіць з яе)
help-problems-11 = Нечага не хапае
help-problems-12 = Праўка → Паказаць усё, картка Тэгі (схаваныя тэгі), Выгляд → Плоскасці сячэння

## Акно, пошук, спалучэнні клавіш
help-window-title = Кіраўніцтва карыстальніка
help-search = Пошук
help-search-hint = інструмент, каманда або слова
help-nothing-found = Нічога не знойдзена
help-in-the-guide = У кіраўніцтве
help-tools = Інструменты
help-commands = Каманды
help-use = Выкарыстаць
help-use-hint = Абраць гэты інструмент
help-shortcuts-note = Змяняйце іх у Акно → Налады → Спалучэнні клавіш. Мыш і трэкпад: гл. Навігацыя па выглядзе. Мадыфікатары вылучэння: гл. Вылучэнне.
help-key-cut-copy-paste-keys = %CMDK%X / %CMDK%C / %CMDK%V
help-key-cut-copy-paste = Выразаць / Капіяваць / Уставіць
help-key-delete = Delete, Backspace
help-key-delete-what = сцерці вылучанае
help-key-esc = Esc
help-key-esc-what = скасаваць аперацыю; выйсці з групы, якая праўляецца
help-key-shift = Shift (утрымліваць)
help-key-shift-what = зафіксаваць напрамак падказкі
help-key-arrows = → ← ↑ ↓
help-key-arrows-what = зафіксаваць чырвоную / зялёную / сінюю вось, паралельна апошняму рабру
help-key-enter = Enter
help-key-enter-what = ужыць значэнне, набранае ў полі памераў
help-key-f1 = F1
help-key-f1-what = гэтае кіраўніцтва па актыўным інструменце; у рэдактары скрыпту — Даведнік па скрыптах

## ======== Скрыпты ========

## Функцыі
script-fn-vec3 = Кропка / вектар (падыходзіць і масіў [x, y, z]).
script-fn-line = Рэбры ад a да b.
script-fn-polyline = Рэбры праз кропкі; замкнёная і плоская → грань.
script-fn-face = Грань праз кропкі.
script-fn-rect = Прамавугольная грань у плоскасці XY.
script-fn-circle = Грань-круг (або рэбры).
script-fn-polygon = Грань правільнага мнагавугольніка.
script-fn-arc = Рэбры дугі вакол center.
script-fn-box = Новая група-паралелепіпед (origin = мінімальны кут).
script-fn-cylinder = Новая група-цыліндр (аснова ў center).
script-fn-cone = Новая група-конус (аснова ў center, вяршыня на h вышэй).
script-fn-pyramid = Новая група-піраміда: аснова w × d ад origin, вяршыня на h вышэй яе сярэдзіны.
script-fn-sphere = Новая група-сфера вакол center.
script-fn-torus = Новая група-тор вакол center: r да сярэдзіны трубкі, tube = яе радыус.
script-fn-pushpull = Выцягнуць / уцягнуць грань; вяртае верхнюю грань.
script-fn-offset = Зрушыць контур грані (d < 0 — досярэдзіны); вяртае новую грань.
script-fn-follow-me = Правесці профіль-грань уздоўж шляху.
script-fn-erase = Сцерці аб’ект або масіў аб’ектаў.
script-fn-move = Перасунуць аб’екты на вектар.
script-fn-copy = Скапіраваць аб’екты са зрухам на вектар; вяртае копіі.
script-fn-rotate = Павярнуць аб’екты вакол восі.
script-fn-scale = Змяніць маштаб аб’ектаў адносна кропкі.
script-fn-group = Згрупаваць аб’екты; вяртае групу.
script-fn-component = Стварыць кампанент; вяртае яго асобнік.
script-fn-explode = Разабраць групу / кампанент.
script-fn-make-unique = Даць асобніку кампанента ўласнае вызначэнне.
script-fn-union = Аб’яднанне двух груп цвёрдых целаў.
script-fn-subtract = Цвёрдае цела a мінус b.
script-fn-intersect = Перасячэнне цвёрдых целаў.
script-fn-trim = a мінус b, b застаецца.
script-fn-split = Падзяліць два цвёрдыя целы на тры.
script-fn-outer-shell = Знешняя абалонка цвёрдых целаў.
script-fn-reverse = Перавярнуць грані.
script-fn-soften = Згладзіць рэбры да кута злому.
script-fn-material = Стварыць / перафарбаваць матэрыял; вяртае яго назву.
script-fn-paint = Размаляваць грані / групы (() здымае).
script-fn-tag = Прызначыць тэг (ствараецца, калі яго няма).
script-fn-make-parametric = Зрабіць групу параметрычнай.
script-fn-set-params = Задаць значэнні згенераванага кампанента або памер / колькасць копій параметрычнай групы.
script-fn-params = Значэнні згенераванага кампанента / параметры параметрычнай групы, або ().
script-fn-reset-params = Вярнуць згенераваны кампанент да значэнняў генератара па змаўчанні.
script-fn-generators = Назвы генератараў дакумента.
script-fn-add-generator = Дадаць / абнавіць генератар з яго зыходнага тэксту; вяртае яго назву.
script-fn-generate = Размясціць згенераваны кампанент у кропцы.
script-fn-dimension = Лінейны памер паміж кропкамі.
script-fn-note = Тэкставая нататка з выносным радком.
script-fn-enter = Далей будаваць унутры групы.
script-fn-exit = Выйсці з групы, у якую зайшлі апошняй.
script-fn-faces = Грані бягучага кантэксту.
script-fn-edges = Рэбры бягучага кантэксту.
script-fn-groups = Групы / кампаненты бягучага кантэксту.
script-fn-all = Усё ў бягучым кантэксце.
script-fn-entities = Усё ўнутры групы.
script-fn-child = Група / кампанент з імем name у бягучым кантэксце (або ўнутры g); deep = шукаць і ў укладзеных групах. () калі няма.
script-fn-children = Групы / кампаненты бягучага кантэксту (або ўнутры g); deep = і ўкладзеныя.
script-fn-show = Паказаць аб’екты (on = false: схаваць).
script-fn-hide = Схаваць аб’екты.
script-fn-selection = Вылучэнне.
script-fn-select = Задаць вылучэнне (() здымае).
script-fn-bounds = #{ "{" } min, max, size, center { "}" } у сусветных каардынатах.
script-fn-model-bounds = Межы ўсёй мадэлі.
script-fn-length = Разабраць даўжыню з адзінкамі ("1.2m") → мм.
script-fn-units = Адзінка даўжыні мадэлі.
script-fn-materials = Назвы матэрыялаў.
script-fn-all-faces = Усе грані, уключаючы ўкладзеныя групы.
script-fn-read-text = Прачытаць тэкставы файл.
script-fn-lang = Код мовы інтэрфейсу ("en", "ru", …): выбраць тэксты, якія плагін друкуе або паказвае.
script-fn-platform = "macos", "windows" або "linux".
script-fn-write-text = Запісаць тэкставы файл.
script-fn-file-exists = Файл або папка існуе.
script-fn-list-files = Файлы ў dir (з падпапкамі) з пашырэннем ("" = любое), адсартаваныя.
script-fn-list-dirs = Папкі ў dir, адсартаваныя.
script-fn-parse-json = Тэкст JSON → табліцы, масівы, лікі, радкі.
script-fn-fetch-text = Спампаваць вэб-старонку / адказ API як тэкст (HTTP(S)); headers: #{ "{" } "X-Auth-Token": key { "}" }.
script-fn-fetch-file = Спампаваць url у файл path.
script-fn-url-encode = Закадзіраваць тэкст працэнтамі для URL (пошукавы запыт).
script-fn-print = Вывесці ў журнал кансолі.
script-fn-len = Даўжыня масіва / радка.
script-fn-range = Цэлыя лікі ад from да to (to не ўключаецца), для цыклаў.
script-fn-min = Меншы з лікаў.
script-fn-max = Большы з лікаў.
script-fn-clamp = x, абмежаваны ў межах lo..hi.
script-fn-ceil = Акругліць уверх да цэлага (як float).
script-fn-noise = Гладкі шум 0..1 у пункце x, y (адна ячэйка = 1). Пры period > 0 паўтараецца кожныя period ячэек (бясшвовыя пліткі); seed выбірае ўзор.
script-fn-fbm = Фрактальны шум 0..1: актавы шуму, кожная ўдвая драбнейшая і ўдвая слабейшая; period і seed — як у noise.

## Раздзелы
script-group-points-and-vectors = Кропкі і вектары
script-group-drawing = Маляванне
script-group-solids = Цвёрдыя целы
script-group-editing = Рэдагаванне
script-group-groups-and-components = Групы і кампаненты
script-group-finding-objects = Пошук аб’ектаў
script-group-visibility-materials-tags = Бачнасць, матэрыялы, тэгі
script-group-solid-tools = Інструменты цвёрдых целаў
script-group-parameters-and-generators = Параметры і генератары
script-group-dimensions-and-notes = Памеры і нататкі
script-group-files-and-network = Файлы і сетка
script-group-output-and-numbers = Вывад і лікі

## Члены аб’ектаў, вектараў, масіваў, радкоў
script-member-kind = "face", "edge", "group", "component" …
script-member-valid = аб’ект яшчэ існуе
script-member-area = плошча грані, мм²
script-member-normal = нармаль грані (сусветная)
script-member-edges = рэбры грані
script-member-faces = грані рабра / суседнія грані грані
script-member-length = даўжыня рабра, мм
script-member-soft = рабро згладжанае
script-member-center = цэнтр межаў (сусветны)
script-member-vertices = кропкі грані / рабра (сусветныя)
script-member-name = назва групы / кампанента (можна прызначыць)
script-member-hidden = аб’ект схаваны (можна прызначыць)
script-member-bounds = #{ "{" } min, max, size, center { "}" } (сусветныя)
script-member-size = памер межаў па x, y, z
script-member-min = мінімальны кут межаў (сусветны)
script-member-max = максімальны кут межаў (сусветны)
script-member-definition = назва вызначэння кампанента
script-member-material = назва матэрыялу або ()
script-member-tag = назва тэга або ()
script-member-x = каардыната x (можна прызначыць)
script-member-y = каардыната y (можна прызначыць)
script-member-z = каардыната z (можна прызначыць)
script-member-length-2 = даўжыня вектара
script-member-normalize = адзінкавы вектар
script-member-dot = скалярны здабытак
script-member-cross = вектарны здабытак
script-member-distance = адлегласць да b
script-member-min-2 = мінімальны кут
script-member-max-2 = максімальны кут
script-member-size-2 = памер па x, y, z
script-member-center-2 = цэнтр
script-member-len = колькасць элементаў
script-member-is-empty = няма элементаў
script-member-push = дадаць элемент у канец
script-member-pop = выдаліць і вярнуць апошні элемент
script-member-insert = уставіць элемент
script-member-remove = выдаліць і вярнуць элемент
script-member-contains = масіў / радок змяшчае значэнне
script-member-index-of = індэкс значэння або -1
script-member-filter = элементы, для якіх замыканне ісціна
script-member-map = новы масіў вынікаў замыкання
script-member-reduce = звярнуць элементы
script-member-len-2 = колькасць сімвалаў
script-member-to-upper = вялікія літары (на месцы)
script-member-to-lower = малыя літары (на месцы)
script-member-trim = абрэзаць прабелы (на месцы)
script-member-split = разбіць на масіў
script-member-starts-with = радок пачынаецца з text
script-member-to-string = як тэкст

## Загалоўныя тэгі плагінаў
script-tag-name = назва ў меню і ў загалоўку дыялогу
script-tag-menu = шлях у меню «Плагіны»
script-tag-description = падказка і тэкст дыялогу
script-tag-param = значэнне, якое пытаюцца ў дыялогу перад запускам
script-tag-pick = N кропак, на якія націснулі перад запускам → points
script-tag-select = аб’екты, вылучаныя перад запускам → objects
script-tag-import = пункт Файл → Імпарт; файл у path
script-tag-export = пункт Файл → Экспарт; файл у path
script-tag-generator = генератар: робіць гэты файл генератарам (гл. docs/GENERATORS.md)
script-tag-formula = генератар: значэнне, вылічанае з параметраў, у p
script-tag-check = генератар: значэнні, для якіх яна ілжывая, адхіляюцца

## Тыпы параметраў (@param key:type)
script-param-type-length = даўжыня з адзінкамі ("1.2m"), у скрыпце мм — па змаўчанні для лікаў
script-param-type-int = цэлы лік
script-param-type-number = дробавы лік
script-param-type-text = тэкст — па змаўчанні для значэння ў двукоссі
script-param-type-bool = сцяжок — па змаўчанні для true / false
script-param-type-choice = генератар: адно з choice(a|b|c)
script-param-type-angle = генератар: градусы, паўзунок
script-param-type-percent = генератар: 0–100, паўзунок
script-param-type-material = генератар: назва матэрыялу дакумента

script-type-face = грань
script-type-edge = рабро
script-type-group = група
script-type-vector = вектар
script-type-num = лік
script-type-bool = bool
script-type-str = радок
script-type-bounds = межы
script-type-faces = масіў граней
script-type-edges = масіў рэбраў
script-type-groups = масіў груп
script-type-vecs = масіў кропак
script-type-array = масіў
script-type-any = аб’ект / вектар

## Тыпы параметраў без прыстаўкі «генератар: » (Даведнік па скрыптах, тэма пра генератар)
script-param-type-choice-plain = адно з choice(a|b|c)
script-param-type-angle-plain = градусы, паўзунок
script-param-type-percent-plain = 0–100, паўзунок
script-param-type-material-plain = назва матэрыялу дакумента

# Памылкі, якія бачыць аўтар скрыпта (API, генератары, параметрычныя рыгі)

## api.rs
script-err-k-expected-a-number-bool-or = { $k }: чакаўся лік, bool або радок, атрымана { $type_name }
script-err-set-params-unknown-key-k-x = set_params: невядомы ключ { $k } (x, y, z, count, axis, gap, stretch)
script-err-set-params-stretch-is-true-false = set_params: stretch — гэта true / false
script-err-set-params-not-parametric-call-make = set_params: не параметрычны (спачатку выклікайце make_parametric)
script-err-paint-no-material-n-create-it = paint: няма матэрыялу { $n } (стварыце яго праз material(name, colour))
script-err-paint-expected-a-material-name-got = paint: чакалася назва матэрыялу, атрымана { $t }
script-err-show-hide-groups-and-components-only = show / hide: толькі групы і кампаненты
script-err-outer-shell-solids-must-be-in = outer_shell: цвёрдыя целы павінны быць у адным кантэксце
script-err-group-entities-must-be-in-one = group: аб’екты павінны быць у адным кантэксце
script-err-scale-zero-factor = scale: нулявы каэфіцыент
script-err-rotate-zero-axis = rotate: нулявая вось
script-err-follow-me-the-path-must-be = follow_me: шлях павінен быць у кантэксце грані
script-err-note-not-added = note: не дададзена
script-err-dimension-not-added = dimension: не дададзена
script-err-what-sizes-must-be-positive-a = { $what }: памеры павінны быць дадатнымі (трубка тора танчэйшая за радыус), сегменты 3..1000
script-err-cylinder-no-face = cylinder: няма грані
script-err-cylinder-radius-and-height-must-be = cylinder: радыус і вышыня павінны быць ненулявымі, сегменты 3..1000
script-err-box-sizes-must-be-non-zero = box: памеры павінны быць ненулявымі
script-err-circle-radius-must-be-positive-segments = circle: радыус павінен быць дадатным, сегменты 3..1000
script-err-face-points-are-not-a-valid = face: кропкі не ўтвараюць правільны плоскі мнагавугольнік
script-err-soft-not-an-edge = soft: не рабро
script-err-faces-not-an-edge-or-face = faces: не рабро і не грань
script-err-length-not-an-edge = length: не рабро
script-err-time-limit-of-secs-s-exceeded = перавышана абмежаванне часу ў { $secs } с
script-err-both-solids-must-be-in-the = абодва цвёрдыя целы павінны быць у адным кантэксце
script-err-expected-a-colour-rrggbb-or-r = чакаўся колер: "#RRGGBB" або [r, g, b]
script-err-bad-colour-s-use-rrggbb = няправільны колер { $s } (выкарыстоўвайце #RRGGBB)
script-err-bad-colour-s = няправільны колер { $s }
script-err-curve-not-added = крывая не дададзена
script-err-degenerate-curve = выраджаная крывая
script-err-expected-edges-got-e = чакаліся рэбры, атрымана { $e }
script-err-expected-a-group-or-component-got = чакалася група або кампанент, атрымана { $e }
script-err-expected-a-face-got-e = чакалася грань, атрымана { $e }
script-err-entity-no-longer-exists-merged-split = аб’екта больш няма (аб’яднаны, падзелены або сцёрты)
script-err-context-no-longer-exists = кантэксту больш няма
script-err-expected-one-entity-got-len = чакаўся адзін аб’ект, атрымана { $len }
script-err-expected-an-entity-or-an-array = чакаўся аб’ект або масіў аб’ектаў, атрымана { $type_name }
script-err-expected-an-entity-got-did-the = чакаўся аб’ект, атрымана () (магчыма, аперацыя не ўдалася?)
script-err-no-group-component-named-name-here = тут няма групы / кампанента з імем `{ $name }`
script-err-expected-an-array-of-points = чакаўся масіў кропак
script-err-expected-a-point-vec-or-x = чакалася кропка (vec3 або [x, y, z]), атрымана { $type_name }
script-err-expected-a-number-got-type-name = чакаўся лік, атрымана { $type_name }
script-err-not-a-length-s = гэта не даўжыня: { $s }
script-err-no-model = няма мадэлі

## generator.rs
script-err-no-such-instance = няма такога асобніка
script-err-not-a-generated-component = не згенераваны кампанент
script-err-no-such-context = няма такога кантэксту
script-err-no-generator-name-in-the-document = у дакуменце няма генератара `{ $name }`
script-err-name-generators-nested-too-deep = { $name }: генератары ўкладзены занадта глыбока
script-err-name-check-expr-is-not-true = { $name }: праверка `{ $expr }` — не true / false
script-err-expected-a-number-got-t = чакаўся лік, атрымана { $t }

## json.rs
script-err-bad-entity-id-s = няправільны ідэнтыфікатар аб’екта { $s }

## rig.rs
script-err-part-has-no-parameter-key = у `{ $part }` няма параметра `{ $key }`
script-err-part-has-no-parameters = у `{ $part }` няма параметраў
script-err-no-part-part-in-the-component = у кампаненце няма часткі `{ $part }`
script-err-part-no-parameter-k-it-has = { $part }: няма параметра `{ $k }` (ёсць { $keys })
script-err-part-the-part-has-no-parameters = { $part }: у часткі няма параметраў (іх маюць толькі згенераваныя кампаненты і кампаненты з параметрамі)
script-err-part-show-hide-needs-a-yes = { $part }: для show / hide патрэбны параметр «так / не» або лікавы
script-err-part-choose-the-part-s-parameter = { $part }: выберыце параметр часткі
script-err-part-the-direction-is-zero = { $part }: напрамак нулявы
script-err-part-the-axis-has-no-direction = { $part }: у восі няма напрамку
script-err-part-choose-a-parameter-or-write = { $part }: выберыце параметр або напішыце выраз
script-err-a-joint-without-a-part = злучэнне без часткі
script-err-k-twice-parameters-and-formulas-share = `{ $k }` двойчы (параметры і формулы маюць агульныя імёны)
script-err-no-such-component = няма такога кампанента
script-err-this-component-is-built-by-a = гэты кампанент пабудаваны генератарам або параметрычнай групай
script-err-groups-can-t-have-parameters-make = у груп не можа быць параметраў: спачатку зрабіце яе кампанентам
script-err-not-a-component-with-parameters = гэта не кампанент з параметрамі
script-err-part-the-part-has-no-parameters-2 = { $part }: у часткі няма параметраў
script-err-part-no-such-part = { $part }: няма такой часткі
script-err-what-the-value-is-not-a = { $what }: значэнне — не лік, сцяг і не тэкст
script-err-what-the-value-is-not-a-2 = { $what }: значэнне — не назва матэрыялу
script-err-what-the-value-is-not-true = { $what }: значэнне — не true / false
script-err-what-the-value-is-not-a-3 = { $what }: значэнне — не лік

## ======== Паведамленні ядра ========

## curves.rs
kernel-err-the-curve-bounds-faces-outside-its = крывая ахінае грані па-за сваёй плоскасцю
kernel-err-other-edges-are-attached-to-the = да крывой далучаны іншыя рэбры
kernel-err-too-few-points = занадта мала кропак
kernel-err-segments-must-be-min-max-curve = колькасць сегментаў павінна быць { $min }..{ $MAX_CURVE_SEGMENTS }
kernel-err-radius-must-be-positive = радыус павінен быць дадатным
kernel-err-not-an-arc-circle-or-polygon = гэта не дуга, не круг і не мнагавугольнік

## chamfer.rs
kernel-err-the-edges-at-a-corner-are = рэбры ў куце калінеарныя
kernel-err-a-corner-of-the-edge-is = кут рабра не просты (занадта шмат рэбраў або граней)
kernel-err-the-chamfer-is-too-big-for = фаска занадта вялікая для рабра
kernel-err-the-chamfer-is-too-big-for-2 = фаска занадта вялікая для суседніх рэбраў
kernel-err-an-end-lies-along-the-edge = канец ляжыць уздоўж рабра
kernel-err-the-radius-must-be-positive = радыус павінен быць дадатным
kernel-err-the-distance-must-be-positive = адлегласць павінна быць дадатнай
kernel-err-the-faces-are-coplanar = грані капланарныя
kernel-err-broken-face-loop = парушаны контур грані
kernel-err-the-edge-is-too-short = рабро занадта кароткае
kernel-err-the-edge-must-be-shared-by = рабро павінна належаць роўна двум граням

## follow_me.rs
kernel-err-path-reverses = шлях разгортваецца назад
kernel-err-profile-is-parallel-to-the-path = профіль паралельны шляху
kernel-err-path-is-not-connected = шлях не звязны
kernel-err-path-branches = шлях разгалінаваны
kernel-err-empty-path = пусты шлях

## mod.rs
kernel-err-nothing-to-flip = няма чаго перавярнуць
kernel-err-flip-axis-is-zero = вось пераварочвання нулявая

## push_pull.rs
kernel-err-through-hole-cap-vertex-missing = скразны атвор: няма вяршыні накрыўкі
kernel-err-push-pull-would-collapse-geometry = выцягванне/уцягванне абрынула б геаметрыю
kernel-err-face-loop-without-edge = контур грані без рабра
kernel-err-distance-is-not-finite = адлегласць не з’яўляецца канечным лікам

## params.rs
kernel-err-empty-base = пустая аснова
kernel-err-size-must-be-positive = памер павінен быць дадатным
kernel-err-not-parametric = не параметрычны
kernel-err-empty-group = пустая група
kernel-err-already-parametric = ужо параметрычны

## spline.rs
kernel-err-the-curve-would-cross-itself = крывая перасякала б сябе
kernel-err-segments-must-be-max-curve-segments = колькасць сегментаў павінна быць 1..{ $MAX_CURVE_SEGMENTS }
kernel-err-not-a-curve-that-can-be = гэта крывая, якую нельга рэдагаваць

## stats.rs
kernel-err-an-empty-name = пустая назва

## text3d.rs
kernel-err-no-outlines = няма контураў

## generator.rs
kernel-gen-name-no-parameter-k-join = { $name }: няма параметра `{ $k }` ({ $join })
kernel-gen-empty-generator-name = пустая назва @generator
kernel-gen-not-a-generator-the-header-needs = гэта не генератар: у загалоўку патрэбна `// @generator Name`
kernel-gen-key-twice-formulas-and-parameters-share = `{ $key }` двойчы (формулы і параметры маюць агульныя імёны)
kernel-gen-key-bool-default-is-true-false = { $key }: значэнне bool па змаўчанні — true / false
kernel-gen-unknown-parameter-type-t-length-int = невядомы тып параметра `{ $t }` (length, int, number, bool, choice, angle, percent, material)
kernel-gen-key-choice-needs-options-choice-a = { $key }: для choice патрэбны варыянты: choice(a|b|c)
kernel-gen-bad-parameter-name-key = няправільная назва параметра `{ $key }`
kernel-gen-check-failed-expr = праверка не прайшла: { $expr }
kernel-gen-key-empty-formula = { $key }: пустая формула
kernel-gen-key-a-formula-is-length-int = { $key }: формула — length, int, number, bool, angle, percent або text, а не `{ $t }`
kernel-gen-bad-formula-name-key = няправільная назва формулы `{ $key }`
kernel-gen-key-bad-number-s-in-the = { $key }: няправільны лік `{ $s }` у дыяпазоне
kernel-gen-key-not-a-name-s = { $key }: не { $name }: { $s }
kernel-gen-key-s-is-not-one-of = { $key }: `{ $s }` — не адно з { $join }

## model.rs
kernel-err-the-generator-s-script-can-t = скрыпт генератара нельга скапіраваць
kernel-err-nothing-selected = нічога не вылучана

## solid.rs
kernel-err-pick-at-least-two-solids = выберыце не менш за два цвёрдыя целы
kernel-err-the-second-solid-has-a-degenerate = у другога цвёрдага цела выраджанае пераўтварэнне
kernel-err-the-solids-do-not-overlap = цвёрдыя целы не перасякаюцца
kernel-err-pick-two-different-solids = выберыце два розныя цвёрдыя целы
# $name — назва групы / кампанента
kernel-err-is-not-a-solid-every-edge = «{ $name }» — не цвёрдае цела (кожнае рабро павінна абмяжоўваць роўна дзве грані, укладзеных груп няма)

## OpError
kernel-err-entity-not-found = аб’ект не знойдзены
kernel-err-degenerate-geometry = выраджаная геаметрыя
# $why — прычына
kernel-err-operation-not-possible = аперацыя немагчымая: { $why }

## ======== Імпарт і экспарт ========

## dae.rs
kernel-io-not-a-collada-document = гэта не дакумент COLLADA
kernel-io-nodes-nested-too-deep = вузлы ўкладзены занадта глыбока

## dxf.rs
kernel-io-no-supported-entities-line-polyline-dface = няма падтрымоўваных аб’ектаў (LINE, POLYLINE, 3DFACE, CIRCLE, ARC)
kernel-io-not-a-dxf-file-no-section = гэта не файл DXF (няма SECTION)
kernel-io-not-a-dxf-file = гэта не файл DXF

## fbx.rs
kernel-io-fbx-description = FBX: { $description }

## gltf.rs
kernel-io-index-out-of-range = індэкс па-за межамі
kernel-io-no-meshes-in-the-gltf-scene = у сцэне glTF няма сетак
kernel-io-node-hierarchy-too-deep = іерархія вузлоў занадта глыбокая
kernel-io-draco-e = Draco: { $e }
kernel-io-draco-primitive-without-bufferview = прымітыў Draco без bufferView
kernel-io-accessor-out-of-range = accessor па-за межамі
kernel-io-unsupported-component-type = непадтрымоўваны тып кампанента
kernel-io-unsupported-accessor-type = непадтрымоўваны тып accessor
kernel-io-accessor-without-count = accessor без count
kernel-io-bufferview-out-of-range = bufferView па-за межамі
kernel-io-bufferview-without-bytelength = bufferView без byteLength
kernel-io-bad-buffer-index = няправільны індэкс буфера
kernel-io-buffer-without-data = буфер без даных
kernel-io-compressed-gltf-e-is-not-supported = сціснуты glTF ({ $e }) не падтрымліваецца
kernel-io-only-gltf-is-supported = падтрымліваецца толькі glTF 2.0
kernel-io-external-file-uri-needs-a-path = знешні файл { $uri } патрабуе шляху
kernel-io-bad-base-in-data-uri = няправільны base64 у data URI
kernel-io-not-a-gltf-file-e = гэта не файл glTF: { $e }
kernel-io-glb-without-json = GLB без JSON
kernel-io-glb-json-e = JSON у GLB: { $e }
kernel-io-truncated-glb-chunk = абрэзаны блок GLB

## meshes.rs
kernel-io-no-faces-in-the-file = у файле няма граней

## native.rs
kernel-io-unsupported-version-version = непадтрымоўваная версія { $version }
kernel-io-not-a-varstotas-material-format-format = гэта не матэрыял varstotas (format = { $format })
kernel-io-missing-root-definition = адсутнічае каранёвае вызначэнне
kernel-io-not-a-varstotas-file-format-format = гэта не файл varstotas (format = { $format })

## obj.rs
kernel-io-obj-line-ln-line = радок OBJ { $ln }: { $line }

## ply.rs
kernel-io-no-faces-a-point-cloud-can = няма граней (воблака кропак імпартаваць нельга)
kernel-io-list-too-long = спіс занадта доўгі
kernel-io-unknown-format-f = невядомы фармат { $f }
kernel-io-property-before-element = уласцівасць перад элементам
kernel-io-bad-element-count = няправільная колькасць элементаў
kernel-io-no-end-header = няма end_header
kernel-io-not-a-ply-file = гэта не файл PLY
kernel-io-truncated-data = даныя абрэзаны
kernel-io-unknown-type-s = невядомы тып { $s }

## points.rs
kernel-io-no-points-in-display = у { $display } няма кропак

## stl.rs
kernel-io-stl-vertex-count-not-a-multiple = колькасць вяршыняў STL не кратная 3
kernel-io-bad-stl-vertex-line = няправільная вяршыня STL: { $line }
kernel-io-not-a-valid-stl = гэта не правільны STL

## threemf.rs
kernel-io-mf-no-d-model-part = 3MF: няма часткі з 3D-мадэллю
kernel-io-mf-no-object-id = 3MF: няма аб’екта { $id }
kernel-io-mf-components-nested-too-deep = 3MF: кампаненты ўкладзены занадта глыбока
kernel-io-mf-no-part-path = 3MF: няма часткі { $path }

## xml.rs
kernel-io-xml-no-root-element = XML: няма каранёвага элемента
kernel-io-xml-unbalanced-tags = XML: тэгі не збалансаваны
kernel-io-xml-e = XML: { $e }

## zip.rs
kernel-io-name-zip-method-m-not-supported = { $name }: метад zip { $m } не падтрымліваецца
kernel-io-truncated-zip = абрэзаны zip
kernel-io-bad-zip-entry-name = няправільны запіс zip { $name }
kernel-io-bad-zip-directory = няправільны каталог zip

## Прэфіксы чытачоў COLLADA / PLY
# $msg — уласнае паведамленне чытача
kernel-io-collada = COLLADA: { $msg }
kernel-io-ply = PLY: { $msg }

## IoError
# $why — сістэмная памылка / паведамленне чытача
kernel-io-error-io = io: { $why }
kernel-io-error-format = format: { $why }

## File templates
template-files-heading = З файлаў шаблонаў
template-files-empty = У гэтай папцы няма шаблонаў .vrst
template-open-folder = Адкрыць папку шаблонаў…
template-folder-error = Не ўдалося прачытаць шаблоны: { $error }
template-created = Створана з шаблона: { $path }
template-load-failed = Не ўдалося загрузіць шаблон { $path }: { $error }
template-folder-label = Папка шаблонаў
template-folder-tip = Змясціце сюды мадэлі .vrst для стварэння новых дакументаў на іх аснове.

app-export-replace-title = Замяніць файлы экспарту?
app-export-replace-files =
    Экспарт заменіць наступныя файлы матэрыялаў або тэкстур:
    { $files }
    Працягнуць?
app-export-folder = Выберыце папку для экспарту { $format } і яго файлаў
app-export-replace-all =
    Экспарт заменіць наступныя файлы:
    { $files }
    Працягнуць?
app-import-missing-title = Не знойдзены файлы побач з мадэллю
app-import-missing =
    { $file } спасылаецца на файлы побач з сабой, якія Varstotas не змог прачытаць:
    { $files }
    Выбраць папку, у якой ляжыць мадэль, каб загрузіць іх? (Праграма ў пясочніцы, напрыклад Flatpak, бачыць толькі файл, выбраны ў дыялогу.)
app-import-not-in-folder = { $file } няма ў папцы { $folder }

## Object and tag locking
cmd-lock = Заблакіраваць
cmd-unlock = Разблакіраваць
cmd-unlock-all = Разблакіраваць усё
cmd-desc-lock = Забараніць вылучэнне і рэдагаванне выбраных аб’ектаў
cmd-desc-unlock-all = Разблакіраваць усе аб’екты і тэгі
edit-locked = Аперацыя змяняе заблакіраваныя аб’екты. Спачатку разблакіруйце іх.
script-member-locked = Блакіроўка рэдагавання з улікам тэга і бацькоўскіх аб’ектаў (толькі чытанне).

## Пачатковы экран і зваротная сувязь
start-menu = Пачатковы экран
start-subtitle = Стварыце мадэль, працягніце працу або паглядзіце прыклады.
start-create = Новая мадэль
start-new = Стварыць
start-open = Адкрыць дакумент…
start-guide = Кароткае кіраўніцтва
start-recent = Апошнія дакументы
start-no-recent = Тут з’явяцца нядаўна адкрытыя дакументы.
start-basics-title = Першыя крокі
start-basics = Прагал: вылучэнне · L: лінія · R: прамавугольнік. Уводзьце дакладныя памеры ў поле вымярэнняў. Кола мышы — маштаб, сярэдняя кнопка — паварот выгляду.
start-show = Паказваць пры запуску
start-continue = Працягнуць
report-menu = Паведаміць пра праблему…
report-intro = Праверце паведамленне і адкрыйце чарнавік у браўзеры. Для адпраўкі ўвайдзіце ў GitHub і адпраўце яго самастойна.
report-access = Для закрытага рэпазіторыя патрэбны доступ. Файлы мадэлі і здымкі экрана аўтаматычна не дадаюцца.
report-title = Кароткае апісанне
report-description = Што адбылося?
report-description-hint = Крокі ўзнаўлення, чаканы вынік і фактычныя паводзіны.
report-errors = Дадаць апошнія 10 папярэджанняў і памылак
report-review = Перад адпраўкай праверце тэкст ніжэй. У журнале могуць быць імёны файлаў і іншыя звесткі.
report-copy = Капіяваць паведамленне
report-github = Адкрыць чарнавік на GitHub
report-paste = Поўны тэкст скапіяваны. Устаўце яго ў апісанне issue на GitHub.

cli-usage =
    Выкарыстанне: Varstotas [--open] <document.vrst>
    Varstotas -- <document.vrst>
    Varstotas --help | --version
cli-one-document = Укажыце адзін дакумент
cli-unknown-option = Невядомы параметр: { $option }
cli-open-path = Пасля --open трэба ўказаць шлях да дакумента

prefs-save-backups = Захоўваць рэзервовую копію папярэдняй версіі
prefs-save-backups-note = Пры захаванні папярэдняя версія застаецца ў { DATA_DIR() }/backups як <name> (…).bak.vrst. Адключэнне не выдаляе існуючыя копіі.

model-info-metadata = Метаданыя дакумента
model-info-metadata-text = Тэкст / апісанне
model-info-license = Ліцэнзія
model-info-license-hint = Назва ліцэнзіі, спасылка або поўны тэкст (неабавязкова)
model-info-metadata-apply = Ужыць
model-info-metadata-reset = Скінуць змены
model-info-format-version = Версія фармату захавання: { $version }
model-info-metadata-edit = Змяніць метаданыя дакумента

model-info-tab-info = Інфа
model-info-tab-meta = Мета
model-info-tab-cleanup = Ачыстка

download-progress-unknown = Спампавана { $received } / агульны памер невядомы

# Practical step-by-step walkthrough.
tutorial-title = Тутарыял
tutorial-step = Крок { $current } / { $total }
tutorial-try = Паспрабуйце самі
tutorial-tools = Уключыць інструмент
tutorial-details = Падрабязней пра гэты крок
tutorial-practice-note = Пакіньце акно побач з мадэллю і выконвайце заданні ў працоўнай вобласці.
tutorial-back = Назад
tutorial-restart = З пачатку
tutorial-finish = Завяршыць
tutorial-next = Далей
tutorial-0-title = Знаёмства з Varstotas
tutorial-0-body = Мадэль складаецца з рэбраў і граняў, якія аб’ядноўваюцца ў групы і паўторна ўжывальныя кампаненты. Тут ёсць дакладнае чарчэнне, зменныя 3D-формы, матэрыялы, сцэны і лісты чарцяжоў. Пройдзем шлях ад новага дакумента да абмену мадэллю.
tutorial-0-task = Выберыце File → New. На прыкладзе простай стальніцы вывучым інструменты і падрыхтуем файл. Пераходзьце далей у сваім тэмпе.
tutorial-1-title = Агледзьце мадэль
tutorial-1-body = Перацягванне сярэдняй кнопкай мышы круціць від, з Shift — перамяшчае камеру. Кола набліжае кропку пад курсорам. Кіраванне трэкпадам і хуткасць мяняюцца ў наладах навігацыі.
tutorial-1-task = Паспрабуйце кручэнне і панараміраванне. Пераключыце Top і Iso ў Camera → Standard Views, затым ужывіце Zoom Extents.
tutorial-2-title = Намалюйце дакладную аснову
tutorial-2-body = Замкнёны плоскі контур рэбраў утварае грань. Rectangle і Circle ствараюць яе адразу. Поле Measurements прымае дакладныя памеры падчас працы інструмента; Падказкі тлумачыць бягучы этап.
tutorial-2-task = Уключыце Rectangle. Пакажыце першы вугал, ссуньце паказальнік, увядзіце ў Measurements 1400mm,800mm і націсніце Enter.
tutorial-3-title = Дадайце аб’ём
tutorial-3-body = Push/Pull выцягвае грань у аб’ём. Draw → Shapes стварае бокс, цыліндр, конус, піраміду, сферу ці тор за адну аперацыю. Гэтыя кампаненты маюць зменныя параметры генератара ў Уласцівасці.
tutorial-3-task = Выцягніце прамавугольнік з Push/Pull і задайце таўшчыню 40. Побач паспрабуйце Box: першы вугал, супрацьлеглы вугал асновы, затым вышыня.
tutorial-4-title = Выкарыстоўвайце прывязкі
tutorial-4-body = Курсор адзначае канцы, сярэдзіны, восі і вузлы сеткі. Каляровыя лініі дапамагаюць трымаць кірунак. Стрэлкі фіксуюць вось, Shift — бягучую прывязку. Меню магніта кіруе іх тыпамі.
tutorial-4-task = Уключыце сетку і Line. Ад вугла вядзіце ўздоўж восі да наступнай лініі сеткі. Паўтарыце з асновай і вышынёй Box. Escape скасоўвае незавершаную аперацыю.
tutorial-5-title = Рэдагуйце і паўтарайце
tutorial-5-body = Move, Rotate і Scale мяняюць вылучэнне; Undo дапамагае эксперыментаваць. Група аддзяляе геаметрыю ад суседніх рэбраў. Копіі кампанента маюць агульнае азначэнне: змяненне адной абнаўляе астатнія.
tutorial-5-task = Вылучыце стальніцу і стварыце групу праз Edit. Перамясціце яе, адмяніце перамяшчэнне і двойчы пстрыкніце для рэдагавання ўнутры. Пасля выйдзіце з групы.
tutorial-6-title = Навядзіце парадак
tutorial-6-body = Outliner паказвае ўкладзеныя групы і кампаненты. Тэгі кіруюць бачнасцю і блакіроўкай рэдагавання. Уласцівасці паказвае параметры вылучэння. Хаванне і блакіроўка незалежныя.
tutorial-6-task = Назавіце стальніцу ў Уласцівасці. Стварыце і прызначце тэг у Tags, пераключыце бачнасць. Паспрабуйце замок і разблакіруйце аб’ект перад афарбоўкай.
tutorial-7-title = Матэрыялы і стылі
tutorial-7-body = Paint прызначае колер і тэкстуру, Texture Position мяняе іх размяшчэнне. Online Materials спампоўвае матэрыялы. Shaded with Textures зручны для мадэлявання, Monochrome адрознівае бакі граняў, Realistic ужывае фізічныя ўласцівасці і святло.
tutorial-7-task = Афарбуйце грань. Параўнайце Shaded with Textures, Monochrome і Realistic. Уключыце X-Ray для схаваных рэбраў.
tutorial-8-title = Пакажыце вынік
tutorial-8-body = Сцэны запамінаюць віды і параметры паказу. Памеры падпісваюць мадэль, сячэнні паказваюць нутро, лісты размяшчаюць віды на паперы. Render стварае выніковую выяву, анімацыя пераходзіць паміж сцэнамі.
tutorial-8-task = Дадайце сцэну Iso і другую з відам Top. Пераключыцеся паміж імі, затым паспрабуйце Dimension або Section.
tutorial-9-title = Захавайце і падзяліцеся
tutorial-9-body = .vrst захоўвае рэдагуемую геаметрыю, матэрыялы, сцэны і метаданыя. Звесткі пра дакумент → Meta змяшчае апісанне, ліцэнзію і прэв’ю. Імпарт і экспарт падтрымліваюць OBJ, STL, GLB ды іншыя фарматы; скрыпты і плагіны пашыраюць інструменты.
tutorial-9-task = Захавайце мадэль. У Звесткі пра дакумент → Meta дадайце апісанне і ліцэнзію, захавайце зноў. У Help → Examples ёсць прыклады; любы крок можна паўтарыць.

app-reversed-faces = Перавернута граняў: { $count }. Рэжым Monochrome паказвае пярэднія і адваротныя бакі.

download-progress-previews = Спампоўванне прэв’ю: { $count }

prefs-trackpad-enabled = Навігацыя тачпадам / плаўнай пракруткай
prefs-gesture-speed = Адчувальнасць
prefs-invert-swipe = Інвертаваць рух двух пальцаў
prefs-pinch-enabled = Маштабаванне шчыпком
prefs-invert-pinch = Інвертаваць шчыпок
prefs-twist = Паварот двума пальцамі
prefs-twist-off = Выключаны
prefs-twist-orbit = Круціць вакол вертыкалі
prefs-twist-roll = Нахіляць від
prefs-invert-twist = Інвертаваць паварот
prefs-trackpad-platform = Падтрымка шчыпка і павароту залежыць ад АС і драйвера. Гэтыя налады таксама дзейнічаюць на прылады з плаўнай пракруткай.

view-triangles = Трохвугольнікі
view-triangles-hint = Паказаць усе рэбры трыянгуляцыі паверх бягучага стылю, у тым ліку мяккія рэбры і дыяганалі палігонаў.

# 3D Printing: basic surface checks and conservative repair.
menu-3d-printing = 3D Printing
print-title = Падрыхтоўка да 3D-друку
print-check-command = Праверыць мадэль…
print-repair-command = Рамонт сеткі…
print-check-description = Праверыць выбраныя аб’екты або бачную мадэль на асноўныя дэфекты сеткі для друку.
print-repair-description = Выправіць швы, дублікаты, арыентацыю і невялікія плоскія адтуліны.
print-scope = Вобласць:
print-visible = Бачная мадэль
print-selection = Вылучэнне
print-tab-check = Праверка
print-tab-repair = Рамонт
print-working = Апрацоўваецца копія мадэлі…
print-worker-failed = Не ўдалося завяршыць праверку. Запусціце яе зноў.
print-result-stale = Мадэль або вылучэнне змяніліся. Рамонт не ўжыты; паўтарыце праверку.
print-repair-undo = Рамонт для 3D-друку
print-stale = Справаздача састарэла. Запусціце праверку зноў.
print-repair-weld = Злучыць супадальныя вяршыні
print-repair-degenerate = Выдаліць выраджаныя грані
print-repair-duplicates = Выдаліць грані-дублікаты
print-repair-orient = Узгадніць арыентацыю граней
print-repair-shells = Разгарнуць замкнёныя абалонкі з улікам поласцей
print-repair-holes = Закрыць невялікія плоскія адтуліны
print-hole-size = Максімальны памер адтуліны:
print-repair-note = Рамонт можна адмяніць. Заблакаваная, схаваная і працэдурная геаметрыя захоўваецца. Складаныя і ўкладзеныя контуры адтулін застаюцца для ручнога рамонту.
print-repair-whole-objects = Для рамонту выберыце групы/кампаненты цалкам або пераключыцеся на «Бачная мадэль».
print-repair-run = Выправіць і праверыць
print-repaired = Злучана вяршынь: { $welded }; выдалена граней: { $removed }; разгорнута граней: { $reversed }; закрыта адтулін: { $holes }; прапушчана аб’ектаў: { $skipped }.
print-empty = У гэтай вобласці няма бачных граней. Выберыце аб’ект або «Бачная мадэль».
print-counts = Граней: { $faces } · трохвугольнікаў: { $triangles } · абалонак: { $shells } (замкнёных: { $closed })
print-size = Памер: { $x } × { $y } × { $z } мм
print-area = Плошча паверхні: { $area } мм²
print-volume = Замкнёны аб’ём: { $volume } мм³
print-found-problems = Знойдзены праблемы паверхні
print-basic-pass = Базавая праверка паверхні пройдзена
print-limitations = Таўшчыня ацэньваецца па пробах. Перасячэнні, навісанні і межы прынтара — на суседніх укладках. Праверце слаі ў слайсеры.
print-tolerance = Допуск супадзення вяршынь:
print-highlight = Падсвечваць праблемы ў мадэлі
print-check-run = Праверыць
print-copy = Капіяваць справаздачу
print-open-edges = Адкрытыя гранічныя рэбры
print-nonmanifold = Рэбры з больш чым дзвюма гранямі
print-winding = Няўзгодненая арыентацыя рэбраў
print-degenerate = Выраджаныя трохвугольнікі / пустыя грані
print-duplicates = Трохвугольнікі-дублікаты
print-inverted = Вывернутыя замкнёныя абалонкі
print-wire = Асобныя рэбры (не друкуюцца)
print-export-visible = Экспарт бачнай мадэлі

# Printing preparation tools
print-analysis = Аналіз
print-bed = Прынтар і стол
print-split = Разразанне і злучальнікі
print-export = Экспарт для друку
print-export-stale = Мадэль або налады змяніліся. Паўтарыце экспарт перад адкрыццём слайсера.
print-cut-undo = Разразанне для друку
print-place-undo = Размяшчэнне на друкавальным стале
print-placement-done = Размяшчэнне абноўлена. Абнавіце праверку.
print-cut-select = Для разразання выберыце адну групу ці кампанент цалкам.
print-cut-done = Створана частак: { $parts }, у тым ліку асобных штыфтоў: { $dowels }. Размясціце іх на стале.
print-export-extension = Выберыце .3mf ці .stl.
print-export-done = Экспартавана частак: { $parts } у { $path }
print-slicer-opened = Экспартаваны файл адкрыты ў праграме па змаўчанні для яго.
print-min-wall = Мінімальная таўшчыня сценкі
print-two-nozzles = Два дыяметры сопла
print-sample-budget = Максімум проб таўшчыні
print-pair-budget = Максімум пар для праверкі перасячэнняў
print-overhang-angle = Вугал навісання ад вертыкалі ўніз
print-map = Карта паверхні
print-map-none = Няма
print-map-wall = Таўшчыня сценак
print-map-overhang = Навісанні
print-intersection-count = Пар трохвугольнікаў з перасячэннем: { $n }; праверана пар: { $pairs }
print-intersection-incomplete = Дасягнуты ліміт пар: праверка перасячэнняў не завершана.
print-wall-coverage = Проб таўшчыні: { $samples } / { $possible }; трохвугольнікаў: { $triangles } / { $total }
print-wall-result = Мінімум па пробах: { $minimum } мм; тонкіх: { $thin }; невядомых: { $unknown }
print-overhang-result = Плошча навісанняў: { $area } мм² (трохвугольнікаў: { $n })
print-wall-unreliable = Адкрытыя, перакрытыя ці няправільна арыентаваныя паверхні зніжаюць дакладнасць вымярэнняў.
print-wall-estimate = Таўшчыня ацэньваецца прамянямі ўнутр, да чатырох проб на трохвугольнік. Дробныя дэталі паміж пробамі могуць быць прапушчаны. Шэры — выхад не знойдзены, чырвоны — ніжэй парога. Кантакт са сталом выключаны з навісанняў. Праверце слаі ў слайсеры.
print-profile-name = Профіль прынтара
print-bed-width = Шырыня стала X
print-bed-depth = Глыбіня стала Y
print-bed-height = Вышыня друку Z
print-bed-margin = Адступ ад краю стала
print-bed-gap = Зазор паміж дэталямі
print-nozzle = Дыяметр сопла
print-arrange-rotate = Дазволіць паварот на 90° пры размяшчэнні
print-bed-show = Паказваць аб’ём друку ў сцэне
print-drop = Апусціць на стол
print-center = Па цэнтры стала
print-arrange = Раскласці дэталі
print-lay-face = Пакласці выбранай гранню
print-placement-note = Размяшчэнне захоўвае маштаб і працуе з цэлымі дэталямі. Укладка выкарыстоўвае габарыты: складаныя формы часам трэба размясціць уручную. Папярэджанні пра перакрыцце адносяцца да габарытаў.
print-bed-result = Дэталяў: { $parts }; па-за межамі: { $outside }; па-за плоскасцю стала: { $floating }; перакрыццяў габарытаў: { $overlaps }
print-fits = Змяшчаецца
print-outside = Па-за аб’ёмам друку
print-cut-normal = Нармаль плоскасці разрэзу
print-cut-offset = Зрушэнне плоскасці ад пачатку каардынат
print-connectors = Дадаць гнёзды і асобныя штыфты
print-dowel-count = Колькасць штыфтоў
print-dowel-diameter = Дыяметр штыфта
print-dowel-length = Даўжыня штыфта
print-dowel-clearance = Радыяльны і восевы зазор
print-dowel-wall = Мінімальная сценка вакол гнязда
print-dowel-note = Разразанне патрабуе замкнутых дэталяў з узгодненай арыентацыяй. Гнёзды размяшчаюцца ўздоўж разрэзу з праверкай навакольнага матэрыялу. Палавіны і асобныя штыфты застаюцца ў становішчы зборкі: затым раскладзіце іх. Падбярыце зазор для свайго прынтара.
print-cut-run = Разрэзаць і закрыць сячэнні
print-cut-to-bed = Разрэзаць па памерах прынтара
print-cut-to-bed-note = Аўтаматычныя разрэзы ўлічваюць карысны аб’ём X/Y/Z, да 256 дэталяў. Часткі закрываюцца без злучальнікаў і захоўваюць становішча зборкі.
print-export-scope-note = Экспарт выкарыстоўвае выбраную вобласць і захоўвае сусветныя каардынаты ў міліметрах. 3MF захоўвае асобныя аб’екты; STL змяшчае трохвугольнікі.
print-export-materials-note = 3MF уключае базавыя колеры, імёны, апісанне, ліцэнзію і крыніцы. Тэкстуры PBR не экспартуюцца. STL змяшчае толькі геаметрыю.
print-export-3mf = Экспарт 3MF…
print-export-stl = Экспарт STL…
print-slicer-open = Адкрыць у слайсеры
print-slicer-note = Экспарт адкрываецца ў праграме, прызначанай у сістэме для .3mf / .stl: зрабіце слайсер праграмай па змаўчанні для гэтых файлаў. Пасля змен мадэлі паўтарыце экспарт.
print-export-missing = Экспартаваны файл адсутнічае. Паўтарыце экспарт.

# Printing operation errors
print-error-1 = Некарэктнае пераўтварэнне бацькоўскага аб’екта
print-error-2 = У гэтай вобласці няма бачных граняў для друку
print-error-3 = Некарэктныя памеры прынтара ці адступ
print-error-4 = Заблакаваныя і працэдурныя аб’екты нельга змяняць
print-error-5 = Дэталь змяшчае заблакаваную геаметрыю
print-error-6 = Перад разразаннем пакажыце ўсю геаметрыю дэталі
print-error-7 = Дэталь змяшчае заблакаваныя рэбры
print-error-8 = Дэталь змяшчае заблакаваныя аб’екты
print-error-9 = Выберыце дэталі цалкам
print-error-10 = Для размяшчэння і разразання выберыце групы ці кампаненты цалкам
print-error-11 = Згрупуйце свабодную геаметрыю перад размяшчэннем, калі іншыя каранёвыя грані схаваны
print-error-12 = Няма бачных дэталяў
print-error-13 = Дэталь не мае паверхні
print-error-14 = Незваротнае пераўтварэнне бацькоўскага аб’екта
print-error-15 = Некарэктныя памеры прынтара
print-error-16 = Няма дэталяў
print-error-17 = Дэталі не змяшчаюцца на адным стале ў зыходным маштабе: выберыце менш дэталяў ці разрэжце іх
print-error-18 = Выроджаная грань
print-error-19 = Выбраная грань заглыблена: частка геаметрыі апынецца пад сталом
print-error-20 = Неадназначная мяжа сячэння: выпраўце дэталь перад разразаннем
print-error-21 = Адкрытая мяжа сячэння
print-error-22 = Некарэктная мяжа сячэння
print-error-23 = Для адтуліны ў сячэнні не знойдзены знешні контур
print-error-24 = Пасля закрыцця сячэння паверхня не замкнутая: спачатку выпраўце зыходную сетку
print-error-25 = Плоскасць не падзяляе гэтую дэталь
print-error-26 = Некарэктная плоскасць разрэзу
print-error-27 = Для разразання выберыце роўна адну дэталь цалкам
print-error-28 = Перад разразаннем выпраўце адкрытыя, дубляваныя і няправільна арыентаваныя паверхні
print-error-29 = Разразанне патрабуе поўнай праверкі без знойдзеных перасячэнняў паверхняў
print-error-30 = Выберыце адну дэталь цалкам
print-error-31 = Спатрэбіцца больш за 256 частак: павялічце профіль ці разразайце паэтапна
print-error-32 = Некарэктныя памеры, колькасць штыфтоў ці зазор
print-error-33 = Не знойдзена сячэнне для злучальнікаў
print-error-34 = Злучальнікі занадта блізка: паменшыце колькасць ці дыяметр
print-error-35 = Злучальнік выходзіць за межы дэталі ці недастаткова сценкі/глыбіні. Паменшыце памеры або колькасць, змяніце плоскасць ці выберыце больш тоўстую вобласць
print-error-36 = Некарэктнае гняздо
print-error-37 = Не атрымалася выразаць гняздо
print-error-38 = Некарэктны штыфт
print-bed-frame = Паказаць увесь аб’ём друку
print-part-name = Дэталь для друку
print-socket-name = Выразанне гнязда
print-dowel-name = Зборачны штыфт

print-go-check = Перайсці да праверкі
print-analysis-run = Запусціць аналіз
print-bed-refresh = Абнавіць праверку стала

# Window → Physics
physics-title = Фізіка
physics-drop = Падзенне
physics-drop-hint = Вылучаныя групы і кампаненты падаюць і супакойваюцца на ўсім астатнім, што бачна.
physics-shape = Форма
physics-shape-hull = Абалонка
physics-shape-hull-hint = Выпуклая абалонка кожнага аб'екта: хутка і ўстойліва, западзіны запаўняюцца.
physics-shape-pieces = Часткі
physics-shape-pieces-hint = Выпуклыя часткі кожнага аб'екта: у чашу можна нешта пакласці, падрыхтоўка даўжэй.
physics-friction = Трэнне
physics-bounce = Пругкасць
physics-gravity = Цяжар
physics-time = Мяжа часу
physics-floor = Падлога на Z = 0
physics-no-bodies = Вылучыце групы або кампаненты, якія павінны ўпасці.
physics-bodies = Аб'ектаў для падзення: { $n }
physics-run = Запусціць
physics-running = Сімуляцыя… { $time } с
physics-done = Супакоіліся за { $time } с
physics-stopped = Спынена на { $time } с
physics-fell = Аб'ектаў без апоры: { $n } (яны застаюцца на месцы)
physics-apply = Ужыць
physics-again = Запусціць зноў
physics-stop = Стоп
physics-cancel = Адмена
physics-interrupted = Мадэль змянілася: сімуляцыя спынена.
physics-failed = Сімуляцыя не ўдалася: { $error }
physics-undo = Падзенне
physics-applied = Перамешчана аб'ектаў: { $n }
physics-cloth = Тканіна
physics-cloth-hint = Вылучаная грань (або група з адной грані) падае як тканіна і аблягае ўсё астатняе, што бачна. Яна становіцца групай гладкіх трохвугольнікаў.
physics-cloth-no-face = Вылучыце адну грань або групу з адной грані.
physics-cloth-too-small = Грань меншая за крок.
physics-cloth-too-fine = Пунктаў: { $n } — павялічце крок.
physics-cloth-particles = Пунктаў тканіны: { $n }
physics-cloth-spacing = Крок
physics-cloth-spacing-hint = Адлегласць паміж пунктамі тканіны: чым меншая, тым драбнейшыя складкі і даўжэйшы разлік.
physics-cloth-stiffness = Жорсткасць
physics-cloth-stiffness-hint = Наколькі тканіна супраціўляецца расцяжэнню.
physics-cloth-bending = Выгін
physics-cloth-bending-hint = Наколькі тканіна супраціўляецца згібу: мала — шоўк, многа — скура.
physics-cloth-thickness = Таўшчыня
physics-cloth-thickness-hint = Зазор паміж тканінай і тым, на чым яна ляжыць (не меней за дзясятую частку кроку).
physics-cloth-self = Не праходзіць праз сябе
physics-cloth-self-hint = Складкі сутыкаюцца адна з адной (павольней).
physics-cloth-undo = Тканіна
physics-cloth-applied = Тканіна стала новай групай.
