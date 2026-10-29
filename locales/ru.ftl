# Тексты интерфейса Varstotas: русский.
# Разделы соответствуют областям ключей (первое слово ключа). См. docs/LOCALIZATION.md.

## ======== Общее ========

language-name = Русский

# Системный файловый менеджер, для пунктов «Показать в …» / «Открыть папку …».
file-manager = файловый менеджер
    .macos = Finder
    .windows = Проводник

# Показать файл в системном файловом менеджере.
reveal-in-file-manager = Показать в файловом менеджере
    .macos = Показать в Finder
    .windows = Показать в Проводнике

ok = OK
cancel = Отмена
close = Закрыть
apply = Применить
delete = Удалить

## ======== Приложение ========

app-edit-dimension-text = Правка текста размера
app-dimension-text-hint = Пусто возвращает измеренную длину; `<>` обозначает её.
# { $step } — шаг сетки с единицей измерения.
app-grid-step = Шаг сетки { $step }
# { $input } — введённый текст.
app-invalid-grid-step = Неверный шаг сетки: { $input }
app-invalid-value = Неверное значение: { $input }
app-undo-move-axes = Перемещение осей
app-no-credits = В этой модели нет загруженных компонентов с указанием авторства
# { $name } — название отменяемого действия.
app-flash-undo = Отмена: { $name }
app-nothing-to-undo = Нечего отменять
app-flash-redo = Повтор: { $name }
app-nothing-to-redo = Нечего повторять
app-select-to-group = Сначала выделите, что сгруппировать
app-undo-make-group = Сгруппировать
app-undo-make-component = Создать компонент
app-select-to-explode = Выделите группу или компонент, чтобы разбить
app-undo-explode = Разбить
app-undo-hide = Скрыть
app-undo-unhide-all = Показать все
# { $name } — название инструмента.
app-not-implemented = { $name }: пока не реализовано
app-undo-delete = Удалить
app-cant-move-into-itself = Нельзя переместить группу или компонент внутрь самих себя
app-undo-move-out-of-group = Вынести из группы
app-undo-move-into-group = Переместить в группу
app-undo-add-scene = Добавить сцену
app-undo-update-scene = Обновить сцену
app-undo-delete-scene = Удалить сцену
app-undo-move-scene = Переместить сцену
app-undo-cut = Вырезать
app-no-guides = Направляющих нет
app-snapping-on = Привязки включены
app-snapping-off = Привязки выключены
app-select-one-component = Выделите один компонент
app-undo-make-unique = Сделать уникальным
# { $count } — число плагинов.
app-plugins-loaded = { $count ->
        [one] Загружен { $count } плагин
        [few] Загружено { $count } плагина
        [many] Загружено { $count } плагинов
       *[other] Загружено { $count } плагина
    }
app-select-one-group-or-component = Выделите одну группу или один компонент
app-undo-component-parameters = Параметры компонента
app-undo-make-parametric = Сделать параметрическим
app-undo-active-cut = Активный разрез
app-undo-reverse-section = Обратить сечение
app-undo-soften-edges = Сглаживание рёбер
app-select-faces-to-reverse = Выделите грани, чтобы перевернуть
app-undo-reverse-faces = Перевернуть грани
app-select-face-to-orient = Выделите грань, по которой сориентировать соседние
app-undo-orient-faces = Ориентировать грани
app-faces-agree = Все связанные грани уже ориентированы одинаково
# { $count } — число граней.
app-faces-reversed = { $count ->
        [one] Перевёрнута { $count } грань
        [few] Перевёрнуты { $count } грани
        [many] Перевёрнуто { $count } граней
       *[other] Перевёрнуто { $count } грани
    }
app-move-axes-hint = Смещение по красной, зелёной, синей оси и необязательный поворот вокруг синей: `dx, dy, dz[, угол]`.
app-undo-reset-axes = Сбросить оси
app-select-to-flip = Выделите, что отразить
app-undo-flip-red = Отразить по красной
app-undo-flip-green = Отразить по зелёной
app-undo-flip-blue = Отразить по синей
app-select-to-intersect = Выделите грани, группы или компоненты для пересечения
app-undo-intersect-model = Пересечь с моделью
app-undo-intersect-selection = Пересечь с выделением
app-undo-intersect-context = Пересечь с контекстом
app-no-intersections = Пересечений не найдено
# { $count } — число рёбер.
app-intersections-created = { $count ->
        [one] Создано { $count } ребро пересечения
        [few] Создано { $count } ребра пересечения
        [many] Создано { $count } рёбер пересечения
       *[other] Создано { $count } ребра пересечения
    }
app-nothing-to-copy = Нечего копировать: ничего не выделено
# { $error } — текст ошибки.
app-copy-failed = Не удалось скопировать: { $error }
app-text-no-characters = В тексте нет видимых символов
app-clipboard-empty = Буфер обмена пуст
# { $factor } — коэффициент масштаба.
app-resized = Размер изменён ×{ $factor }
app-scene-update = Обновить
app-scene-add = Добавить
app-scene-delete = Удалить
app-scene-move-left = Влево
app-scene-move-right = Вправо
app-pause-animation = Пауза анимации
app-play-animation = Воспроизвести анимацию
# { $path } — путь к файлу, { $error } — текст ошибки.
app-cant-use = Не удалось использовать { $path }: { $error }
app-undo-add-material = Добавить материал
app-dropped-material = Добавлен материал { $name }: щёлкните по грани, чтобы окрасить её
app-no-scenes = Нет сцен для показа. Добавьте сцены в панели «Сцены».
# { $path } — путь к файлу.
app-opened = Открыт { $path }
app-opened-backup = Резервная копия { $path } открыта как новый документ: при сохранении будет спрошено, куда
app-could-not-open = Не удалось открыть { $path }: { $error }
app-saved = Сохранён { $path }
app-save-failed = Не удалось сохранить: { $error }
# { $warning } — текст предупреждения.
app-exported-warning = Экспортирован { $path }. { $warning }
app-exported = Экспортирован { $path }
app-export-failed = Не удалось экспортировать: { $error }
app-write-denied-windows = Windows запретил доступ. Если это «Документы», «Изображения» или «Рабочий стол», Varstotas может блокировать контролируемый доступ к папкам Defender: Безопасность Windows → Защита от вирусов и угроз → Защита от программ-шантажистов → Разрешить работу приложения через контролируемый доступ к папкам.
# { $format } — название формата.
app-undo-import = Импорт { $format }
app-imported = Импортирован { $path }
app-autosave-failed = Не удалось выполнить автосохранение в { $path }: { $error }
# { $name } — имя файла.
app-recovered = { $name } восстановлен из автосохранения
app-could-not-recover = Не удалось восстановить { $name }: { $error }

## Фильтры диалогов файлов

app-filter-image = Изображение
app-filter-script = Скрипт
app-filter-model = Модель Varstotas
app-filter-backup = Резервная копия Varstotas (.vrst.bak)

## ======== Меню ========

menu-intersect-faces = Пересечь грани
menu-flip-along = Отразить по
menu-axes-place = Установить
menu-axes-move = Переместить…
menu-axes-reset = Сбросить
menu-active-cut = Активный разрез
menu-reverse = Обратить
menu-align-view = Выровнять вид
menu-unfold = Развёртка…
menu-texture = Текстура
menu-texture-position = Положение
menu-uv-editor = UV-редактор…
menu-mark-seam = Отметить как шов
menu-clear-seam = Снять шов
menu-file = Файл
menu-new-from-template = Новый из шаблона
menu-open-recent = Открыть недавние
menu-preview-image = Изображение предпросмотра
menu-preview-image-hint = Картинка, которую библиотека компонентов показывает для этого файла (иначе рисуется модель)
menu-import = Импорт
menu-export = Экспорт
menu-unfold-export = Развёртка (SVG / PDF / DXF)…
menu-edit = Правка
menu-view = Вид
menu-face-style = Стиль граней
menu-shaded = С заливкой
menu-shaded-textures = С текстурами
menu-monochrome = Монохром
menu-hidden-line = Скрытые линии
menu-wireframe = Каркас
menu-realistic = Реалистичный
menu-x-ray = Рентген
menu-edge-style = Стиль рёбер
menu-edges = Рёбра
menu-edges-hint = Рёбра в этом стиле граней (у стиля «Реалистичный» свой переключатель)
menu-profiles = Контуры
menu-guides = Направляющие
menu-axes = Оси
menu-ground = Земля
menu-sky = Небо
menu-sky-hint = Градиент неба за моделью; выключено — простой светлый фон
menu-grid = Сетка
menu-shadows = Тени
menu-section-planes = Секущие плоскости
menu-section-cuts = Разрезы сечением
menu-section-fill = Заливка сечений
menu-viewports = Видовые экраны
menu-viewports-one = Один
menu-viewports-two = Два (рядом)
menu-viewports-four = Четыре (2 × 2)
menu-presets = Пресеты
menu-animation = Анимация
menu-pause = Пауза
menu-play = Воспроизвести
menu-next-scene = Следующая сцена
menu-previous-scene = Предыдущая сцена
menu-add-scene = Добавить сцену
menu-camera = Камера
menu-standard-views = Стандартные виды
menu-parallel-projection = Параллельная проекция
menu-perspective = Перспектива
menu-draw = Рисование
menu-shapes = Фигуры
menu-tools = Инструменты
menu-solid-tools = Твердотельные операции
menu-snapping = Привязки
menu-extensions = Плагины
menu-window = Окно
menu-side-bar = Боковая панель
menu-sheets = Листы…
menu-help = Справка
menu-user-guide = Руководство
menu-user-guide-hint = Как работать с Varstotas: инструменты, меню, файлы, клавиши (F1 над моделью: активный инструмент)
menu-examples = Примеры
menu-examples-hint = Открыть примеры в репозитории в браузере
menu-script-reference = Справочник по скриптам
menu-script-reference-hint = Скрипты, генераторы и скрипты компонентов: функции, свойства, примеры
menu-licences = Лицензии…
menu-licences-hint = Лицензия Varstotas, компоненты с открытым кодом, форматы файлов и онлайн-библиотеки с их условиями
menu-about = О программе Varstotas
menu-increments = Шаги
menu-all-on = Все включить
menu-all-off = Все выключить
menu-show-grid = Показать сетку
menu-increments-move = Перемещение
menu-increments-rotate = Поворот
menu-increments-scale = Масштаб
menu-erase = Стереть
# { $name } — название последнего изменения.
menu-undo-named = Отменить: { $name }
menu-redo-named = Повторить: { $name }

## ======== Команды ========

cmd-new = Создать
cmd-open = Открыть…
cmd-save = Сохранить
cmd-save-as = Сохранить как…
cmd-preview-show = Показать…
cmd-preview-from-file = Из файла изображения…
cmd-preview-from-view = Из текущего вида
cmd-preview-remove = Удалить
cmd-import-obj = OBJ…
cmd-import-stl = STL…
cmd-import-dxf = DXF…
cmd-import-gltf = glTF / GLB…
cmd-mf = 3MF…
cmd-import-dae = COLLADA (DAE)…
cmd-import-fbx = FBX…
cmd-import-ply = PLY…
cmd-import-csv = Точки (CSV)…
cmd-export-image = 2D-изображение (PNG)…
cmd-export-animation = Анимация (GIF / PNG)…
cmd-export-glb = glTF (GLB)…
cmd-export-drawing = 2D-чертёж (SVG / PDF)…
cmd-sheets = Листы (PDF / SVG)…
cmd-export-report = Отчёт: детали и площади (CSV)…
cmd-export-faces-json = Грани (JSON)…
cmd-export-attributions = Авторство (кредиты загруженных моделей)…
cmd-run-script = Запустить скрипт…
cmd-quit = Выход
cmd-undo = Отменить
cmd-redo = Повторить
cmd-cut = Вырезать
cmd-copy = Копировать
cmd-paste = Вставить
cmd-paste-in-place = Вставить на место
cmd-delete = Удалить
cmd-purge-unused = Удалить неиспользуемое
cmd-delete-guides = Удалить направляющие
cmd-select-all = Выделить всё
cmd-select-none = Снять выделение
cmd-hide = Скрыть
cmd-unhide-all = Показать все
cmd-make-group = Сгруппировать
cmd-make-component = Создать компонент…
cmd-make-unique = Сделать уникальным
cmd-make-parametric = Сделать параметрическим
cmd-explode = Разбить
cmd-close-group = Закрыть группу
cmd-intersect-model = С моделью
cmd-intersect-selection = С выделением
cmd-intersect-context = С контекстом
cmd-reverse-faces = Перевернуть грани
cmd-orient-faces = Ориентировать грани
cmd-flip-red = Красное направление
cmd-flip-green = Зелёное направление
cmd-flip-blue = Синее направление
cmd-unfold = Развёртка…
cmd-cut-list = Раскрой…
cmd-physics = Физика…
cmd-text-3d = Объёмный текст…
cmd-library-3d = Онлайн-библиотека 3D…
cmd-materials-online = Онлайн-библиотека материалов…
cmd-zoom-extents = Показать всё
cmd-snapping = Привязки
cmd-show-grid = Показать сетку
cmd-side-bar = Боковая панель
cmd-render = Рендер…
cmd-work-planes = Рабочие плоскости…
cmd-model-info = Сведения о документе…
cmd-component-params = Параметры компонента…
cmd-generator-editor = Редактор генераторов…
cmd-console = Консоль скриптов
cmd-log = Журнал
cmd-preferences = Настройки…

## Описания (одна строка для подсказки на панели инструментов)

cmd-desc-flip = Отразить выделение вдоль этой оси.
cmd-desc-snap = Эта привязка вкл. / выкл.
cmd-desc-new = Создать новый документ.
cmd-desc-open = Открыть файл Varstotas.
cmd-desc-save = Сохранить документ.
cmd-desc-save-as = Сохранить документ под новым именем.
cmd-desc-preview-show = Картинка, которую библиотека компонентов показывает для этого файла.
cmd-desc-preview-from-file = Использовать файл изображения как предпросмотр документа.
cmd-desc-preview-from-view = Использовать текущий вид как предпросмотр документа.
cmd-desc-preview-remove = Убрать предпросмотр (библиотека нарисует модель).
cmd-desc-run-script = Запустить файл скрипта Rhai на модели.
cmd-desc-quit = Закрыть Varstotas.
cmd-desc-cut = Скопировать выделение и стереть его.
cmd-desc-copy = Скопировать выделение.
cmd-desc-paste = Разместить скопированную геометрию мышью.
cmd-desc-paste-in-place = Вставить туда, где была геометрия.
cmd-desc-delete = Стереть выделение.
cmd-desc-purge-unused = Убрать материалы и компоненты, которые модель больше не использует.
cmd-desc-delete-guides = Стереть все направляющие линии и точки.
cmd-desc-select-all = Выделить всё в активном контексте.
cmd-desc-select-none = Снять выделение.
cmd-desc-hide = Скрыть выделение.
cmd-desc-unhide-all = Показать всё скрытое.
cmd-desc-make-group = Завернуть выделение в группу.
cmd-desc-make-component = Превратить выделение в компонент.
cmd-desc-make-unique = Дать выделенному компоненту собственное определение: его правка больше не меняет другие копии.
cmd-desc-make-parametric = Дать выделенной группе параметры (Параметры компонента).
cmd-desc-explode = Распаковать выделенные группы и компоненты.
cmd-desc-close-group = Выйти из редактируемой группы.
cmd-desc-reverse-faces = Поменять лицевую и обратную стороны выделенных граней.
cmd-desc-orient-faces = Повернуть грани рядом с выделенной так же.
cmd-desc-unfold = Разложить выделенные грани на плоскости (SVG / PDF / DXF).
cmd-desc-cut-list = Разложить детали по листам и доскам: карты раскроя, список покупок, стоимость (PDF / SVG / CSV).
cmd-desc-physics = Уронить выделенные группы и компоненты на модель под действием тяжести или опустить грань как ткань.
cmd-desc-text-3d = Добавить текст как объёмную геометрию (группу).
cmd-desc-zoom-extents = Вписать всю модель в вид.
cmd-desc-snapping = Привязки вкл. / выкл.
cmd-desc-show-grid = Сетка на земле вкл. / выкл.
cmd-desc-side-bar = Боковая панель с карточками (Свойства объекта, Материалы …) показана или скрыта.
cmd-desc-render = Изображения модели, построенные трассировкой путей (Окно → Рендер).
cmd-desc-work-planes = Сохранённые плоскости рисования.
cmd-desc-model-info = Статистика документа, Удалить неиспользуемое.
cmd-desc-component-params = Параметры выделенного компонента: значения, которые поворачивают, сдвигают, показывают или красят его части.
cmd-desc-library-3d = Бесплатные модели из онлайн-библиотек.
cmd-desc-materials-online = Бесплатные PBR-материалы из онлайн-библиотек.
cmd-desc-uv-editor = Раскладка текстурных координат выбранных граней.
cmd-desc-generator-editor = Писать и запускать скрипты генераторов.
cmd-desc-console = Выполнять команды Rhai на модели.
cmd-desc-log = Ошибки и предупреждения отовсюду: скрипты, импорт / экспорт, библиотеки, загрузки …
cmd-desc-preferences = Настройки Varstotas.
# { $name } — «Отменить» или «Повторить».
cmd-desc-undo-redo = { $name } последнее изменение.
cmd-desc-intersect-model = Добавить рёбра там, где выделенные грани режут модель.
cmd-desc-intersect-selection = Добавить рёбра там, где выделенные грани режут друг друга.
cmd-desc-intersect-context = Добавить рёбра там, где выделенные грани режут редактируемую группу.
cmd-desc-solid-outer-shell = Оставить внешнюю оболочку двух выделенных тел.
cmd-desc-solid-intersect = Оставить общий объём двух выделенных тел.
cmd-desc-solid-union = Соединить два выделенных тела.
cmd-desc-solid-subtract = Вырезать первое из второго из двух выделенных тел.
cmd-desc-solid-trim = Обрезать второе по первому из двух выделенных тел.
cmd-desc-solid-split = Разделить на части два выделенных тела.
# { $card } — название карточки.
cmd-desc-card = Карточка «{ $card }» боковой панели: показать или скрыть.
# { $name } — название команды.
cmd-desc-default = { $name }.

cmd-name-preview-show = Изображение предпросмотра: показать

cmd-name-preview-from-file = Изображение предпросмотра из файла

cmd-name-preview-from-view = Изображение предпросмотра из вида

cmd-name-preview-remove = Удалить изображение предпросмотра

cmd-name-intersect-model = Пересечь с моделью

cmd-name-intersect-selection = Пересечь с выделением

cmd-name-intersect-context = Пересечь с контекстом

cmd-name-flip-red = Отразить по красной

cmd-name-flip-green = Отразить по зелёной

cmd-name-flip-blue = Отразить по синей

# { $name } — название формата, как в меню.
cmd-name-import = Импорт { $name }
cmd-name-export = Экспорт { $name }
# { $name } — название твердотельной операции.
cmd-name-solid = Твердотельная операция: { $name }
# { $name } — название привязки.
cmd-name-snap = Привязка: { $name }
# { $card } — название карточки.
cmd-name-card = Боковая панель: { $card }

## ======== Инструменты ========

tool-select = Выделение
tool-eraser = Ластик
tool-paint = Заливка
tool-texture-position = Положение текстуры
tool-line = Линия
tool-arc = Дуга по 2 точкам
tool-arc-center = Дуга
tool-arc3 = Дуга по 3 точкам
tool-pie = Сектор
tool-bezier = Кривая Безье
tool-smooth-curve = Гладкая кривая
tool-freehand = От руки
tool-edit-curve = Правка кривой
tool-rectangle = Прямоугольник
tool-rotated-rectangle = Повёрнутый прямоугольник
tool-circle = Окружность
tool-polygon = Многоугольник
tool-box = Параллелепипед
tool-cylinder = Цилиндр
tool-cone = Конус
tool-pyramid = Пирамида
tool-sphere = Сфера
tool-torus = Тор
tool-push-pull = Тяни/Толкай
tool-move = Перемещение
tool-rotate = Поворот
tool-scale = Масштаб
tool-offset = Смещение контура
tool-chamfer = Фаска
tool-fillet = Скругление
tool-solid-tools = Твердотельные операции
tool-follow-me = Протягивание
tool-tape = Рулетка
tool-protractor = Транспортир
tool-axes = Оси
tool-dimension = Размеры
tool-text = Текст
tool-section = Секущая плоскость
tool-guide-plane = Направляющая плоскость
tool-slice = Разрез
tool-orbit = Вращение
tool-pan = Панорама
tool-zoom = Масштабирование вида
tool-position-camera = Установка камеры
tool-look-around = Осмотреться
tool-walk = Обход

tool-select-desc = Выделяйте объекты, чтобы изменять их другими инструментами.
tool-eraser-desc = Стирать рёбра, направляющие и другие объекты.
tool-paint-desc = Применить материал к граням и группам.
tool-texture-position-desc = Перемещать, поворачивать и масштабировать текстуру грани за маркеры.
tool-line-desc = Рисовать рёбра. Замкнутые плоские контуры становятся гранями.
tool-arc-desc = Нарисовать дугу по двум конечным точкам и выпуклости.
tool-arc3-desc = Нарисовать дугу через три точки.
tool-arc-center-desc = Нарисовать дугу по центру и двум точкам.
tool-pie-desc = Нарисовать замкнутую грань-сектор по центру и двум точкам.
tool-bezier-desc = Нарисовать кривую с рычагами: щелчок — угол, нажатие и перетаскивание — гладкая точка.
tool-smooth-curve-desc = Нарисовать гладкую кривую через щёлкаемые точки; замкнутая, она образует грань.
tool-freehand-desc = Нарисовать кривую от руки: нажмите и тяните; если конец в начале, получится грань.
tool-edit-curve-desc = Изменить форму нарисованной кривой: тяните точки и рычаги, добавляйте или удаляйте точки.
tool-rectangle-desc = Нарисовать прямоугольную грань по двум углам.
tool-rotated-rectangle-desc = Нарисовать прямоугольник под любым углом по ребру и ширине.
tool-circle-desc = Нарисовать окружность по центру и радиусу.
tool-polygon-desc = Нарисовать правильный многоугольник.
tool-box-desc = Нарисовать параллелепипед: углы основания, затем высота. Размеры остаются редактируемыми.
tool-cylinder-desc = Нарисовать цилиндр: центр, радиус, высота. Размеры остаются редактируемыми.
tool-cone-desc = Нарисовать конус: центр, радиус, высота. Размеры остаются редактируемыми.
tool-pyramid-desc = Нарисовать пирамиду: углы основания, затем высота. Размеры остаются редактируемыми.
tool-sphere-desc = Нарисовать сферу на грани: центр, радиус. Размеры остаются редактируемыми.
tool-torus-desc = Нарисовать тор на грани: центр, радиус, трубка. Размеры остаются редактируемыми.
tool-push-pull-desc = Тянуть или толкать грань, добавляя или убирая объём.
tool-move-desc = Перемещать, растягивать или копировать объекты.
tool-rotate-desc = Поворачивать, растягивать или копировать объекты вокруг оси.
tool-scale-desc = Изменять размер и растягивать объекты.
tool-offset-desc = Смещать рёбра грани внутрь или наружу.
tool-chamfer-desc = Срезать ребро между двумя гранями плоской фаской.
tool-fillet-desc = Скруглить ребро между двумя гранями.
tool-solid-tools-desc = Объединять тела-группы: внешняя оболочка, пересечение, объединение, вычитание, обрезка, разделение.
tool-follow-me-desc = Протянуть грань вдоль пути.
tool-tape-desc = Измерять расстояния и создавать направляющие.
tool-protractor-desc = Измерять углы и создавать наклонные направляющие.
tool-axes-desc = Переместить или переориентировать оси рисования.
tool-dimension-desc = Ставить линейные размеры между точками или по рёбрам.
tool-text-desc = Добавить текст с выноской, указывающей на модель.
tool-section-desc = Разрезать модель секущей плоскостью, чтобы увидеть внутри.
tool-guide-plane-desc = Добавить рабочую плоскость для рисования в воздухе; оси и сетка следуют за ней.
tool-slice-desc = Разрезать выделение или модель плоскостью; при желании разделить на две группы.
tool-orbit-desc = Вращать камеру вокруг модели.
tool-pan-desc = Двигать камеру в сторону.
tool-zoom-desc = Приближать или отдалять камеру.
tool-position-camera-desc = Встать в точку на уровне глаз и посмотреть на модель.
tool-look-around-desc = Повернуть голову: вращать вид вокруг глаза.
tool-walk-desc = Ходить по модели на уровне глаз.

solid-union = Объединение
solid-intersect = Пересечение
solid-subtract = Вычитание
solid-trim = Обрезка
solid-split = Разделение
solid-outer-shell = Внешняя оболочка

## Разрез

tool-slice-undo = Разрез
tool-slice-undo-split = Разрез с разделением
tool-slice-nothing = Разрез: плоскость ничего не режет
tool-slice-status-first = Щёлкните точку на секущей плоскости.
tool-slice-status-second = Щёлкните вторую точку. Двойной щелчок / Enter: плоскость грани; стрелки: нормаль вдоль красной / зелёной / синей оси / ребра.
tool-slice-status-third = Щёлкните третью точку, чтобы разрезать.
# { $step } — один из текстов tool-slice-status-*; { $split } — «on» или «off».
tool-slice-status = { $step } Режет выделение или всё в активном контексте. %COPY%: { $split ->
        [on] Разделение ВКЛ: части становятся двумя группами
       *[off] Разделение ВЫКЛ: добавляются только рёбра
    }.

## Подписи поля значений (общие)

tool-vcb-measurements = Значения
tool-vcb-radius = Радиус
tool-vcb-angle = Угол
tool-vcb-length = Длина
tool-vcb-segments = Сегменты
tool-vcb-distance = Расстояние
tool-vcb-width-angle = Ширина, угол
tool-vcb-dimensions = Размеры
tool-vcb-height = Высота
tool-vcb-sides = Стороны
tool-vcb-bulge = Выпуклость
tool-vcb-origin = Начало
tool-vcb-point = Точка
tool-vcb-eye-height = Высота глаз
tool-vcb-height-offset = Смещение по высоте
tool-vcb-fov = Угол обзора

## Дуги, прямоугольники

tool-undo-pie = Сектор
tool-undo-arc = Дуга
tool-undo-arc3 = Дуга по 3 точкам
tool-undo-rotated-rectangle = Повёрнутый прямоугольник
# { $label } — название фигуры (название действия для отмены).
tool-degenerate = { $label }: вырожденный ввод
tool-cursor-angle = Угол
tool-arc-center-first = Выберите центр (грань задаёт плоскость; стрелки фиксируют плоскость). Введите `{ $segs }s` для числа сегментов.
tool-arc-center-start = Выберите начальную точку (радиус) или введите радиус.
tool-arc-center-end = Двигайтесь вокруг центра и щёлкните, чтобы закончить, или введите угол.
tool-arc-what-pie = сектор
tool-arc-what-arc = дуга
tool-arc3-first = Выберите начальную точку дуги.
tool-arc3-second = Выберите вторую точку, через которую проходит дуга, или введите длину.
tool-arc3-third = Выберите конечную точку или введите угол дуги.
tool-rotrect-first = Выберите первый угол прямоугольника.
tool-rotrect-second = Выберите второй угол (первое ребро) или введите длину (`l,w`).
tool-rotrect-third = Двигайтесь, чтобы задать ширину и угол, и щёлкните, или введите `ширина` или `ширина,угол`.

## Кривая Безье / кривые

tool-bezier-end = Enter или двойной щелчок завершает, щелчок на первой точке замыкает кривую в грань
tool-bezier-freehand-start = Нажмите и тяните, чтобы рисовать; закончите в начале, чтобы замкнуть в грань.
tool-bezier-freehand-draw = Тяните, чтобы рисовать; отпустите, чтобы закончить.
tool-bezier-smooth-first = Щёлкните первую точку кривой.
# { $end } — текст tool-bezier-end.
tool-bezier-smooth-next = Щёлкните следующую точку; { $end }. Backspace удаляет последнюю точку.
tool-bezier-handles-first = Щёлкните первую точку кривой; нажмите и тяните, чтобы вытянуть рычаги.
tool-bezier-handles-drag = Тяните, чтобы изогнуть кривую в этой точке; отпустите, чтобы поставить её.
# { $end } — текст tool-bezier-end.
tool-bezier-handles-next = Щёлкните следующую точку (тяните для рычагов); { $end }. Backspace удаляет последнюю точку.

## Инструменты камеры

tool-position-camera-status = Щёлкните, чтобы встать в точку, или тяните от места, где стоять, к месту, куда смотреть. Введите высоту глаз.
tool-look-around-status = Тяните, чтобы осмотреться. Введите высоту глаз и нажмите Enter.
tool-walk-status = Удерживайте кнопку и двигайте вверх/вниз, чтобы идти, влево/вправо, чтобы поворачивать (дальше — быстрее). Shift = вверх/вниз и вбок, %CMD% = бег. Стрелки — шаги.

## Скругление / Фаска

tool-undo-fillet = Скругление
tool-undo-chamfer = Фаска
# { $what } — название операции с маленькой буквы, { $error } — причина.
tool-chamfer-cant = { $what }: не удалось применить к этому ребру: { $error }
tool-chamfer-what-fillet = Скругление
tool-chamfer-what-chamfer = Фаска
tool-fillet-sizing = Отведите от ребра, чтобы задать радиус, и щёлкните. Введите радиус или `12s`; Esc отменяет.
tool-chamfer-sizing = Отведите от ребра, чтобы задать размер, и щёлкните. Введите расстояние; Esc отменяет.
tool-fillet-hover = Щёлкните ребро между двумя гранями, чтобы скруглить (выделенные рёбра: все сразу); двойной щелчок — прошлый радиус.
tool-chamfer-hover = Щёлкните ребро между двумя гранями, чтобы снять фаску (выделенные рёбра: все сразу); двойной щелчок — прошлый размер.

## Рулетка, транспортир, оси

tool-undo-resize-model = Изменить размер модели
tool-undo-resize = Изменить размер
tool-undo-create-guide = Создать направляющую
tool-undo-place-axes = Установить оси
tool-tape-guides-on = направляющие вкл.
tool-tape-measure-only = только измерение
# { $g } — tool-tape-guides-on или tool-tape-measure-only.
tool-tape-status-resize = Введите новую длину, чтобы изменить размер модели, или выберите начальную точку ({ $g }).
tool-tape-status-start = Выберите начальную точку (ребро — для параллельной направляющей). %COPY% переключает направляющие ({ $g }).
tool-tape-status-end = Выберите конечную точку или введите расстояние ({ $g }).
tool-protractor-center = Выберите центр транспортира (стрелки фиксируют его ось).
tool-protractor-base = Выберите базовое направление.
tool-protractor-angle = Выберите угол (привязка через 15°) или введите градусы.
tool-axes-origin = Выберите новое начало или введите x,y,z.
tool-axes-red = Выберите точку на новой красной оси.
tool-axes-green = Выберите точку в сторону новой зелёной оси.

## Правка кривых, размеры

tool-undo-add-curve-point = Добавить точку кривой
tool-undo-edit-curve = Правка кривой
tool-undo-remove-curve-point = Удалить точку кривой
tool-undo-edit-segments = Изменить сегменты
tool-curve-edit-too-few = Правка кривой: слишком мало точек, чтобы удалить одну
tool-curve-edit-pick = Щёлкните кривую, нарисованную инструментом «Кривая Безье», «Гладкая кривая» или «От руки».
tool-curve-edit-drag = Тяните, чтобы изменить форму; отпустите, чтобы применить, Esc отменяет.
tool-curve-edit-insert = Щёлкните, чтобы добавить здесь точку (форма кривой сохранится).
tool-curve-edit-idle = Тяните точки и рычаги; щёлкните кривую, чтобы добавить точку, Delete удаляет выделенную точку; двойной щелчок по точке: угол / гладкая. Enter завершает.
tool-undo-dimension = Размер
tool-dimension-first = Выберите начальную точку размера или щёлкните ребро, чтобы проставить его длину.
tool-dimension-second = Выберите конечную точку размера.
tool-dimension-third = Отведите, чтобы сместить размерную линию, и щёлкните, чтобы поставить.

## Линия, прямоугольник, окружность, многоугольник, дуга

tool-undo-line = Линия
tool-undo-rectangle = Прямоугольник
tool-undo-polygon = Многоугольник
tool-undo-circle = Окружность
tool-line-first = Выберите начальную точку. Стрелки фиксируют ось.
tool-line-next = Выберите конечную точку или введите длину. Shift = зафиксировать привязку, стрелки = зафиксировать ось, Esc/двойной щелчок = закончить.
tool-rectangle-first = Выберите первый угол. Стрелки выбирают плоскость (→ красная, ← зелёная, ↑ синяя нормаль).
tool-rectangle-second = Выберите противоположный угол или введите ширина,высота. Esc = отмена.
tool-circle-what-polygon = многоугольника
tool-circle-what-circle = окружности
tool-circle-for-sides = сторон
tool-circle-for-segments = сегментов
# { $what } — tool-circle-what-*, { $kind } — tool-circle-for-*.
tool-circle-first = Выберите центр { $what } (грань под курсором задаёт ориентацию; стрелки фиксируют нормаль). Введите `{ $segs }s` для числа { $kind }.
tool-circle-radius = Выберите точку на радиусе или введите радиус.
tool-arc-first = Выберите начальную точку дуги. Начните с конца ребра, чтобы рисовать по касательной к нему.
tool-arc-end-tangent = Выберите конечную точку (двойной щелчок — голубая касательная дуга) или введите длину хорды.
tool-arc-end = Выберите конечную точку или введите длину хорды.
tool-arc-bulge = Двигайтесь, чтобы задать выпуклость, и щёлкните, или введите выпуклость (`500r` для радиуса). Введите `12s` для сегментов.

## Ластик, Заливка, Тяни/Толкай, Смещение контура, Протягивание

tool-undo-hide = Скрыть
tool-undo-unsoften = Убрать сглаживание
tool-undo-soften = Сгладить
tool-undo-erase = Стереть
tool-eraser-status = Щёлкайте или проведите по рёбрам, направляющим, размерам или группам, чтобы стереть. Shift = скрыть, %CMD% = сгладить, %CMD%+Shift = убрать сглаживание.
tool-paint-mods = Alt = образец, Ctrl = связанные грани с тем же материалом, Shift = все подходящие грани, Ctrl+Shift = подходящие связанные грани
    .macos = Cmd = образец, Option = связанные грани с тем же материалом, Shift = все подходящие грани, Option+Shift = подходящие связанные грани
tool-undo-paint = Заливка
# { $mods } — tool-paint-mods.
tool-paint-status = Щёлкните грань или группу, чтобы залить. { $mods }.
tool-paint-default-material = По умолчанию
tool-undo-push-pull = Тяни/Толкай
# { $keep } — «on» или «off».
tool-push-pull-moving = Двигайте, чтобы задать расстояние, щёлкните или введите значение. %COPY% = оставить исходную грань ({ $keep ->
        [on] вкл.
       *[off] выкл.
    }).
tool-push-pull-idle = Щёлкните грань, чтобы тянуть или толкать. Двойной щелчок повторяет прошлое расстояние.
tool-undo-offset = Смещение контура
tool-offset-moving = Двигайте внутрь или наружу и щёлкните, или введите расстояние.
tool-offset-idle = Щёлкните грань, чтобы сместить её рёбра. Двойной щелчок повторяет прошлое смещение.
tool-undo-follow-me = Протягивание
tool-follow-me-profile = Щёлкните грань-профиль (предварительно выделите рёбра пути, чтобы протянуть сразу).
tool-follow-me-path = Щёлкайте или тяните вдоль рёбер пути; двойной щелчок/Enter — закончить. Щёлкните грань, чтобы протянуть вокруг её края. Esc = отмена.

## Выделение

tool-select-mods = Ctrl = добавить, Shift = переключить, Ctrl+Shift = убрать
    .macos = Option (или Cmd) = добавить, Shift = переключить, Shift+Option = убрать
tool-undo-aim-light = Направить свет
tool-undo-active-cut = Активный разрез
tool-select-aim = Направьте свет: опустите маркер туда, куда он должен светить. Esc = отмена.
# { $mods } — tool-select-mods.
tool-select-box = Тяните слева направо для выделения рамкой, справа налево для выделения секущей рамкой. { $mods }.
tool-select-idle = Выделяйте объекты. { $mods }. Тяните, чтобы выделить несколько. Двойной щелчок по группе — править её.

## Перемещение, Поворот, Масштаб, Вставка

tool-undo-copy = Копировать
tool-undo-move = Переместить
tool-undo-copy-array = Копировать массив
tool-undo-rotate = Повернуть
tool-undo-rotate-array = Повернуть массив
tool-undo-scale = Масштабировать
tool-undo-paste = Вставить
tool-suffix-copy = (копия)
tool-suffix-on = (вкл.)
# { $copy } — пусто или " " + tool-suffix-copy.
tool-move-from = Выберите точку, от которой перемещать{ $copy }. Перемещает выделение или объект под курсором. %COPY% = копия; после копии введите `5x` или `/5` для массивов.
tool-move-to = Выберите назначение или введите расстояние{ $copy }. Shift = зафиксировать привязку, стрелки = зафиксировать ось.
tool-rotate-center = Выберите центр поворота (грань ориентирует транспортир; стрелки фиксируют его ось).
tool-rotate-zero = Выберите нулевое направление.
# { $step } — tool-rotate-rounded или tool-rotate-snaps; { $on } — пусто или " " + tool-suffix-on.
tool-rotate-angle = Выберите угол ({ $step }) или введите градусы. %COPY% = копия{ $on }; после копии введите `6x` для кругового массива.
# { $step } — угол.
tool-rotate-rounded = округление до { $step }°
tool-rotate-snaps = привязка через 15°
tool-scale-uniformly = равномерно
tool-scale-two-axes = по двум осям
tool-scale-one-axis = по одной оси
tool-scale-about-centre = от центра
tool-scale-about-opposite = от противоположного маркера
# { $what } — tool-scale-uniformly / -two-axes / -one-axis, { $about } — tool-scale-about-*.
tool-scale-dragging = Двигайте, чтобы масштабировать { $what } { $about }; щёлкните или введите коэффициент (по осям: `1.5; 2`). %CMD% = от центра.
tool-scale-select = Выделите объекты (или наведите на один), затем тяните маркер.
tool-scale-idle = Тяните маркер: угловые масштабируют равномерно, середины рёбер — по двум осям, центры граней — по одной. %CMD% = от центра.
# { $factors } — коэффициент(ы) масштаба.
tool-scale-tooltip-uniform = Равномерный масштаб { $factors }
tool-scale-tooltip = Масштаб { $factors }
# { $name } — название размещаемого компонента.
tool-paste-place-named = Щёлкните, чтобы разместить { $name } (Esc отменяет).
# { $count } — число объектов.
tool-paste-place = Щёлкните, чтобы разместить вставляемые объекты: { $count ->
        [one] { $count } объект
        [few] { $count } объекта
        [many] { $count } объектов
       *[other] { $count } объекта
    } (Esc отменяет).

## Сечение, тела, текст, текстура, рабочая плоскость, прочее

tool-undo-section-plane = Секущая плоскость
tool-section-status = Щёлкните грань, чтобы поставить секущую плоскость. Она отсекает сторону за своими стрелками.

## Фигуры (параллелепипед, цилиндр, конус, пирамида, сфера, тор)

tool-vcb-tube = Трубка
# { $what } — название фигуры с маленькой буквы.
tool-prim-box-first = Фигура «{ $what }»: щёлкните первый угол основания (грань под курсором задаёт ориентацию; стрелки задают плоскость).
tool-prim-round-first = Фигура «{ $what }»: щёлкните центр основания (грань под курсором задаёт ориентацию). Введите `{ $segs }s` для сегментов.
tool-prim-box-second = Щёлкните противоположный угол или введите `ширина, глубина` (`ширина, глубина, высота` ставит сразу).
tool-prim-sphere-radius = Двигайте, чтобы задать радиус, и щёлкните, или введите его.
tool-prim-torus-radius = Щёлкните радиус (до середины трубки) или введите `радиус, трубка`.
tool-prim-round-radius = Щёлкните радиус или введите `радиус` (`радиус, высота` ставит сразу).
tool-prim-torus-tube = Двигайте, чтобы задать радиус трубки, и щёлкните, или введите его.
tool-prim-height = Двигайте вверх или вниз, чтобы задать высоту, и щёлкните, или введите её.

# { $name } — название твердотельной операции.
tool-solid-not-solid = { $name }: не тело (каждое ребро должно ограничивать ровно две грани, без вложенных групп).
tool-solid-first = { $name }: щёлкните первую группу или компонент-тело.
tool-solid-next = { $name }: щёлкните следующее тело, чтобы добавить его (Esc — начать заново).
tool-solid-cut = { $name }: щёлкните тело, которым резать первое (Esc — начать заново).
tool-solid-second = { $name }: щёлкните второе тело (Esc — начать заново).
tool-undo-text = Текст
tool-text-first = Щёлкните точку, ребро или грань, чтобы прикрепить текст, или пустое место для текста без выноски.
tool-text-place = Двигайте, чтобы разместить текст, и щёлкните. Правьте текст в Свойствах объекта.
tool-text3d-status = Щёлкните, чтобы разместить объёмный текст (Esc отменяет).
tool-undo-texture-position = Положение текстуры
tool-texture-pick = Щёлкните грань с текстурой, чтобы изменить положение текстуры.
tool-texture-move = Перемещение текстуры. Esc отменяет.
tool-texture-green = Поворот и масштаб текстуры вокруг красного маркера.
tool-texture-blue = Масштаб высоты текстуры.
tool-texture-idle = Тяните грань или красный маркер, чтобы переместить, зелёный — повернуть/масштабировать, синий — растянуть. Готово: Enter или щелчок снаружи.
tool-texture-idle-many = Граней: { $count }, текстура продолжается от выбранной. Shift+щелчок добавляет или убирает грань, Alt+щелчок продолжает текстуру на другую.
tool-texture-cant-continue = Текстуру нельзя продолжить на эту грань (она перпендикулярна).
tool-texture-vcb = Плитка

tool-undo-add-guide-plane = Добавить направляющую плоскость
tool-undo-edit-guide-plane = Изменить направляющую плоскость
tool-workplane-origin = Щёлкните начало плоскости (или введите x,y,z). Тяните маркеры активной плоскости: начало, конец нормали, красный маркер вращения.
tool-workplane-red = Щёлкните точку на новой красной оси. Двойной щелчок / Enter: плоскость грани; стрелки: нормаль вдоль красной / зелёной / синей оси / ребра.
tool-workplane-third = Щёлкните третью точку на плоскости.
tool-workplane-drag-origin = Переместить начало плоскости.
tool-workplane-drag-normal = Наклонить плоскость: переместить её нормаль.
tool-workplane-drag-spin = Повернуть плоскость вокруг её нормали.
tool-zoom-status = Тяните вверх/вниз для масштаба. Введите угол обзора и нажмите Enter.
tool-orbit-status = Тяните, чтобы вращать. Shift = панорама.
tool-pan-status = Тяните для панорамы.
# { $prompt } — подсказка плагина.
tool-plugin-select = { $prompt } — щёлкайте, чтобы добавить / убрать, Enter — продолжить, Esc отменяет.
# { $name } — название инструмента.
tool-not-implemented = Инструмент «{ $name }» пока не реализован.

## Панель подсказок (длинная справка по инструментам; строка, начинающаяся с •, — пункт списка)

tool-select-instructor =
    Щёлкните объект, чтобы выделить его.
    • Двойной щелчок по грани: выделить её с рёбрами.
    • Тройной щелчок: выделить всё связанное.
    • Тяните слева направо: выделение рамкой (целиком внутри).
    • Тяните справа налево: выделение секущей рамкой (касание).
    • Ctrl = добавить, Shift = переключить, Ctrl+Shift = убрать.
    • Двойной щелчок по группе — править её; Esc — закрыть.
    • Delete стирает выделение.
    .macos =
        Щёлкните объект, чтобы выделить его.
        • Двойной щелчок по грани: выделить её с рёбрами.
        • Тройной щелчок: выделить всё связанное.
        • Тяните слева направо: выделение рамкой (целиком внутри).
        • Тяните справа налево: выделение секущей рамкой (касание).
        • Option (или Cmd) = добавить, Shift = переключить, Shift+Option = убрать.
        • Двойной щелчок по группе — править её; Esc — закрыть.
        • Delete стирает выделение.
tool-eraser-instructor = Щёлкайте или проводите по рёбрам, чтобы стереть их. Стирание ребра убирает грани, которые оно ограничивает.
tool-paint-instructor =
    Выберите материал в панели «Материалы», затем щёлкайте грани.
    • Alt: взять образец материала.
    • Ctrl: залить связанные грани с тем же материалом.
    • Shift: заменить материал на всех подходящих гранях.
    • Ctrl+Shift: заменить его на подходящих связанных гранях.
    .macos =
        Выберите материал в панели «Материалы», затем щёлкайте грани.
        • Cmd: взять образец материала.
        • Option: залить связанные грани с тем же материалом.
        • Shift: заменить материал на всех подходящих гранях.
        • Option+Shift: заменить его на подходящих связанных гранях.
tool-texture-position-instructor =
    Задайте положение текстуры грани с текстурой (выделите или щёлкните её).
    • Тяните грань или красный маркер: переместить текстуру.
    • Зелёный маркер: повернуть и масштабировать вокруг красного.
    • Синий маркер: растянуть высоту текстуры.
    • Маркеры прилипают к вершинам и рёбрам; двойной щелчок по ребру выравнивает текстуру по нему.
    • Shift+щелчок добавляет грань, Alt+щелчок продолжает на неё текстуру (через углы).
    • Введите размер плитки на грани (`800` или `800;400`) или угол (`30°`).
    • Esc отменяет перетаскивание; Enter или щелчок снаружи завершает.
tool-line-instructor =
    Щёлкните для начала, щёлкните для конца. Каждый конец начинает новый отрезок.
    Введите длину и нажмите Enter для точности.
    Shift фиксирует направление привязки; стрелки фиксируют оси.
tool-arc-instructor =
    Щёлкните начальную точку, щёлкните конечную, затем двигайте, чтобы задать выпуклость, и щёлкните.
    • Введите длину хорды, затем выпуклость (или `500r` для радиуса); `12s` задаёт сегменты.
    • Если начать с конца ребра, голубая дуга касается его.
    • Двойной щелчок рисует дугу, касательную к предыдущей.
tool-arc-center-instructor =
    Щёлкните центр, щёлкните начальную точку (радиус), затем двигайтесь вокруг и щёлкните конец.
    После первого щелчка введите радиус, затем угол. `12s` задаёт сегменты.
tool-arc3-instructor =
    Щёлкните начальную точку, щёлкните точку, через которую проходит дуга, затем щёлкните конец.
    После первого щелчка введите длину, после второго — угол.
tool-bezier-instructor =
    Щёлкайте точки кривой; нажмите и тяните в точке, чтобы вытянуть её рычаги (простой щелчок делает угол).
    Enter или двойной щелчок завершает открытую кривую (только рёбра); щелчок на первой точке замыкает её в грань.
    Backspace удаляет последнюю точку. Введите длину до следующей точки в сторону курсора; `12s` задаёт сегменты на участок.
tool-edit-curve-instructor =
    Меняйте форму кривой, нарисованной инструментом «Кривая Безье», «Гладкая кривая» или «От руки» (двойной щелчок по ней в Выделении или выделите её и выберите «Правка кривой»).
    • Тяните точку (квадрат), чтобы переместить её, рычаг (круг) — чтобы изогнуть кривую; гладкая точка держит рычаги на одной линии.
    • Щёлкните кривую между точками, чтобы добавить точку; Delete удаляет выделенную точку.
    • Двойной щелчок по точке: угол ↔ гладкая. `12s` задаёт сегменты на участок.
    • Enter, Esc или щелчок по пустому месту завершает.
tool-smooth-curve-instructor =
    Щёлкайте точки; кривая плавно проходит через них.
    Enter или двойной щелчок завершает открытую кривую (только рёбра); щелчок на первой точке замыкает её в грань.
    Backspace удаляет последнюю точку. Введите длину до следующей точки в сторону курсора; `12s` задаёт сегменты на участок.
tool-freehand-instructor =
    Нажмите и тяните, чтобы рисовать; отпустите, чтобы закончить. Штрих сглаживается.
    Штрих, закончившийся в начале, замыкается в грань; иначе остаются рёбра.
tool-pie-instructor =
    Щёлкните центр, щёлкните начальную точку (радиус), затем щёлкните конец, чтобы получить замкнутую грань-сектор.
    Введите радиус, затем угол.
tool-rectangle-instructor = Щёлкните первый угол, затем противоположный. Введите ширина,высота для точности.
tool-rotated-rectangle-instructor =
    Щёлкните первый угол, щёлкните второй (первое ребро), затем двигайте, чтобы задать ширину, и щёлкните.
    Введите длину (или `l,w`) ребра, затем ширину (или `w,угол`).
tool-circle-instructor = Щёлкните центр, затем двигайте и щёлкните, чтобы задать радиус. Введите `24s` перед щелчком, чтобы задать сегменты.
tool-polygon-instructor = Щёлкните центр, затем радиус. Введите `6s`, чтобы задать число сторон.
tool-box-instructor =
    Щёлкните первый угол основания, щёлкните противоположный, затем двигайте вверх или вниз и щёлкните, чтобы задать высоту.
    • Грань под курсором несёт основание; стрелки после первого щелчка задают его плоскость.
    • Введите `ширина, глубина`, затем высоту — или сразу `ширина, глубина, высота`.
    • Результат — компонент: меняйте его размеры в Свойствах объекта; разбейте его для обычной геометрии.
tool-cylinder-instructor =
    Щёлкните центр основания, щёлкните радиус, затем двигайте вверх или вниз и щёлкните, чтобы задать высоту.
    • Введите радиус, затем высоту — или сразу `радиус, высота`; `32s` задаёт сегменты.
    • Результат — компонент: меняйте его размеры в Свойствах объекта; разбейте его для обычной геометрии.
tool-sphere-instructor =
    Щёлкните, где стоит сфера, затем двигайте и щёлкните, чтобы задать радиус (или введите его).
    • `32s` задаёт сегменты. Результат — компонент: меняйте его размеры в Свойствах объекта.
tool-torus-instructor =
    Щёлкните центр, щёлкните радиус (до середины трубки), затем двигайте и щёлкните, чтобы задать трубку.
    • Сразу введите `радиус, трубка`; `32s` задаёт сегменты.
    • Результат — компонент: меняйте его размеры в Свойствах объекта.
tool-push-pull-instructor =
    Наведите на грань, щёлкните, двигайте, чтобы задать расстояние, щёлкните снова.
    %COPY% оставляет исходную грань. Двойной щелчок повторяет прошлое расстояние.
tool-move-instructor = Щёлкните точку, чтобы взять выделение, щёлкните назначение. %COPY% делает копию; введите `5x` для массива.
tool-rotate-instructor = Щёлкните центр, щёлкните нулевое направление, щёлкните угол. Введите угол в градусах.
tool-scale-instructor = Выделите объекты, затем тяните маркер. Введите коэффициент масштаба.
tool-offset-instructor = Щёлкните грань, двигайте внутрь или наружу, щёлкните. Введите расстояние.
tool-chamfer-instructor =
    Щёлкните ребро между двумя гранями (или одно из выделенных рёбер: все сразу), отведите от него, чтобы задать размер, щёлкните снова.
    • Вводите расстояние при задании размера; двойной щелчок берёт прошлый размер.
    • Ввод сразу после фаски переделывает её с новым размером. Esc отменяет.
tool-fillet-instructor =
    Щёлкните ребро между двумя гранями (или одно из выделенных рёбер: все сразу), отведите от него, чтобы задать радиус, щёлкните снова.
    • Введите радиус, `12s` для числа сегментов или оба: `20,12s`.
    • Двойной щелчок берёт прошлый радиус; ввод сразу после скругления переделывает его. Esc отменяет.
tool-solid-tools-instructor =
    Инструменты → Твердотельные операции: выберите операцию, затем щёлкните первую группу-тело, потом вторую.
    • Если тела выделены, операция выполняется сразу.
    • Вычитание / Обрезка режут второе тело первым (Обрезка сохраняет первое).
    • Разделение даёт три части; Внешняя оболочка / Объединение продолжают добавлять щёлкаемые тела.
    • Тело: каждое ребро ограничивает ровно две грани, без вложенных групп.
tool-follow-me-instructor = Выделите путь, затем щёлкните грань-профиль, чтобы протянуть её.
tool-tape-instructor = Щёлкните две точки, чтобы измерить. Начало на ребре создаёт параллельную направляющую.
tool-protractor-instructor = Щёлкните центр, щёлкните базовое направление, щёлкните угол, чтобы создать направляющую.
tool-axes-instructor =
    Щёлкните новое начало, щёлкните точку на красной оси, затем точку в сторону зелёной оси.
    Введите `x,y,z` для начала. Щёлкните оси правой кнопкой: Установить / Переместить / Сбросить.
tool-dimension-instructor =
    Щёлкните начальную и конечную точки (или щёлкните ребро), затем двигайте, чтобы сместить размер, и щёлкните.
    Двойной щелчок по тексту размера в Выделении — править его; пустой текст возвращает измеренную длину.
tool-text-instructor =
    Щёлкните точку, ребро или грань, отведите текст и щёлкните снова.
    • Текст по умолчанию: координаты точки, длина ребра, площадь грани.
    • Щёлкните пустое место для текста без выноски.
    • Правьте текст в Свойствах объекта; перемещайте его Перемещением или вместе с геометрией.
tool-section-instructor =
    Щёлкните грань, чтобы поставить на неё секущую плоскость; она становится активным разрезом.
    • Стрелки указывают на часть, которая остаётся видимой.
    • Выделите плоскость за её границу; перемещайте Перемещением, удаляйте клавишей Delete.
    • Правый щелчок по плоскости: Активный разрез, Обратить, Выровнять вид.
    • Вид → Секущие плоскости / Разрезы сечением / Заливка сечений.
tool-guide-plane-instructor =
    Щёлкните три точки: начало, точку на красной оси, точку на плоскости.
    • Двойной щелчок по грани (или Enter после первого щелчка): плоскость этой грани.
    • После первого щелчка стрелки задают нормаль: → красная, ← зелёная, ↑ синяя, ↓ последнее ребро.
    • Новая плоскость активна: точки в воздухе ложатся на неё, оси и сетка следуют за ней.
    • Тяните маркеры активной плоскости: начало, конец нормали (наклон), красный маркер (вращение).
    • Окно → Рабочие плоскости: активировать, скрыть, переименовать, удалить или ввести углы.
tool-slice-instructor =
    Задайте секущую плоскость как направляющую: щёлкните три точки.
    • Двойной щелчок по грани (или Enter после первого щелчка): плоскость этой грани.
    • После первого щелчка стрелки задают нормаль: → красная, ← зелёная, ↑ синяя, ↓ последнее ребро.
    • Режет выделение (грани, группы, компоненты) или всё в активном контексте.
    • Рёбра добавляются вдоль разреза; компонент, общий с другими копиями, сначала делается уникальным.
    • %COPY% переключает разделение (+ у курсора): каждое разрезанное тело становится двумя группами, закрытыми в месте разреза.
tool-orbit-instructor =
    Тяните, чтобы вращать вокруг точки под курсором.
    Средняя кнопка мыши вращает в любом инструменте; Shift+средняя — панорама.
    Трекпад: прокрутка двумя пальцами вращает, Shift+прокрутка — панорама, щипок — масштаб.
tool-pan-instructor = Тяните для панорамы вида. Shift + средняя кнопка мыши — панорама в любом инструменте.
tool-zoom-instructor =
    Тяните вверх, чтобы приблизить, вниз — отдалить. Колесо мыши масштабирует к курсору в любом инструменте.
    Введите угол обзора (например 35) и нажмите Enter. Shift+Z показывает всю модель.
tool-position-camera-instructor =
    Щёлкните, чтобы встать в точку, глядя туда, куда сейчас смотрит камера.
    • Тяните от места, где стоять, к месту, куда смотреть.
    • Перед щелчком введите высоту глаз (по умолчанию 1676 мм).
    • Затем переключается на «Осмотреться».
tool-look-around-instructor = Тяните, чтобы поворачивать вид вокруг глаза. Введите высоту глаз и нажмите Enter.
tool-walk-instructor =
    Тяните вверх/вниз, чтобы идти вперёд/назад, влево/вправо — поворачивать.
    • Shift: вверх/вниз и вбок. %CMD%: бег.
    • Стрелки: шаги и повороты. Введите высоту глаз и нажмите Enter.

# { $prompt } — подсказка плагина, { $index } из { $total } — выбираемая точка.
tool-plugin-pick = { $prompt } ({ $index } из { $total }; Esc отменяет).

## ======== Привязки и вывод ========

snap-endpoint = Конечные точки
snap-midpoint = Середины
snap-center = Центры
snap-intersection = Пересечения
snap-origin = Начало координат
snap-on-edge = На ребре
snap-on-face = На грани
snap-on-guide = На направляющих линиях
snap-guide-point = Точки направляющих
snap-on-red-axis = Красная ось
snap-on-green-axis = Зелёная ось
snap-on-blue-axis = Синяя ось
snap-parallel-to-edge = Параллельно ребру
snap-perpendicular-to-edge = Перпендикулярно ребру
snap-from-point = От точки
snap-tangent = Касательная в вершине
snap-square = Квадрат
snap-golden-section = Золотое сечение
snap-half-circle = Полуокружность
snap-grid-point = Точки сетки
snap-on-grid-line = Линии сетки

inference-endpoint = Конечная точка
inference-midpoint = Середина
inference-center = Центр
inference-intersection = Пересечение
inference-on-edge = На ребре
inference-on-red-axis = На красной оси
inference-on-green-axis = На зелёной оси
inference-on-blue-axis = На синей оси
inference-parallel-to-edge = Параллельно ребру
inference-perpendicular-to-edge = Перпендикулярно ребру
inference-from-point = От точки
inference-on-face = На грани
inference-guide-point = Точка направляющей
inference-on-guide = На линии
inference-origin = Начало координат
inference-square = Квадрат
inference-golden-section = Золотое сечение
inference-half-circle = Полуокружность
inference-tangent = Касательная в вершине
inference-grid-point = Точка сетки
inference-on-grid-line = На линии сетки

## Группы меню привязок

snap-group-points = Точки
snap-group-on-objects = На объектах
snap-group-directions = Направления
snap-group-shapes = Фигуры
snap-group-grid = Сетка
## ======== Правка ========

edit-nothing-selected = ничего не выделено
edit-nothing-to-copy = нечего копировать
edit-component-file-empty = файл компонента пуст
edit-internal-error = внутренняя ошибка
edit-untitled = Без имени
# { $error } — текст ошибки.
edit-parameters-error = Параметры: { $error }
edit-undo-erase = Стереть
edit-undo-delete-guides = Удалить направляющие

# { $label } — название отменяемой операции, { $error } — причина.
edit-failed = { $label }: не удалось — { $error }
edit-op-error = { $label }: { $error }

## ======== Виды ========

view-iso = Изометрия
view-top = Сверху
view-bottom = Снизу
view-front = Спереди
view-back = Сзади
view-left = Слева
view-right = Справа

## HUD вида (выпадающие списки видового экрана)

view-style-shaded = С заливкой
view-style-textured = С текстурами
view-style-textured-menu = С текстурами
view-style-monochrome = Монохром
view-style-hidden-line = Скрытые линии
view-style-wireframe = Каркас
view-style-realistic = Реалистичный
# $label — название стиля.
view-style-xray-suffix = { $label } · Рентген
view-parallel = Параллельная
view-perspective = Перспектива
# $view — название стандартного вида, $projection — Параллельная / Перспектива.
view-label = { $view } · { $projection }
view-parallel-projection = Параллельная проекция
view-fov = Угол обзора
view-fov-tip = Вертикальный угол, который видит камера с перспективой (его можно ввести и в инструменте «Масштабирование вида»)
view-zoom-extents = Показать всё
view-viewports = Видовые экраны
view-panes-1 = Один вид
view-panes-2 = Два вида рядом
view-panes-4 = Четыре вида (2 × 2)
view-presets = Пресеты видовых экранов
# $label — текущий вид, например «Сверху · Параллельная».
view-hud-view-tip = Вид: { $label } — стандартные виды, перспектива или параллельная проекция, угол обзора, видовые экраны
# $label — текущий стиль.
view-hud-style-tip = Стиль: { $label } — как рисуются грани и рёбра (также Вид → Стиль граней)
view-xray = Рентген
view-edges = Рёбра
view-profiles = Контуры
view-show-axes = Оси
view-show-ground = Земля
view-show-sky = Небо
view-show-grid = Сетка
view-show-guides = Направляющие
view-show-shadows = Тени
view-show-section-planes = Секущие плоскости
view-show-section-cuts = Разрезы сечением
view-show-section-fill = Заливка сечений
view-show-model-header = Модель (все виды)

## Пресеты видовых экранов

view-preset-front-iso = Спереди + Изометрия
view-preset-top-iso = Сверху + Изометрия
view-preset-front-right = Спереди + Справа
view-preset-front-left-top-iso = Спереди, Слева, Сверху + Изометрия
view-preset-top-front-right-iso = Сверху, Спереди, Справа + Изометрия
view-preset-front-back-left-right = Спереди, Сзади, Слева, Справа

## Сообщения видового экрана

view-nothing-rendered = ничего не отрисовано
view-no-renderer = 3D-отрисовка недоступна
# $error — причина.
view-renderer-unavailable-yet = 3D-отрисовка пока недоступна ({ $error })
view-renderer-unavailable = 3D-отрисовка недоступна (нужен бэкенд wgpu)
view-group = Группа
# $path — цепочка имён групп, например «Стол › Ножка».
view-editing = Правка: Модель › { $path }   (Esc — закрыть)
view-hud-show-tip = Показать: оси, землю, небо, сетку — в этом виде; направляющие, тени, секущие плоскости, разрезы и заливку — во всех видах (также меню Вид)

## ======== Панель инструментов ========

toolbar-size-small = Маленькие
toolbar-size-medium = Средние
toolbar-size-large = Большие

## Заголовки резерва

toolbar-group-draw = Рисование
toolbar-group-tools = Инструменты
toolbar-group-camera = Камера
toolbar-group-file = Файл
toolbar-group-edit = Правка
toolbar-group-snapping = Привязки
toolbar-group-window = Окно

## Кнопки

toolbar-snapping-menu = Меню привязок
toolbar-separator = Разделитель
toolbar-snapping = Привязки
# $state — "off", "some" или "on".
toolbar-snapping-tip = Выберите, к каким точкам и направлениям привязывается курсор. { $state ->
        [off] Привязки выключены.
        [some] Часть привязок выключена.
       *[on] Все привязки включены.
    }

## ======== Строка состояния ========

status-grid-step = Шаг сетки
status-measurements-tip = Поле значений: просто введите число и нажмите Enter (например, 2500, 2.5m, 300,200, 24s)

## ======== Рабочие плоскости ========

workplane-title = Рабочие плоскости
workplane-none = Рабочих плоскостей нет. Добавьте через Инструменты → Направляющая плоскость.
workplane-active-tip = Активная: рисование идёт на этой плоскости
workplane-visible = Видима
workplane-deactivate = Отключить
workplane-no-active-hint = Без активной плоскости используются оси рисования.
# $name — имя плоскости.
workplane-world = { $name } (мировая)
workplane-origin = Начало
workplane-angles = Наклон / азимут / поворот
workplane-tilt-tip = Наклон: угол нормали от синей оси
workplane-azimuth-tip = Азимут: направление наклона, от красной оси
workplane-spin-tip = Поворот вокруг нормали
workplane-undo-delete = Удалить рабочую плоскость
workplane-undo-edit = Правка рабочих плоскостей

## ======== Листы и векторный экспорт ========

layout-credits-none = Авторство: нет загруженных моделей
# $lines — по строке на каждую загруженную модель.
layout-credits = Авторство:
    { $lines }
layout-col-name = Имя
layout-col-qty = Кол-во
layout-col-size = Размер
layout-col-material = Материал
layout-no-parts = Нет групп и компонентов
layout-scale-as-shown = Как на экране
# $n — знаменатель масштаба.
layout-scale-label = Масштаб 1:{ $n }
layout-tb-author = Автор
layout-tb-date = Дата
layout-tb-scale = Масштаб
layout-tb-sheet = Лист
# $n — номер листа, $total — число листов.
layout-tb-sheet-of = { $n } из { $total }

## ======== Настройки ========

prefs-language = Язык
# { $language } — название языка системы.
prefs-language-system = Как в системе ({ $language })
prefs-language-note = Свои переводы: положите <code>.ftl в { DATA_DIR() }/locales/ (см. docs/LOCALIZATION.md); так же можно переопределить отдельные тексты встроенного языка.

prefs-section-general = Общие
prefs-section-drawing = Рисование
prefs-section-libraries = Библиотеки
prefs-section-downloads = Загрузки
prefs-section-view = Вид
prefs-section-navigation = Навигация
prefs-section-interface = Интерфейс
prefs-section-shortcuts = Сочетания клавиш
prefs-section-toolbar = Панель инструментов
prefs-section-render = Рендер

## Слова для поиска по разделам (кроме их названий; перевод перечисляет свои)

prefs-words-general = автосохранение минуты шаблон новый документ единицы отмена шаги история резервная копия предыдущая версия мост python mcp ии агент скрипты
prefs-words-render = рендер изображение размер сэмплы отскоки качество шумоподавление программы gif анимация oidn open image denoise
prefs-words-drawing = привязки привязка конечная точка середина ребро грань ось сетка шаг шаги перемещение поворот масштаб угол
prefs-words-libraries = компоненты материалы папка примеры в модели плитки маленькие большие список
prefs-words-downloads = следить загрузки папка браузер
prefs-words-interface = тема светлая тёмная системная оформление масштаб размер значки панель инструментов язык перевод запуск стартовый экран приветствие
prefs-words-navigation = мышь трекпад вращение масштабирование панорама скорость инвертировать колесо курсор средняя кнопка два пальца жест щипок поворот жест наклон мультитач тачпад
prefs-words-shortcuts = клавиатура сочетания клавиш горячие клавиши
prefs-words-toolbar = панель инструментов кнопки порядок резерв разделитель настройка инструменты
prefs-words-view = новый документ стиль с заливкой текстуры монохром скрытые линии каркас реалистичный рёбра контуры оси земля небо сетка тени перспектива параллельная проекция угол обзора камера
prefs-section-cut-list = Раскрой
prefs-words-cut-list = раскрой заготовки шаблоны лист доска брус фанера пиломатериал цена обрезки остатки погонаж плинтус круг профиль
prefs-cut-presets = Шаблоны заготовок
prefs-cut-presets-note = Ваши листы и доски — появляются в Редакторе материала → Заготовки → + Добавить заготовку. Добавьте здесь или сохраните заготовку материала кнопкой ☆.
prefs-cut-none = Шаблонов пока нет.
prefs-cut-remove = Удалить шаблон
prefs-cut-add-sheet = + Лист
prefs-cut-add-board = + Доска
prefs-cut-add-linear = + Погонаж
prefs-cut-offcuts = Мои обрезки
prefs-cut-offcuts-note = Остатки, сохранённые кнопкой Раскрой → Сохранить обрезки. Раскрой сначала берёт детали из них — по имени материала и заготовке.
prefs-cut-offcuts-none = Обрезков пока нет.
prefs-cut-use-offcuts = Использовать в раскрое
prefs-cut-offcut-remove = Удалить этот обрезок
prefs-cut-offcuts-clear = Удалить все

## Окно

prefs-title = Настройки
prefs-search = Поиск
prefs-nothing-found = Ничего не найдено
prefs-reset = Сбросить
prefs-reset-tip = Вернуть настройки этого раздела по умолчанию
# $section — название раздела.
prefs-defaults-flash = Настройки → { $section }: по умолчанию

## Строки с папкой или программой

# $path — папка.
prefs-folder-default = { $path } (по умолчанию)
prefs-file-found = { $path } (найдено)
prefs-choose = Выбрать…
prefs-default = По умолчанию
prefs-show-in-finder = Показать в файловом менеджере
    .macos = Показать в Finder
    .windows = Показать в Проводнике

## Общие

prefs-data-folder = Папка данных
prefs-data-folder-note = Здесь хранятся плагины, библиотеки, шаблоны, ваши переводы и мост для Python-скриптов и ИИ-агентов (bridge.json, python/).
prefs-autosave = Автосохранение
prefs-autosave-every = Автосохранение каждые
prefs-minutes = мин.
prefs-autosave-note = Несохранённые изменения записываются в { DATA_DIR() }/autosave, а не рядом с файлом.
prefs-new-document = Новый документ
prefs-template-note = Шаблон для Файл → Новый (единицы и масштаб); вид задаётся в разделе «Вид».
prefs-undo = Отмена
prefs-keep = Хранить
prefs-undo-steps = шагов отмены на документ
prefs-undo-note = Каждый шаг хранит копию модели: большие модели с многими шагами занимают больше памяти.
prefs-gpu = Графический API
prefs-gpu-auto = Автоматически
prefs-gpu-gl-software = OpenGL, программный (llvmpipe): медленно, для виртуального GPU, который рисует неправильно
prefs-gpu-restart = Перезапустить
prefs-gpu-in-use = Используется: { $api }
prefs-gpu-fell-back = { $api } здесь не работает; другой API выбран автоматически.
prefs-gpu-note = «Автоматически» при запуске проверяет каждый API в отдельном процессе и берёт первый рабочий (видеокарту раньше программного рендера); API, который однажды не смог рисовать, пропускается до следующей версии. Новый выбор действует после перезапуска; документ переносится, вместе с несохранёнными изменениями (но без истории отмены).
prefs-gpu-env = Задано переменной окружения WGPU_BACKEND.
prefs-gpu-restart-failed = Не удалось перезапустить: { $error }
prefs-appimage-menu = Меню приложений (AppImage)
prefs-appimage-add = Добавить в меню приложений
prefs-appimage-remove = Убрать из меню приложений
prefs-appimage-note = Пункт меню, иконка и тип файлов .vrst для этого AppImage, только для вас (в ~/.local/share). Если переместили файл AppImage, запустите его один раз — пункт меню обновится.
appimage-offer-title = Добавить Varstotas в меню приложений?
appimage-offer-text = Этого AppImage нет в меню приложений, поэтому в доке стандартная иконка, а файлы .vrst не открываются в Varstotas. Добавить пункт меню, иконку и тип файлов .vrst для вас (в ~/.local/share)? Убрать их можно в Preferences → General.
appimage-offer-add = Добавить в меню
appimage-offer-no = Нет, спасибо
appimage-added = Varstotas добавлен в меню приложений
appimage-removed = Varstotas убран из меню приложений
appimage-failed = Не удалось изменить пункт меню приложений: { $error }
prefs-bridge = Скрипты и ИИ-агенты
prefs-bridge-external = Разрешить подключение скриптам и ИИ-агентам вне Varstotas (MCP)
prefs-bridge-note = Выключено: к мосту обращаются только скрипты, запущенные из Varstotas (Файл → Запустить скрипт…, плагины). Включено: в { DATA_DIR() }/bridge.json записываются локальный адрес и токен для других программ, например MCP-сервера для ИИ-ассистента. Мост слушает только 127.0.0.1.

## Рендер

prefs-new-documents = Новые документы
prefs-match-view = По форме вида
prefs-match-view-tip = Высота следует форме видового экрана
prefs-denoise = Шумоподавление при остановке рендера
prefs-denoise-ai = Через Open Image Denoise, если установлен
prefs-use-current-doc = Взять из документа
prefs-use-current-doc-tip = Взять настройки рендера открытого документа
prefs-apply-current-doc = Применить к документу
prefs-apply-current-doc-tip = Открытый документ получит эти настройки (Отменить вернёт прежние)
prefs-render-note = Файл → Новый начинает с этих настроек; Окно → Рендер меняет настройки открытого документа.
prefs-programs = Программы
prefs-oidn = Библиотека Open Image Denoise (шумоподавление рендера)
prefs-oidn-missing = не найдена: шумоподавление выполнит встроенный фильтр (установите Open Image Denoise: brew install open-image-denoise или пакет вашей системы)

## Рисование

prefs-grid = Сетка
prefs-show-grid = Показывать сетку
prefs-grid-step = шаг
prefs-grid-major = крупная через
prefs-grid-note = При отдалении шаг умножается на число делений крупной сетки. При активном Выделении ввод длины задаёт шаг.
prefs-snapping = Привязки и шаги
prefs-snapping-note = Те же переключатели, что и в меню «Привязки».

## Библиотеки

prefs-components = Компоненты
prefs-library-folder = Папка библиотеки
prefs-components-folder-tip = Компоненты (.vrst) и генераторы (.rhai) карточки «Компоненты»
prefs-show-examples = Показывать примеры
prefs-show-examples-tip = Встроенные примеры генераторов
prefs-show-in-model = Показывать в модели
prefs-show-in-model-tip = Компоненты и генераторы открытого документа
prefs-materials = Материалы
prefs-materials-folder-tip = Материалы (.vrstmat) карточки «Материалы»; подпапки — категории
prefs-show-mat-examples-tip = Материалы, поставляемые с Varstotas (демо «Галерея материалов»)
prefs-tiles = Плитки
prefs-tiles-note = Для обеих карточек; также в их меню «…».

## Загрузки

prefs-watch-downloads = Следить за загрузками
prefs-watch-offer = Предлагать импорт новых файлов моделей
prefs-watch-offer-tip = Каждый новый файл модели или архив в папке предлагается к импорту (всегда включено, пока ждёт страница «Другие сайты»)
prefs-watched-folder = Отслеживаемая папка
prefs-watched-folder-tip = Куда браузер сохраняет файлы
prefs-downloads-note = Также в Окно → Онлайн-библиотека 3D → Другие сайты.

## Панель инструментов

prefs-toolbar-note = Кнопки панели инструментов по порядку; остальные ждут в резерве (они остаются в меню).
prefs-toolbar-on = На панели инструментов
prefs-toolbar-reserve = Резерв
prefs-toolbar-to-reserve = В резерв
prefs-toolbar-down = Вниз
prefs-toolbar-up = Вверх
prefs-toolbar-to-end = В конец панели инструментов

## Интерфейс

prefs-theme = Тема
prefs-theme-light = Светлая
prefs-theme-dark = Тёмная
prefs-theme-system = Как в системе
prefs-theme-note = Видовой экран сохраняет свои цвета (небо, земля); листы и развёртки остаются белой бумагой.
prefs-scale = Масштаб
prefs-scale-note = Меню, панели и окна; %CMDK%+ / %CMDK%− / %CMDK%0 тоже меняют его.
prefs-toolbar-icons = Значки панели инструментов

## Сочетания клавиш

prefs-shortcuts-note = Щёлкните сочетание и нажмите новые клавиши: Esc — отмена, Backspace — убрать сочетание. Клавиша, которая была у другой команды, забирается у неё.
# $command — название команды.
prefs-shortcut-removed = { $command }: без сочетания
# $keys — нажатое сочетание, $why — причина.
prefs-shortcut-reserved = { $keys } не может быть сочетанием: { $why }
# $other — команда, у которой было это сочетание.
prefs-shortcut-taken = { $command }: { $keys } (забрано у «{ $other }»)
prefs-shortcut-set = { $command }: { $keys }
prefs-shortcuts-tools = Инструменты
prefs-press-keys = Нажмите клавиши…
prefs-default-none = По умолчанию: нет
# $keys — список сочетаний по умолчанию.
prefs-default-keys = По умолчанию: { $keys }

## Навигация

prefs-speed = Скорость
prefs-nav-orbit = Вращение
prefs-nav-pan = Панорама
prefs-nav-zoom = Масштабирование
prefs-invert-wheel = Инвертировать колесо
prefs-invert-wheel-tip = Колесо от себя отдаляет вид
prefs-zoom-cursor = Масштабировать к курсору
prefs-zoom-cursor-tip = Выкл.: к центру вида
prefs-mouse = Мышь
prefs-middle-drag = Перетаскивание средней кнопкой:
prefs-shift-other = Shift включает второе действие.
prefs-trackpad = Трекпад
prefs-two-fingers = Два пальца:
prefs-trackpad-note = Shift меняет вращение и панораму местами. Щипок и %CMD% + два пальца масштабируют независимо от колеса мыши.

## Вид (значения по умолчанию для нового документа)

prefs-view-note = Как выглядит новый документ (Файл → Новый, Новый из шаблона). Меню «Вид» и кнопки видового экрана меняют текущий вид.
prefs-camera = Камера
prefs-fov = Угол обзора
prefs-use-current-view = Взять из текущего вида
prefs-use-current-view-tip = Взять стиль, переключатели и камеру активного видового экрана
prefs-apply-current-view = Применить к текущему виду
prefs-apply-current-view-tip = Настроить активный видовой экран как новый документ

## ======== Сочетания клавиш ========

key-new = Новый
key-open = Открыть…
key-save = Сохранить
key-save-as = Сохранить как…
key-quit = Выход
key-undo = Отменить
key-redo = Повторить
key-select-all = Выделить всё
key-select-none = Снять выделение
key-make-group = Сгруппировать
key-zoom-extents = Показать всё
key-toggle-snapping = Привязки вкл. / выкл.
key-next-scene = Следующая сцена
key-prev-scene = Предыдущая сцена

## Названия клавиш и зарезервированные клавиши (Настройки → Сочетания клавиш)

key-name-space = Пробел
key-name-pgdn = PgDn
key-name-pgup = PgUp
key-reserved-tools = эта клавиша используется всеми инструментами
key-reserved-arrows = стрелки фиксируют оси привязки
key-reserved-f1 = F1 открывает руководство
key-reserved-digits = цифры вводятся в поле значений
key-reserved-measurements = эта клавиша вводится в поле значений
key-reserved-clipboard = Вырезать / Копировать / Вставить

## ======== Шаблоны ========

template-architectural-mm = Архитектурный — миллиметры
template-architectural-m = Архитектурный — метры
template-woodworking-mm = Столярный — миллиметры
template-imperial = Британский — дюймы

template-architectural-mm-desc = Здания и интерьеры, длины в миллиметрах.
template-architectural-m-desc = Здания и участки, длины в метрах.
template-woodworking-mm-desc = Мебель и мелкие предметы: вид крупным планом, длины в миллиметрах.
template-imperial-desc = Длины в дюймах.

## ======== Боковая панель ========

## Карточки (названия — идентификаторы в настройках; здесь показываемые имена)

tray-card-entity-info = Свойства объекта
tray-card-materials = Материалы
tray-card-material-editor = Редактор материала
tray-card-components = Компоненты
tray-card-tags = Теги
tray-card-outliner = Структура
tray-card-scenes = Сцены
tray-card-shadows = Тени
tray-card-environment = Окружение
tray-card-soften-edges = Сглаживание рёбер
tray-card-instructor = Подсказки
tray-card-model-info-tip = Сведения о документе: статистика документа, Удалить неиспользуемое
tray-cards-all-off = Все карточки выключены: Окно → Боковая панель

tray-untagged = Без тега
tray-default = По умолчанию
tray-rename = Переименовать
tray-invalid-value = Неверное значение
# $error — причина.
tray-kernel-not-ready = (ядро не готово: { $error })
tray-through = Насквозь

## Тени

tray-shadows-show = Показывать тени
tray-shadows-time = Время
tray-shadows-date = Дата
tray-shadows-light = Светлее
tray-shadows-dark = Темнее
tray-shadows-display = Показ
tray-shadows-on-faces = На гранях
tray-shadows-on-ground = На земле
tray-shadows-location = Местоположение
tray-shadows-latitude = Широта
tray-shadows-longitude = Долгота
tray-shadows-utc-offset = Смещение UTC
tray-shadows-north-angle = Угол севера
# $alt — высота солнца в градусах.
tray-sun-above = Солнце на { $alt }° над горизонтом
tray-sun-below = Солнце под горизонтом

## Сглаживание рёбер

tray-soften-allow-normals = Допустимый угол между нормалями
tray-soften-smooth-normals = Сгладить нормали
tray-soften-coplanar = Сглаживать компланарные
tray-soften-select = Выберите рёбра или грани
# $n — число рёбер.
tray-soften-edges-count = { $n ->
        [one] { $n } ребро
        [few] { $n } ребра
        [many] { $n } рёбер
       *[other] { $n } рёбер
    }

## Свойства объекта

tray-info-invalid-context = Неверный контекст
tray-info-no-selection = Нет выделения
tray-info-units = Единицы
tray-unit-millimeters = Миллиметры
tray-unit-centimeters = Сантиметры
tray-unit-meters = Метры
tray-unit-inches = Дюймы
tray-unit-feet = Футы
tray-info-faces = Грани
tray-info-edges = Рёбра
tray-info-groups = Группы
tray-info-components = Компоненты
tray-info-face = Грань
tray-info-area = Площадь
tray-info-front = Лицевая
tray-info-back = Обратная
tray-info-tag = Тег
tray-info-edge = Ребро
tray-info-length = Длина
tray-info-soft = Мягкое
tray-info-smooth = Гладкое
tray-info-group = Группа
tray-info-component = Компонент
tray-info-parametric-group = Параметрическая группа
# $kind — Группа, Компонент или Параметрическая группа.
tray-info-solid = Тело: { $kind }
tray-info-volume = Объём
tray-info-instance = Экземпляр
tray-info-name-hint = имя
tray-info-definition = Определение
tray-info-material = Материал
tray-info-source = Источник
tray-info-licence-unknown = лицензия неизвестна
# $line — строка авторства модели.
tray-info-source-hover =
    { $line }
    Файл → Экспорт → Авторство перечисляет авторов модели
# $count — число выделенных элементов.
tray-info-n-guides = { $count ->
        [one] { $count } направляющая
        [few] { $count } направляющие
        [many] { $count } направляющих
       *[other] { $count } направляющих
    }
tray-info-n-dimensions = { $count ->
        [one] { $count } размер
        [few] { $count } размера
        [many] { $count } размеров
       *[other] { $count } размеров
    }
tray-info-n-entities = { $count ->
        [one] { $count } объект
        [few] { $count } объекта
        [many] { $count } объектов
       *[other] { $count } объектов
    }
tray-info-contains = Содержит
tray-count-faces = { $n ->
        [one] { $n } грань
        [few] { $n } грани
        [many] { $n } граней
       *[other] { $n } граней
    }
tray-count-edges = { $n ->
        [one] { $n } ребро
        [few] { $n } ребра
        [many] { $n } рёбер
       *[other] { $n } рёбер
    }
tray-count-groups = { $n ->
        [one] { $n } группа
        [few] { $n } группы
        [many] { $n } групп
       *[other] { $n } групп
    }
tray-count-components = { $n ->
        [one] { $n } компонент
        [few] { $n } компонента
        [many] { $n } компонентов
       *[other] { $n } компонентов
    }
tray-count-guides = { $n ->
        [one] { $n } направляющая
        [few] { $n } направляющие
        [many] { $n } направляющих
       *[other] { $n } направляющих
    }
tray-count-dimensions = { $n ->
        [one] { $n } размер
        [few] { $n } размера
        [many] { $n } размеров
       *[other] { $n } размеров
    }

## Названия шагов отмены

tray-undo-rename = Переименовать
tray-undo-soften-edge = Сгладить ребро
tray-undo-unsoften-edge = Убрать сглаживание ребра
tray-undo-set-tag = Задать тег
tray-undo-texture-position = Положение текстуры
tray-undo-edit-material = Правка материала
tray-undo-edit-segments = Правка сегментов
tray-undo-edit-radius = Правка радиуса
tray-undo-edit-text = Правка текста
tray-undo-edit-dimension-text = Правка текста размера
tray-undo-rename-section = Переименовать сечение
tray-undo-delete-tag = Удалить тег
tray-undo-rename-tag = Переименовать тег
tray-undo-add-tag = Добавить тег
tray-undo-scene-properties = Свойства сцены
tray-undo-rename-scene = Переименовать сцену

## Параметрическая группа

tray-params-make = Сделать параметрической
tray-params-make-hover = Размер и копии задаются в Свойствах объекта
tray-params-title = Параметры
tray-params-length = Длина (X)
tray-params-width = Ширина (Y)
tray-params-height = Высота (Z)
tray-params-gap = Зазор
tray-params-copies = Копии
tray-params-along = Вдоль
tray-params-resize = Изменить размер
tray-params-stretch = Растянуть
tray-params-stretch-hover = Сдвигает дальнюю половину: концы сохраняют размер
tray-params-scale = Масштаб

## Текстура грани

tray-tex-title = Текстура
tray-tex-offset-x = Смещение X
tray-tex-offset-y = Смещение Y
tray-tex-rotation = Поворот
tray-tex-scale-x = Масштаб X
tray-tex-scale-y = Масштаб Y
tray-tex-reset = Сбросить
tray-tex-tile-w = Ширина плитки
tray-tex-tile-h = Высота плитки
uv-title = UV-редактор
uv-undo = Правка UV
uv-undo-mark-seams = Отметить швы
uv-undo-clear-seams = Снять швы
uv-seams-marked = Швами отмечено рёбер: { $count }.
uv-seams-cleared = Снято швов: { $count }.
uv-no-edges = Сначала выделите рёбра (вместе с гранями).
uv-nothing = Нечего менять.
uv-empty = Выделите грани с текстурой (или покрасьте их), чтобы править раскладку текстуры.
uv-unfold-hover = Разложить каждый остров плоско в натуральную величину, разрезав по швам
uv-planar = Плоская
uv-box = Кубическая
uv-cylinder = Цилиндрическая
uv-sphere = Сферическая
uv-seams-mark-hover = Отметить выделенные рёбра как швы
uv-seams-clear-hover = Снять швы с выделенных рёбер (иначе все)
uv-seams-hard-hover = Отметить швом каждое жёсткое (не смягчённое) ребро между гранями
uv-rotate-ccw = Повернуть на 90° против часовой
uv-rotate = Повернуть на 90° по часовой
uv-flip-u = Отразить слева ↔ направо
uv-flip-v = Отразить сверху ↕ вниз
uv-fit-hover = Масштабировать каждый остров на одну плитку
uv-pack-hover = Разложить острова в одной плитке в одном масштабе
uv-align-hover = Повернуть каждый остров, чтобы его самое длинное ребро шло прямо
uv-reset-hover = Сбросить положение текстуры на островах (вернуть исходное наложение)
uv-tool-select = Выбор: щелчок — остров (в режиме граней — грань), Shift+щелчок добавляет или убирает, рамка — протянуть по пустому месту; перетаскивание выбранного двигает его
uv-tool-move = Двигать выбранные острова или грани (красная / зелёная стрелка гизмо — вдоль одной оси)
uv-tool-rotate = Поворачивать выбранные острова или грани вокруг опорной точки
uv-tool-scale = Масштабировать выбранные острова или грани от опорной точки (красный / зелёный квадрат гизмо — по одной оси)
uv-mode-island = Выбирать острова целиком
uv-mode-face = Выбирать отдельные грани
uv-select-all = Выбрать всё
uv-frame = Показать все острова
uv-project-hover = Проекция текстуры: плоская, коробка, цилиндр или сфера
uv-reset-rotation = Сбросить поворот: развернуть острова так, чтобы текстура шла ровно (форма развёртки сохраняется)
uv-reset-scale = Сбросить масштаб: вернуть текстуре её собственный размер (форма развёртки сохраняется)
uv-hint-select = Щелчок / рамка: выбор · щелчок ещё раз: та, что ниже · Shift: добавить или убрать · тянуть выбранное: двигать · колесо: масштаб · правая кнопка: панорама
uv-hint-move = Тянуть: двигать выбранное · стрелки гизмо: по одной оси · прилипает к углам и сетке тайлов (Alt: без прилипания; Shift: шаги вкл/выкл) · ввести dx; dy
uv-hint-rotate = Тянуть: поворот вокруг опорной точки · центр гизмо можно перетащить, двойной щелчок — в центр · Shift: шаг угла вкл/выкл · ввести угол
uv-hint-scale = Тянуть: масштаб от опорной точки · красный / зелёный квадрат: одна ось, оранжевый: обе · центр гизмо можно перетащить, двойной щелчок — в центр · Shift: шаги вкл/выкл · ввести k или kx; ky
uv-value-move = Сдвиг
uv-value-rotate = Угол
uv-value-scale = Масштаб
uv-value-hover = Точное значение, Enter применяет; сразу после перетаскивания заменяет его
uv-checker-hover = Показать шахматку вместо текстур (здесь и в 3D-виде), чтобы видеть растяжения
uv-density = Плотность текселей
uv-px-per-m = пикс/м
uv-summary = Граней: { $faces } · островов: { $islands } · выбрано: { $selected }
uv-material = Материал: { $name }
uv-map-hover = Какая карта материала показана под развёрткой
uv-no-texture = Нет текстуры: у граней и текущего материала нет изображения
uv-tile-hover = Повторять текстуру по всему полю (иначе — одна плитка)
tray-tex-width = Ширина текстуры
tray-tex-height = Высота текстуры
tray-tex-keep-aspect = Сохранять пропорции

## Кривые

tray-curve = Кривая
tray-curve-closed = Замкнутая кривая
tray-curve-points = Точки
# $count — число точек, $smooth — сколько из них гладкие.
tray-curve-points-value = { $count } (гладких: { $smooth })
tray-curve-segments = Сегменты
tray-curve-segments-hover = Рёбер на изогнутый участок между двумя точками
tray-curve-edit = Правка кривой
tray-curve-edit-hover = Перетаскивайте её точки и рычаги (или дважды щёлкните кривую)
tray-curve-arc = Дуга
tray-curve-circle = Окружность
tray-curve-polygon = Многоугольник
tray-curve-radius = Радиус
tray-curve-sides = Стороны
tray-curve-circumference = Длина окружности
tray-curve-arc-length = Длина дуги
tray-curve-angle = Угол

## Размер, свет, секущая плоскость, направляющие

tray-dim-text = Текст
tray-dim-linear = Линейный размер
tray-light-title = Источник света
tray-light-render = Рендер…
tray-light-render-hover = Окно → Рендер
tray-light-hint = Перемещение / Поворот двигают и вращают его; тащите квадратный маркер, чтобы нацелить.
tray-section-title = Секущая плоскость
tray-section-name = Имя
tray-section-active-cut = Активный разрез
tray-section-normal = Нормаль
tray-section-reverse = Обратить
tray-section-align-view = Выровнять вид
tray-guide-line = Направляющая линия
tray-guide-direction = Направление
tray-guide-point = Направляющая точка
tray-guide-position = Положение

## Карточка материалов

tray-mat-current = Текущий материал заливки (B)
tray-mat-custom = Свой
tray-mat-use = Использовать

## Теги

tray-tags-add = Добавить тег
tray-tags-header = Видимость · Имя
tray-tags-untagged-hover = Переключает видимость геометрии, групп и компонентов верхнего уровня без тега (их содержимое принадлежит им)
tray-tags-toggle = Переключить видимость
tray-tags-rename-hover = Дважды щёлкните, чтобы переименовать; правая кнопка — больше
tray-tags-delete-hover = Удалить тег (то, что им помечено, станет без тега)
# $n — номер нового тега.
tray-tag-new-name = Тег { $n }
# $name — тег, $uses — сколько элементов было им помечено.
tray-tag-deleted = Тег «{ $name }» удалён
tray-tag-deleted-uses = Тег «{ $name }» удалён: элементов без тега теперь — { $uses }

## Структура

tray-outliner-empty = Групп и компонентов пока нет.
tray-outliner-hide-show = Скрыть / показать
tray-outliner-zoom = Показать
tray-outliner-zoom-hover = Выделить и показать в виде (двойной щелчок)
tray-outliner-edit-group = Править группу
tray-outliner-edit-component = Править компонент
tray-outliner-unhide = Показать
tray-outliner-hide = Скрыть
tray-outliner-explode = Разбить
tray-outliner-make-unique = Сделать уникальным
tray-outliner-make-unique-hover = Собственное определение: его правка больше не меняет другие копии
tray-outliner-move-out = Вынести из группы
tray-outliner-move-out-hover = В группу / компонент вокруг этого (или перетащите строку)

## Сцены

tray-scenes-add = Добавить сцену из текущего вида
tray-scenes-update = Обновить выбранную сцену по текущему виду
tray-scenes-delete = Удалить выбранную сцену
tray-scenes-up = Сдвинуть сцену вверх
tray-scenes-down = Сдвинуть сцену вниз
tray-scenes-play = Воспроизвести
tray-scenes-pause = Пауза
tray-scenes-play-hover = Воспроизвести анимацию сцен (Вид → Анимация)
tray-scenes-empty = Сцен нет. Нажмите +, чтобы сохранить текущий вид.
tray-scenes-click-hover = Щёлкните, чтобы перейти к сцене, дважды — чтобы переименовать
tray-scenes-include = Включить в анимацию
tray-scenes-animation = Анимация
tray-scenes-transitions = Переходы между сценами
tray-scenes-transition = Переход
tray-scenes-delay = Задержка сцены

## Подсказки

# $key — сочетание клавиш.
tray-instructor-shortcut = Сочетание: { $key }
tray-instructor-more = Подробнее в руководстве (F1)
tray-instructor-not-implemented = Этот инструмент пока не реализован.

## Карточки библиотек, общая рамка (tray_lib)

tray-tiles-small = Маленькие плитки
tray-tiles-large = Большие плитки
tray-tiles-list = Список
tray-lib-search = Поиск
tray-lib-refresh = Обновить: перечитать папку библиотеки
tray-lib-more = Папка библиотеки, что показывать, размер плиток
tray-lib-section-show = Показать раздел
tray-lib-section-fold = Свернуть раздел
# $unused — число неиспользуемых элементов.
tray-purge-materials = Удалить неиспользуемое: убрать материалы, которые не использует модель (их { $unused })
tray-purge-materials-none = Удалить неиспользуемое: неиспользуемых материалов нет
tray-purge-components = Удалить неиспользуемое: убрать компоненты, которые не использует модель (их { $unused })
tray-purge-components-none = Удалить неиспользуемое: неиспользуемых компонентов нет
tray-lib-fav-add = Добавить в избранное
tray-lib-fav-remove = Убрать из избранного
tray-lib-move-to-folder = Переместить в папку
tray-my-library = Моя библиотека
# $path — файл, который мешает.
tray-lib-exists = { $path } уже существует
tray-folder-new = Новая папка…
tray-folder-rename = Переименовать папку…
tray-folder-delete = Удалить папку…
tray-folder-delete-files = Удалить папку вместе с файлами…

## ======== Библиотека компонентов ========

## Разделы

library-examples = Примеры
library-examples-hover = Генераторы, поставляемые с Varstotas (не файлы)
library-in-model = В модели
library-in-model-hover = Компоненты и генераторы открытого документа («Сохранить в библиотеку» создаёт файл)
library-favourites = Избранное
library-favourites-hover = Правая кнопка на плитке библиотеки → Добавить в избранное
library-recent = Недавние
library-recent-hover = Последние размещённые компоненты библиотеки
library-hide-examples = Скрыть встроенные примеры (… → Показывать примеры вернёт их)
library-hide-in-model = Скрыть компоненты документа (… → Показывать в модели вернёт их)
# $path — папка.
library-folder-reveal-hover =
    { $path }
    Щёлкните, чтобы показать в Finder
    .windows =
    { $path }
    Щёлкните, чтобы показать в Проводнике
library-show-in-finder = Показать в Finder
    .windows = Показать в Проводнике
    .linux = Показать в файловом менеджере

## Верхняя строка и меню «…»

library-plus-hover = Сохранить выделение в библиотеку, новый генератор
library-online-hover = Онлайн-библиотека 3D: бесплатные модели из сети (Poly Haven, Smithsonian, NASA, Kenney, плагины)
library-save-selection = Сохранить выделение в библиотеку…
library-select-first = Сначала что-нибудь выделите
library-new-generator = Новый генератор…
library-new-generator-hover = Компонент, построенный скриптом с параметрами (Окно → Редактор генераторов)
library-choose-folder = Выбрать папку…
library-default-folder = Папка по умолчанию
library-default-folder-hover = Вернуть { DATA_DIR() }/components
library-purge = Удалить неиспользуемые компоненты
library-purge-hover = Убрать компоненты (и определения групп), которые не использует модель
library-show-examples = Показывать примеры
library-show-examples-hover = Встроенные примеры генераторов
library-show-in-model = Показывать в модели
library-show-in-model-hover = Компоненты и генераторы открытого документа
library-empty = Компонентов пока нет: выделите что-нибудь и выберите + → Сохранить выделение в библиотеку.
library-nothing-found = Ничего не найдено
library-image-filter = Изображение
library-name = Имя
library-save = Сохранить
# $n — число элементов.
library-unreadable-files = Не удалось прочитать файлов: { $n }
library-unreadable-generators = Не удалось прочитать генераторов: { $n }

## Плитки и их контекстные меню

# $name — компонент, $note — дополнительная строка (оканчивается переводом строки или пуста), $x $y $z — размер.
library-tile-hover =
    { $name }
    { $note }{ $x } × { $y } × { $z }
    Щёлкните, чтобы разместить
# $uses — сколько копий размещено.
library-in-document = В этом документе размещено: { $uses }
library-place = Разместить
library-select-instances = Выделить экземпляры
library-preview-from-view = Превью из вида
library-preview-from-view-file-hover = Видовой экран в его нынешнем виде становится картинкой (сохраняется в файле): сначала кадрируйте модель
library-preview-from-view-def-hover = Видовой экран в его нынешнем виде становится картинкой плитки (сохраняется с документом): сначала кадрируйте компонент
library-preview-image = Задать картинку превью…
library-preview-image-file-hover = Картинка, показываемая здесь вместо нарисованной модели (сохраняется в файле)
library-preview-image-def-hover = Картинка, показываемая на плитке вместо отрисованной модели
library-rename-ellipsis = Переименовать…
library-delete-ellipsis = Удалить…
library-save-to-library = Сохранить в библиотеку
library-save-to-library-hover = Записать в папку библиотеки, для любого документа
library-parameters = Параметры…
library-parameters-hover = Окно → Параметры компонента: значения, которые двигают, показывают или красят его части
library-delete-from-document = Удалить из документа…
library-delete-from-document-hover = Компонент и все его размещённые копии

## Картинки превью

library-undo-set-preview = Задать картинку превью
library-undo-remove-preview = Убрать картинку превью
library-undo-set-def-preview = Задать превью компонента
library-undo-remove-def-preview = Убрать превью компонента
library-preview-set-file = Картинка превью задана (сохранена в файле)
library-preview-removed = Картинка превью убрана
# $name — компонент.
library-preview-set-def = Картинка превью для { $name } задана (сохранена с документом)
library-preview-removed-def = Картинка превью для { $name } убрана
# $path — файл картинки, $error — причина.
library-preview-error-file = Картинка превью: { $path }: { $error }
library-preview-error = Картинка превью: { $error }

## Сохранение, размещение, перемещение

# $path — файл, $name — компонент, $error — причина.
library-exists = { $path } уже существует
library-save-error = Сохранение в библиотеку: { $error }
library-saved = Сохранено в библиотеку: { $path }
library-save-failed = Не удалось сохранить в библиотеку: { $error }
library-place-failed = Разместить { $name }: { $error }
library-no-copies = Копий этого компонента на верхнем уровне модели нет
library-move-failed = Переместить в папку: { $error }

## Диалоги папок

library-folder-new-title = Новая папка
library-folder-rename-title = Переименовать папку
library-folder-name = Имя папки
library-create = Создать
library-rename = Переименовать
# $heading — заголовок диалога, $name — папка, $error — причина.
library-folder-done = { $heading }: { $name }
library-folder-failed = { $heading }: { $error }
library-folder-delete-title = Удалить папку?
# $name — папка.
library-folder-delete-text-materials = Папка «{ $name }» будет удалена из библиотеки материалов:
library-folder-delete-text-components = Папка «{ $name }» будет удалена из библиотеки компонентов:
library-folder-empty = Папка пуста.
# $files — число файлов, $dirs — число подпапок.
library-folder-what = Файлов: { $files }
library-folder-what-dirs = Файлов: { $files }, подпапок: { $dirs }
# $what — текст library-folder-what.
library-folder-not-empty = ⚠ Папка не пуста, вместе с ней будет удалено: { $what }.
# $n — сколько файлов не перечислено.
library-folder-more = … и ещё { $n }
library-undone-copies-stay = Это нельзя отменить. Уже размещённые в документах копии остаются.
library-folder-delete-with = Удалить вместе с файлами ({ $files })
library-folder-deleted = Папка { $name } удалена
library-folder-deleted-files = Папка { $name } удалена вместе с файлами ({ $files })
library-folder-delete-failed = Удалить папку: { $error }

## Диалоги удаления

library-delete-title = Удалить из библиотеки?
library-delete-text = «{ $name }» будет удалён из папки библиотеки:
library-delete-doc-title = Удалить из документа?
library-delete-gen-text = Генератор «{ $name }» будет убран из этого документа.
# $uses — число размещённых компонентов.
library-delete-gen-uses = Размещённые компоненты ({ $uses }) станут обычными, без параметров.
library-undo-brings-back = Правка → Отменить вернёт его.
library-undo-brings-them-back = Правка → Отменить вернёт их.
library-undo-delete-generator = Удалить генератор { $name }
# $n — число компонентов, оставшихся без параметров.
library-generator-removed-uses = Генератор { $name } убран, компонентов без параметров: { $n }
library-generator-removed = Генератор { $name } убран
library-deleted = Удалено из библиотеки: { $name }
library-delete-failed = Удалить { $path }: { $error }
library-undo-delete-component = Удалить компонент { $name }
# $n — число удалённых копий.
library-component-deleted-copies = Удалён { $name } и размещённых копий: { $n }
library-component-deleted = Компонент { $name } удалён из документа
library-delete-def-text = Размещений «{ $name }»: { $uses }. Компонент и все его копии будут удалены.

## Диалог переименования

library-rename-material = Переименовать материал
library-rename-component = Переименовать компонент
# $file — файл библиотеки.
library-rename-file-too = Файл библиотеки { $file } тоже переименовывается.
library-undo-rename-component = Переименовать компонент { $name }
library-renamed = Переименовано в { $name }
library-rename-failed = Переименовать { $path }: { $error }

## ======== Материалы ========

## Карточка библиотеки

mat-plus-hover = Новый материал из изображения, импорт файлов материалов
mat-online-hover = Онлайн-библиотека материалов: бесплатные PBR-материалы (Poly Haven, ambientCG, плагины)
mat-new-from-texture = Новый из текстуры…
mat-new-from-texture-hover = Материал из файла изображения (PNG, JPEG)
mat-new-procedural = Новый процедурный…
mat-new-procedural-hover = Бесшовная текстура по шаблону: кирпич, плитка, доски, шум или шахматка
proc-title = Процедурный материал
proc-bricks = Кирпич
proc-tiles = Плитка
proc-planks = Доски
proc-noise = Шум
proc-checker = Шахматка
proc-main = Цвет
proc-joint = Швы
proc-second = Второй цвет
proc-count = Элементов поперёк × вверх
proc-scale = Масштаб
proc-joint-width = Ширина шва
proc-variation = Разброс
proc-seed = Вариант
proc-another = Другой
proc-width = Ширина плитки
proc-name = Название
proc-hint = Плитка повторяется без швов; в превью — 2 × 2 плитки.
proc-create = Создать материал
proc-plugin-hover = Узор из плагина (@texture_generator)
proc-generating = Генерация…
proc-created = Создан материал { $name }: щёлкайте по граням, чтобы окрасить
mat-import = Импорт…
mat-import-hover = Добавить файлы материалов (.vrstmat, форматы плагинов) в документ
mat-choose-folder = Выбрать папку…
mat-default-folder = Папка по умолчанию
mat-default-folder-hover = Вернуть { DATA_DIR() }/materials
mat-purge = Удалить неиспользуемые материалы
mat-purge-hover = Убрать материалы, которые не использует модель
mat-show-examples-hover = Материалы, поставляемые с Varstotas (демо «Галерея материалов»)
mat-in-model-hover = Материалы открытого документа
mat-favourites-hover = Правая кнопка на материале библиотеки → Добавить в избранное
mat-recent-hover = Последние использованные материалы библиотеки
mat-examples-hover = Материалы, поставляемые с Varstotas (демо «Галерея материалов»)
mat-empty = Пусто: щёлкните правой кнопкой материал модели → Сохранить в библиотеку
mat-use = Использовать
mat-use-hover = Красить им (при покраске он добавляется в модель)
mat-export = Экспорт…
mat-save-hover = Записать в папку библиотеки материалов, для любого документа
mat-copy-hover = Скопировать в вашу библиотеку материалов
mat-delete-model-hover = Убрать из модели: то, что им покрашено, получит материал по умолчанию (Отменить вернёт)
mat-replace = Заменить текущим материалом
mat-replace-hover = Перекрасить всё, что окрашено этим материалом, текущим материалом и удалить его из модели (Отменить вернёт его)
mat-undo-replace = Заменить { $name } на { $with }
mat-replaced = { $name } → { $with }: перекрашено { $n }
mat-file-filter = Материал Varstotas
mat-all-filter = Все материалы

## Описание плитки (при наведении)

mat-kind-metal = металл
mat-kind-glass = стекло
mat-kind-emissive = светящийся
mat-kind-texture = текстура
# $name — пресет шейдера.
mat-kind-shader = шейдер { $name }
mat-kind-shader-custom = свой
# $value — шероховатость, от 0 до 1.
mat-kind-roughness = шероховатость { $value }

## Сообщения

mat-undo-add = Добавить материал { $name }
mat-undo-delete = Удалить материал { $name }
mat-undo-rename = Переименовать материал { $name }
mat-saved = Сохранено в библиотеку материалов: { $name }
mat-save-failed = Не удалось сохранить материал: { $error }
mat-exported = Экспортировано: { $path }
mat-export-failed = Не удалось экспортировать материал: { $error }
# $names — импортированные материалы, $plugin — импортёр.
mat-imported-with = Импортировано через { $plugin }: { $names }
mat-import-empty = { $plugin }: в { $path } нет материалов
mat-import-failed-with = Не удаётся импортировать { $path } через { $plugin }: { $error }
mat-imported = Импортирован материал { $name }
mat-import-failed = Не удаётся импортировать { $path }: { $error }
# $n — число окрашенных элементов, вернувшихся к материалу по умолчанию.
mat-deleted-uses = Материал { $name } удалён, использований вернулось к материалу по умолчанию: { $n } (Отменить вернёт)
mat-deleted = Материал { $name } удалён (Отменить вернёт)
mat-delete-failed = Не удаётся удалить материал { $name }
mat-rename-exists = Материал «{ $name }» уже есть
mat-file-exists = { $path } уже существует
mat-delete-title = Удалить из библиотеки материалов?
mat-delete-undone = Это нельзя отменить. Документы, использующие его, сохраняют свою копию.
mat-deleted-from-library = Удалено из библиотеки материалов: { $name }

## Онлайн-библиотека материалов (окно и каталоги)

mat-online-title = Онлайн-библиотека материалов
mat-online-plugins-hint =
    Плагины с @material_source
    перечислены здесь
mat-online-search = Поиск: кирпич, дуб, бетон…
mat-online-description = Бесплатные фотоскан-материалы CC0 (карты цвета, нормалей, шероховатости, окклюзии, металла в реальном масштабе). Загрузки попадают в библиотеку материалов.
mat-online-click-hint = Щёлкните материал, чтобы посмотреть; двойной щелчок скачивает его и красит им.
# $width — длина.
mat-online-wide = шириной { $width }
mat-online-use-hover = Уже в библиотеке: красить им
mat-online-download-use = Скачать и использовать
mat-online-download-use-hover = Сохранить в библиотеку, добавить в модель и покрасить
mat-online-download-hover = Сохранить в библиотеку материалов
# $source — название источника.
mat-online-loading = Загрузка каталога { $source }…
mat-online-load-failed = Не удаётся загрузить каталог: { $error }
# $n — число показанных материалов.
mat-online-count = { $n ->
        [one] { $n } материал
        [few] { $n } материала
        [many] { $n } материалов
       *[other] { $n } материалов
    }
mat-online-load-hover = Запросить у плагина материалы с этими значениями
mat-online-downloaded = { $name } загружен в библиотеку материалов
mat-terms-polyhaven = Работает на Poly Haven (polyhaven.com). Текстуры под CC0: свободно для любого использования, в том числе коммерческого; указывать авторство не нужно.
mat-terms-ambientcg = Материалы ambientCG под CC0: свободно для любого использования, в том числе коммерческого; указывать авторство необязательно.
mat-err-not-catalogue = это не каталог
mat-err-no-colour-map = нет карты цвета

## Редактор материала
mated-window-title = Редактор материала
mated-none = Нет материала: выберите образец или материал из библиотеки либо возьмите его из выделения.
mated-not-in-model = Ещё не в модели: добавится, когда вы покрасите им или измените его.
mated-used = { $count ->
        [one] Используется на { $count } грани или объекте
       *[other] Используется на { $count } гранях и объектах
    }
mated-pick = Взять из выделения
mated-pick-tip = Пипетка: сделать текущим материал выбранной грани или объекта
mated-pick-none = Выделите покрашенную грань или объект, чтобы взять его материал
mated-picked = Текущий материал: { $name }
mated-select-tip = Выделить грани и объекты этого контекста, покрашенные этим материалом
mated-selected = { $count ->
        [one] Выделен { $count } элемент
        [few] Выделено { $count } элемента
        [many] Выделено { $count } элементов
       *[other] Выделено { $count } элемента
    }
mated-window-tip = Открыть редактор в окне
mated-basic = Основное
mated-color = Цвет
mated-opacity = Непрозрачность
mated-texture = Текстура
mated-no-texture = Без текстуры
mated-look = Вид
mated-hue = Оттенок
mated-saturation = Насыщенность
mated-lightness = Яркость
mated-cutoff = Порог прозрачности
mated-colorize = Тонировать
mated-colorize-tip = Окрасить изображение цветом материала, сохранив его яркость
mated-decal = Наклейка
mated-decal-tip = Показать одну плитку без повторов: вокруг неё виден цвет материала
mated-world = Мировые координаты
mated-world-tip = Координаты от положения в мире (кубическая проекция): без учёта позиции текстуры, узор продолжается через грани
mated-look-reset = Сбросить вид
mated-undo-look = Вид текстуры
mated-undo-color = Изменить цвет материала
mated-textures = Текстуры
mated-stock = Заготовки (раскрой)
mated-stock-not-in-model = Покрасьте что-нибудь этим материалом, чтобы указать, в каком виде его покупают.
mated-stock-kind = Покупается как
mated-stock-sheet = Лист
mated-stock-board = Доска
mated-stock-linear = Погонаж
mated-stock-remove = Удалить эту заготовку
mated-stock-thickness = Толщина
mated-stock-size = Размер
mated-stock-grain = Волокна вдоль длины
mated-stock-width = Ширина
mated-stock-lengths = Длины в продаже
mated-stock-profile = Профиль
mated-stock-profile-hint = напр. Круг Ø 20
mated-stock-section = Сечение
mated-stock-section-tip = Наибольшее сечение детали, которое из него выходит (в любую сторону); 0 — любое.
mated-stock-price-sheet = Цена за лист
mated-stock-price = Цена
mated-stock-per-metre = за метр
mated-stock-per-piece = за штуку
mated-stock-prices-tip = По цене на каждую длину доски в том же порядке, что и длины, или одна цена для всех длин.
mated-stock-save-preset = Сохранить как шаблон заготовки (Настройки → Раскрой)
mated-stock-saved = Сохранено в шаблоны заготовок: { $name }
mated-stock-my-presets = Мои шаблоны
mated-stock-edit-presets = Изменить шаблоны…
mated-stock-add = + Добавить заготовку
mated-stock-note = Детали из этого материала кроятся из погонажа, в сечение которого они помещаются, иначе из заготовки ближайшей толщины: из досок самого узкого сечения, в которое они помещаются, из листов того формата, что выходит дешевле (Окно → Раскрой).
mated-stock-preset-ply-18 = Фанера 2440 × 1220 × 18
mated-stock-preset-ply-12 = Фанера 2440 × 1220 × 12
mated-stock-preset-birch-18 = Берёзовая фанера 1525 × 1525 × 18
mated-stock-preset-mdf-16 = МДФ 2800 × 2070 × 16
mated-stock-preset-chipboard-16 = ЛДСП 2750 × 1830 × 16
mated-stock-preset-hdf-3 = ХДФ 2440 × 1220 × 3
mated-stock-preset-pine-95x45 = Сосна 95 × 45, 2,4–6 м
mated-stock-preset-board-145x20 = Доска 145 × 20, 2–3 м
mated-stock-preset-rod-20 = Круг Ø 20, 1–2 м
mated-stock-preset-skirting-70 = Плинтус 70 × 15, 2,5 м
mated-undo-stock = Заготовки материала
mated-slot-base = Основная текстура
mated-slot-empty = Пусто
mated-view-tip = Открыть крупно
mated-downscale = Уменьшить
mated-downscale-tip = Большое изображение: рендер использует не больше 1024 px. Пересохранить в { $side } px, чтобы документ был меньше
mated-load-tip = Загрузить изображение…
mated-replace-tip = Заменить другим изображением… (размер плитки сохранится)
mated-clear-tip = Убрать эту текстуру
mated-export-tip = Сохранить изображение в файл…
mated-edit-tip = Редактировать в графическом редакторе по умолчанию для файла: сохранение там обновит материал
mated-map-set = Загрузить набор карт из папки…
mated-map-set-tip = Заполнить слоты из папки по суффиксам имён файлов: _color / _albedo / _diff, _normal (OpenGL предпочтительнее), _rough, _metal, _ao, _emission
mated-watching = { $count ->
        [one] Отслеживается { $count } текстура, открытая в редакторе
        [few] Отслеживаются { $count } текстуры, открытые в редакторе
        [many] Отслеживается { $count } текстур, открытых в редакторе
       *[other] Отслеживается { $count } текстуры, открытой в редакторе
    }
mated-view-title = { $material }: { $slot }
mated-view-tiled = Повторить 3 × 3
mated-undo-texture = Загрузить текстуру
mated-undo-map-set = Загрузить набор карт
mated-undo-clear = Убрать текстуру
mated-undo-downscale = Уменьшить текстуру
mated-undo-reload = Перезагрузить текстуру
mated-no-maps = В { $path } нет изображений с суффиксами карт
mated-maps-loaded = { $count ->
        [one] Загружена { $count } карта
        [few] Загружены { $count } карты
        [many] Загружено { $count } карт
       *[other] Загружено { $count } карты
    }
mated-exported = Сохранено: { $path }
mated-editing = Редактируется { $file }: сохраните его, чтобы обновить материал
mated-cant-open-editor = Не удалось открыть графический редактор: { $error }
mated-reloaded = { $slot }: перезагружено из графического редактора

## ======== Онлайн-библиотека 3D ========

model-lib-licence-any = Любая лицензия
model-lib-licence-commercial = Коммерческое использование разрешено
model-lib-licence-cc0 = Только CC0
model-lib-licence-known = Известная лицензия

## Окно и фильтры

model-lib-title = Онлайн-библиотека 3D
model-lib-source = Источник
model-lib-source-count = { $label } ({ $n })
model-lib-plugins-hint =
    Плагины с @model_source
    перечислены здесь
model-lib-search-enter = Поиск, затем Enter: стул, череп…
model-lib-search = Поиск: стул, камень, растение…
model-lib-all-categories = Все категории
# $description — описание источника.
model-lib-downloads-go = { $description } Загрузки попадают в библиотеку компонентов.
model-lib-from-url = По ссылке
model-lib-import = Импорт
model-lib-import-hover = Скачать файл в библиотеку компонентов и разместить его
model-lib-click-hint = Щёлкните модель, чтобы посмотреть; двойной щелчок скачивает её и размещает.
model-lib-type-search = Введите запрос и нажмите Enter.
model-lib-loading = Загрузка моделей { $source }…
model-lib-load-failed = Не удаётся загрузить модели: { $error }
model-lib-retry = Повторить
# $n — число показанных моделей.
model-lib-count = { $n ->
        [one] { $n } модель
        [few] { $n } модели
        [many] { $n } моделей
       *[other] { $n } моделей
    }
model-lib-show = Показать
model-lib-format-any = Любой формат
model-lib-size-any = Любой размер
model-lib-size-10k = ≤ 10 тыс. треугольников
model-lib-size-50k = ≤ 50 тыс. треугольников
model-lib-size-100k = ≤ 100 тыс. треугольников
model-lib-clear = Очистить
model-lib-reload = Перезагрузить
model-lib-load = Загрузить
model-lib-choose-with = выберите через …
model-lib-choose-it = Выбрать
model-lib-plugin-apply-hover = Использовать эти значения (затем искать)
model-lib-plugin-load-hover = Запросить у плагина модели с этими значениями

## Сведения о модели

# $n — число треугольников.
model-lib-triangles = { $n ->
        [one] { $n } треугольник
        [few] { $n } треугольника
        [many] { $n } треугольников
       *[other] { $n } треугольников
    }
# $value — размер в мегабайтах.
model-lib-megabytes = { $value } МБ
model-lib-heavy = ⚠ Тяжёлая модель: может замедлить вид.
model-lib-downloading = Загрузка…
# $n — число моделей набора в библиотеке.
model-lib-in-components = ✔ Моделей в «Компонентах»: { $n }
model-lib-show-in-components = Показать в «Компонентах»
model-lib-download-browser = Скачать в браузере
model-lib-download-browser-hover = Открыть загрузку набора в браузере; архив импортируется из «Загрузок» вместе с авторством
model-lib-download-again = Скачать снова
model-lib-download-pack = Скачать набор
model-lib-download-pack-hover = Скачать набор: каждая модель станет компонентом библиотеки
model-lib-place-hover = Уже в библиотеке компонентов: разместить в модели
model-lib-download-place = Скачать и разместить
model-lib-download-place-hover = Сохранить в библиотеку компонентов и разместить в модели
model-lib-download = Скачать
model-lib-download-hover = Сохранить в библиотеку компонентов
model-lib-web-page = Веб-страница

## Сообщения

# $name — модель.
model-lib-cancelled = Загрузка { $name } отменена
model-lib-fav-added = ☆ { $name } добавлено в избранное
model-lib-fav-removed = { $name } убрано из избранного
model-lib-paste-url = Вставьте адрес файла модели (https://…/model.glb)
model-lib-browser-download = Скачайте { $name } в браузере: Varstotas импортирует архив из «Загрузок»
# $n — число моделей, $folder — папка библиотеки, $pack — папка набора.
model-lib-downloaded-pack = Загружено моделей: { $n } → Компоненты → { $folder } → { $pack }
model-lib-downloaded = Загружено: { $name } (граней: { $faces }) → Компоненты → { $folder }
model-lib-not-in-library = { $path } нет в библиотеке компонентов
model-lib-download-failed = Не удалось загрузить: { $error }
model-lib-import-failed = Импорт { $path }: { $error }
model-lib-undo-import = Импорт { $name }
model-lib-imported = Импортировано: { $path }

## Источники (online_models)

model-lib-source-favourites = Избранное
model-lib-source-downloaded = Загруженные
model-lib-terms-polyhaven = Работает на Poly Haven (polyhaven.com). Модели под CC0: свободно для любого использования, в том числе коммерческого; указывать авторство не нужно.
model-lib-terms-smithsonian = Большинство объектов под CC0, но некоторые помечены «Usage conditions apply» (только некоммерческое использование). Перед коммерческим использованием проверьте страницу объекта; товарные знаки и прочие права на сами объекты не охвачены.
model-lib-terms-nasa = Материалы NASA, как правило, не защищены авторским правом, но эмблема и логотипы NASA на моделях защищены, и ничто не должно намекать, что NASA одобряет ваш продукт.
model-lib-terms-kenney = Наборы Kenney под CC0: свободно для любого использования, в том числе коммерческого; указывать авторство необязательно. Логотип Kenney сюда не входит.
model-lib-terms-quaternius = Наборы Quaternius под CC0: свободно для любого использования, в том числе коммерческого; указывать авторство необязательно.
model-lib-terms-saved = Каждая модель сохраняет лицензию, с которой пришла (в её сведениях; Файл → Экспорт → Авторство). Varstotas предупреждает, если лицензия размещённой или экспортируемой модели ограничивает её использование.
model-lib-desc-polyhaven = Бесплатные модели CC0 (glTF, текстуры 1K) в натуральную величину.
model-lib-desc-smithsonian = Smithsonian Open Access 3D: сканы музейных предметов. Введите запрос и нажмите Enter.
model-lib-desc-nasa = NASA 3D Resources: космические аппараты, роверы, планеты (GLB).
model-lib-desc-kenney = Игровые наборы Kenney (CC0): набор скачивается целиком, каждая модель становится компонентом в Компоненты → Kenney → <набор>.
model-lib-desc-quaternius = Низкополигональные наборы Quaternius (CC0). Они скачиваются в браузере (Google Drive): Varstotas открывает страницу и импортирует архив из «Загрузок».
model-lib-desc-favourites = Модели, отмеченные ☆ (щёлкните ☆ в сведениях о модели).
model-lib-desc-history = Всё, что скачано или импортировано из «Загрузок», сначала новые. Вводите текст для поиска.

## Ошибки каталогов и импорта

model-lib-err-not-catalogue = это не каталог
model-lib-err-not-search = это не ответ на поиск
model-lib-err-kenney-packs = на kenney.nl не найдено наборов (страница изменилась?)
model-lib-err-quaternius-packs = на quaternius.com не найдено наборов (страница изменилась?)
model-lib-err-plugin-array = плагин должен вернуть массив моделей
model-lib-err-no-download = на странице набора нет загрузки
model-lib-err-quaternius-browser = Наборы Quaternius скачиваются в браузере: используйте «Скачать в браузере»
model-lib-err-importer-depth = плагины-импортёры вызывают друг друга слишком глубоко
model-lib-err-empty-answer = пустой ответ
model-lib-err-plugin-nothing = плагин ничего не вернул
model-lib-err-spec-object = описание модели должно быть объектом
model-lib-err-spec-empty = в описании модели нет file, url, zip или meshes
model-lib-err-archive-no-model = в архиве нет файла модели (glb, gltf, fbx, dae, obj, 3mf, ply, stl, dxf)
# $k — номер меша в ответе.
model-lib-err-mesh-vertices = меш { $k }: нет вершин
model-lib-err-mesh-faces = меш { $k }: нет граней
model-lib-err-no-faces = в мешах нет граней
model-lib-err-model-empty = модель пуста
model-lib-err-no-size = у модели нет размера
model-lib-err-no-home = нет домашней папки
model-lib-err-archive-none = в архиве нет файлов моделей
model-lib-err-none-read = не удалось прочитать ни одной модели
## ======== Загрузки ========

## Сообщения
# $path — файл.
downloads-no-models = { $path }: в нём нет файлов моделей
downloads-no-address = { $path }: в нём нет веб-адреса
downloads-not-model = { $path }: не файл модели (glb, gltf, fbx, dae, obj, 3mf, ply, stl, dxf, varstotas, zip)
# $site — сайт, $title — модель.
downloads-waiting-site = Ожидание файла с { $site }: скачайте его в браузере
downloads-waiting-file = Ожидание файла «{ $title }» ({ $site }): скачайте его в браузере
downloads-no-library = Нет папки библиотеки компонентов
# $name — файл, $folder — папка библиотеки, $n — число моделей.
downloads-imported = Импортировано { $name } в Компоненты → { $folder }
downloads-imported-many = Импортировано моделей из { $name }: { $n } — в Компоненты → { $folder }
downloads-import-failed = Импорт { $name }: { $error }
downloads-err-no-file-name = нет имени файла
downloads-err-no-model-chosen = модель не выбрана
# $label — лицензия, $names — первые модели под ней, $more — " +N" или пусто, $note — её требование.
downloads-licence-part = { $label } ({ $names }{ $more }) — { $note }
# $list — тексты частей про лицензии.
downloads-licences = Лицензии: { $list }. Список авторов: Файл → Экспорт → Авторство.

## Поиск на, Другие сайты

downloads-search-on = Искать на:
# $site — сайт, $query — запрос.
downloads-search-hover = Открыть поиск { $site } по запросу «{ $query }» в браузере
downloads-edit = Правка…
downloads-edit-hover = Изменить список сайтов ({ DATA_DIR() }/sources.toml); список будет прочитан заново
downloads-other = Другие сайты
# $waiting — число ожидающих загрузок.
downloads-other-waiting = Другие сайты — ожидается загрузок: { $waiting }
downloads-other-text = Скачивайте с любого сайта в браузере: Varstotas сам сайты не посещает. Он берёт файл из папки «Загрузки» и сохраняет страницу как источник.
downloads-watch = Следить за загрузками
downloads-watch-hover = Предлагать на импорт каждый новый файл модели или архив из папки «Загрузки» (всегда включено, пока ждёт страница)
# $dir — отслеживаемая папка.
downloads-watching = слежу за { $dir }
downloads-off = выкл.
downloads-import-file = Импорт файла…
downloads-import-file-hover = Импортировать скачанный файл модели или архив
downloads-models-filter = Модели
# $title — модель, $site — сайт, $licence — её лицензия.
downloads-waiting-card = Ожидание файла с этой страницы: { $title } ({ $site }, { $licence })
downloads-page = Страница
# $file — адрес файла.
downloads-browser-file = Скачайте в браузере: { $file }
downloads-stop = Не ждать

## Окно «Импорт загрузки»

downloads-import-title = Импорт загрузки
downloads-importing = Импорт…
# $name — файл, $mb — его размер в мегабайтах.
downloads-file-size = { $name }  ({ $mb } МБ)
downloads-credit-from-page = ✔ Источник взят со страницы, которую вы отправили
downloads-duplicate = ⚠ Этот же файл уже импортировался
downloads-field-title = Название
downloads-field-author = Автор
downloads-field-site = Сайт
downloads-licence = Лицензия
downloads-licence-hint = как указано на странице
downloads-place = Поместить в модель
# Выбрано $on из $n моделей.
downloads-models-count = Выбрано { $on } из { $n ->
        [one] модели
       *[other] моделей
    } (каждая станет компонентом)
downloads-all = Все
downloads-none = Нет
# $folder — папка библиотеки сайта.
downloads-destination = Компоненты → { $folder }
downloads-ignore = Игнорировать
# $n — число других предложений.
downloads-more-waiting = Ещё ожидает: { $n }

## ======== Рендер ========

render-dof-view = Как в виде
render-dof-off = Выкл.
render-dof-on = Вкл.

## Пресеты качества и размеры изображения (идентификаторы остаются английскими)

render-preset-draft = Черновик
render-preset-preview = Просмотр
render-preset-final = Финал
render-preset-best = Максимум
render-size-hd = HD 1280 × 720
render-size-full-hd = Full HD 1920 × 1080
render-size-qhd = QHD 2560 × 1440
render-size-4k = 4K 3840 × 2160
render-size-square = Квадрат 1080
render-size-portrait = Портрет 1080 × 1350

## Месяцы (дата солнца)

render-month-jan = Янв
render-month-feb = Фев
render-month-mar = Мар
render-month-apr = Апр
render-month-may = Май
render-month-jun = Июн
render-month-jul = Июл
render-month-aug = Авг
render-month-sep = Сен
render-month-oct = Окт
render-month-nov = Ноя
render-month-dec = Дек

## Ошибки экспорта и шейдеров

render-export-empty = размер изображения должен быть не менее 1 x 1 пикселя
# $w и $h — запрошенный размер, $max — предел на сторону.
render-export-too-large = размер изображения { $w } x { $h } слишком велик (не более { $max } пикселей на сторону)
render-export-gpu = Ошибка GPU при экспорте изображения: { $error }
render-shader-clash = не компилируется вместе с другими шейдерами: { $error }

## Окно рендера

render-title = Рендер
render-current-view = Рендер текущего вида
render-stop = Стоп
render-continue = Продолжить
render-more-samples = Больше семплов
render-more-samples-tip = Удвоить число семплов
render-save-png = Сохранить PNG…
render-save-exr = Сохранить EXR…
render-save-exr-tip = Линейный свет, 32-битные числа с плавающей точкой
render-all-scenes = Рендер всех сцен…
# $error — причина.
render-failed = Рендер не удался: { $error }
render-error = Рендер: { $error }
# $n — число сцен, $folder — куда они сохранены.
render-scenes-done = Отрендерено сцен: { $n } — в { $folder }
# Сохранено $written сцен, сейчас рендерится сцена $next.
render-scenes-progress = Рендер сцен: сохранено { $written }, в работе сцена { $next }
# $path — сохранённый файл.
render-saved = Сохранено: { $path }
# $secs — число с одним знаком после запятой.
render-status-running = { $samples } / { $max } семплов · { $secs } с
# $denoised — "yes" или "no".
render-status-done = Готово: { $samples } семплов за { $secs } с{ $denoised ->
        [yes] , с шумоподавлением
       *[no] {""}
    }
render-status-stopped = Остановлено на { $samples } семплах · { $secs } с
render-undo-lights = Правка источников света
render-undo-settings = Настройки рендера

## Источники света

render-lights = Источники света ({ $n })
render-add-point = + Точечный
render-add-spot = + Прожектор
render-add-area = + Площадной
render-on = Вкл.
render-light-point = точечный
render-light-spot = прожектор
render-light-area = площадной
render-light-name = Имя
render-light-type = Тип
render-light-position = Положение
render-light-to-centre = В центр вида
render-light-to-centre-tip = Поднять на 2 м над целью камеры
render-light-to-camera = К камере
render-light-aim-at-target = Нацелить на цель
render-light-aim-at-target-tip = Светить на цель вида
render-light-aim = Направление
render-colour = Цвет
render-light-power = Мощность
render-light-radius = Радиус
render-light-radius-tip = Размер лампы: мягче тени
render-light-cone = Конус
render-light-cone-angle = Угол конуса
render-light-soft = Мягкость
render-light-soft-edge = Мягкий край
render-size = Размер

## Настройки

render-sec-image = Изображение
render-sec-sampling = Семплирование
render-sec-camera = Камера
render-sec-light = Свет
render-sec-colour = Цвет
render-quality = Качество
# $samples и $bounces — числа.
render-quality-tip = Семплов: { $samples }, отражений: { $bounces }
render-match-view = Как вид
render-match-view-tip = Пропорции вида при этой высоте
render-size-custom = Свой
render-transparent = Прозрачный фон
render-transparent-tip = Небо и земля за моделью в PNG прозрачны (но по-прежнему освещают её)
render-samples = Семплы
render-time-limit = Лимит времени
render-time-limit-tip = 0 = пока не наберутся семплы
render-bounces = Отражения
render-clamp = Ограничение
render-clamp-tip = Самый яркий непрямой свет семпла: ниже — убирает светлячки, выше — сохраняет яркий отражённый свет; 0 = выкл.
render-denoiser = Шумоподавление
render-denoiser-off = Выкл.
render-denoiser-builtin = Встроенное
render-denoiser-builtin-tip = Фильтр с учётом границ
render-denoiser-ai = ИИ (Open Image Denoise)
render-denoiser-install = Установите Open Image Denoise: brew install open-image-denoise
render-dof = Глубина резкости
render-dof-tip = Как в виде: глубина резкости из эффектов, когда она включена
# $mm — эквивалентное фокусное расстояние.
render-lens = Объектив { $mm } мм
render-lens-tip = Поле зрения вида для камеры 35 мм
render-aperture = Диафрагма
render-aperture-tip = Меньше число f — меньше глубина резкости
render-focus = Фокус
render-focus-target = Цель
render-focus-target-tip = Фокус на цели вида (центр вращения)
render-sun = Солнце
render-sun-tip = Яркость солнца (направление — в «Тенях»)
render-sky = Небо
render-sky-tip = Свет окружения
render-rotation = Поворот
render-rotation-tip = Поворачивает небо / HDRI
render-exposure = Экспозиция
render-env-exposure-tip = Экспозиция окружения (вид и рендер)
render-exposure-tip = Добавляется к экспозиции окружения (только рендер)
render-tone-curve = Тоновая кривая
# $name — тоновая кривая окружения.
render-tone-environment = Окружение: { $name }
render-white-balance = Баланс белого
render-white-balance-tip = 6500 K = нейтральный; выше — теплее
render-tint = Оттенок
render-tint-tip = Зелёный (−) .. пурпурный (+)
render-saturation = Насыщенность
render-contrast = Контраст
render-vignette = Виньетирование
render-reset = Сбросить
render-reset-tip = Вернуть настройки цвета по умолчанию
render-shader-define-surface = define fn surface(s: Surface, p: Params) -> Surface
# $line — строка в коде шейдера пользователя, $msg — сообщение компилятора.
render-shader-line = строка { $line }: { $msg }

## ======== Стиль «Реалистичный» ========

realistic-undo-material = Правка материала
# $path — файл, $error — причина.
realistic-cant-use = Нельзя использовать { $path }: { $error }
realistic-cant-read = Не удалось прочитать { $path }: { $error }
realistic-used-by = Используется стилем «Реалистичный»
realistic-load = Загрузить…
realistic-clear = Очистить
realistic-reset = Сбросить
realistic-preset = Пресет

## Окружение

realistic-background = Фон
realistic-sky-sky = Небо
realistic-sky-color = Цвет
realistic-sky-hdri = HDRI
realistic-sky-physical = Физическое небо
realistic-sky-physical-tip = Дневной свет для положения солнца в карточке «Тени»
realistic-haze = Дымка
realistic-haze-tip = Мутность: 2 = чистое голубое небо, 10 = дымка; время и дата солнца — в карточке «Тени»
realistic-hdri-none = нет (небо)
realistic-hdri-tip = Равнопромежуточная панорама (.hdr)
realistic-rotation = Поворот
realistic-intensity = Интенсивность
realistic-sun = Солнце
realistic-sun-tip = Яркость солнца; его направление берётся из карточки «Тени», когда тени включены
realistic-exposure = Экспозиция
realistic-tone-mapping = Тональная компрессия
realistic-tonemap-aces = ACES (киношная)
realistic-tonemap-agx = AgX (мягкая)
realistic-tonemap-none = Нет
realistic-show-env = Показывать окружение за моделью
realistic-effects = Эффекты

## Физические свойства

realistic-physical = Физика (Реалистичный)
realistic-preset-matte = Матовый
realistic-preset-satin = Сатин
realistic-preset-glossy = Глянцевый
realistic-preset-metal = Металл
realistic-preset-chrome = Хром
realistic-preset-glass = Стекло
realistic-preset-light = Свет
realistic-metallic = Металличность
realistic-metallic-tip = 0 = краска, пластик, дерево; 1 = голый металл
realistic-roughness = Шероховатость
realistic-roughness-tip = 0 = как зеркало, 1 = матовый
realistic-glass = Стекло
realistic-glass-tip = Сколько света проходит насквозь
realistic-ior = IOR
realistic-ior-tip = Показатель преломления (стекло 1,5, вода 1,33)
realistic-emission = Свечение
realistic-emission-tip = Свет, который излучает поверхность: цвет и яркость
realistic-coat-sheen = Лак, Блеск ткани, Края
realistic-clearcoat = Прозрачный лак
realistic-clearcoat-tip = Глянцевый слой лака: лакированное дерево, автомобильная краска, глазурованная плитка
realistic-coat-rough = Шерох. лака
realistic-coat-rough-tip = Шероховатость лака (0 = глянец)
realistic-sheen = Блеск ткани
realistic-sheen-tip = Мягкое свечение под скользящим углом: бархат, ткань, обивка
realistic-sheen-tint = Оттенок блеска
realistic-sheen-tint-tip = 0 = белый блеск, 1 = цвет материала
realistic-anisotropy = Анизотропия
realistic-anisotropy-tip = Растянутые блики вдоль текстуры (+) или поперёк (−): шлифованный металл
realistic-anti-tiling = Антитайлинг
realistic-anti-tiling-tip = Скрывает заметное повторение текстур
realistic-bevel = Фаска, мм
realistic-bevel-tip = Скругляет рёбра граней при рендере (только трассировка путей)
realistic-shader = Шейдер
realistic-shader-tip = Поверхностная программа на WGSL (docs/SHADERS.md): процедурные узоры, волны, свечение
realistic-shader-none = Нет
realistic-shader-custom = Свой
realistic-shader-edit = Правка…
realistic-shader-edit-tip = Написать или изменить шейдер
realistic-map-normal = Нормали
realistic-map-roughness = Шероховатость
realistic-map-metallic = Металличность
realistic-map-occlusion = Затенение
realistic-map-emission = Свечение
realistic-normal-strength = Сила нормалей
realistic-remove-physical = Убрать физические свойства

## Редактор шейдеров

# $material — имя материала.
realistic-shader-window = Шейдер — { $material }
realistic-start-from = Взять за основу…
realistic-start-from-tip = Заменить код этим шейдером
realistic-checker-new = Шахматка (новый)
realistic-reference = Справка
realistic-reference-tip = Поля поверхности, вспомогательные функции и встроенные
realistic-compiles = Компилируется; материал его использует
realistic-ref-surface = Поверхность (s, o)
realistic-ref-parameters = Параметры
realistic-ref-functions = Функции

## Эффекты

realistic-fx-off = Выкл.
realistic-fx-fast = Быстро
realistic-fx-balanced = Баланс
realistic-fx-quality = Качество
realistic-fx-soft-shadows = Мягкие тени
realistic-fx-soft-shadows-tip = Тени размываются по мере удаления от предмета, который их отбрасывает
realistic-fx-sun-size = Размер солнца
realistic-fx-ao = Фоновое затенение
realistic-fx-ao-tip = Темнее складки и контактные тени
realistic-fx-radius = Радиус, мм
realistic-fx-strength = Сила
realistic-fx-reflections = Отражения
realistic-fx-reflections-tip = Модель отражается в глянцевых поверхностях (в экранном пространстве)
realistic-fx-bloom = Свечение
realistic-fx-bloom-tip = Яркий свет светится
realistic-fx-threshold = Порог
realistic-fx-dof = Глубина резкости
realistic-fx-dof-tip = Размытие перед фокусным расстоянием и за ним
realistic-fx-blur = Размытие
realistic-fx-focus = Фокус, мм
realistic-fx-focus-tip = 0 = точка в центре вида
realistic-fx-fog = Туман
realistic-fx-fog-tip = Дымка, густеющая с расстоянием
realistic-fx-fog-distance = Расстояние, мм
realistic-fx-fog-height = Высота, мм
realistic-fx-fog-height-tip = 0 = равномерный туман; иначе редеет на этой высоте
realistic-vignette = Виньетирование
realistic-style = Стиль
realistic-stylize-photo = Фото
realistic-stylize-pencil = Карандаш
realistic-stylize-watercolor = Акварель
realistic-stylize-technical = Технический
realistic-fx-clay = Глина
realistic-fx-clay-tip = Все материалы как простая глина (объёмная макетная модель)
realistic-fx-high-quality = Высокое качество
realistic-fx-high-quality-tip = Больше семплов для мягких теней, затенения и отражений

## ======== Анимация ========

anim-title = Экспорт анимации
anim-no-scenes = в анимации нет сцен
anim-no-scenes-flash = В анимации нет сцен
# $error — причина.
anim-error = Анимация: { $error }
# $what — путь к файлу или "N кадров в <папка>".
anim-saved = Сохранено: { $what }
anim-render-dropped = рендер был отброшен
# $n — число кадров, $dir — папка с кадрами PNG.
anim-frames-in = { $n ->
        [one] { $n } кадр
        [few] { $n } кадра
        [many] { $n } кадров
       *[other] { $n } кадра
    } в { $dir }
anim-cancelled = отменено
anim-animate = Анимировать
anim-scenes = Сцены
anim-add-scenes = Сначала добавьте сцены (с «Включить в анимацию»)
anim-orbit = Облёт вокруг вида
anim-length = Длительность
anim-turns-suffix = {" "}об.
anim-holds-hint = Задержки и переходы: карточка «Сцены» → Анимация
anim-size = Размер
anim-fps = Частота кадров
anim-frames = Кадры
anim-as-view = Как вид
anim-path-traced = Трассировка путей
anim-output = Результат
anim-gif = Анимированный GIF
anim-png-sequence = Последовательность PNG
# Кадр $done из $total.
anim-frame-progress = Кадр { $done } / { $total }
anim-encoding = Запись GIF…
anim-export = Экспорт…

## ======== Развёртка ========

unfold-paper-fit = По размеру деталей
unfold-paper-a4 = A4
unfold-paper-a3 = A3
unfold-paper-a2 = A2

## Окно развёртки и экспорт

cutlist-title = Раскрой
cutlist-kerf = Пропил
cutlist-trim = Обрезка кромок
cutlist-min-offcut = Остатки от
cutlist-no-parts = Нет деталей: раскрой строится из групп и компонентов модели.
cutlist-no-stock-at-all = Ни у одной детали нет заготовки: укажите, в каком виде покупается её материал, в редакторе материалов (Заготовки).
cutlist-no-material = без материала
cutlist-stock-sheet = { $material } { $length } × { $width } × { $thickness }
cutlist-stock-board = { $material } { $width } × { $thickness }
cutlist-stock-linear = { $material } { $profile }
cutlist-stock-offcut = { $stock } (обрезок)
cutlist-buy-sheets = { $n ->
    [one] { $n } лист
    [few] { $n } листа
   *[other] { $n } листов
}
cutlist-buy-boards = { $n } × { $length }
cutlist-offcuts-used = { $n ->
    [one] { $n } ваш обрезок
    [few] { $n } ваших обрезка
   *[other] { $n } ваших обрезков
}
cutlist-waste = , отходы { $percent }%
cutlist-cost = , стоимость { $cost }
cutlist-total = Итого: { $cost }
cutlist-issue-no-stock = у материала «{ $material }» нет заготовки (Редактор материалов → Заготовки)
cutlist-issue-no-material = без материала, значит без заготовки
cutlist-issue-too-big = больше заготовки
cutlist-issue-thickness = толщина { $part_thickness }, кроится из { $stock }
cutlist-click-select = Нажмите, чтобы выделить в модели
cutlist-tab-diagrams = Карты раскроя
cutlist-tab-parts = Детали ({ $n })
cutlist-col-part = Деталь
cutlist-col-count = Кол-во
cutlist-col-size = Длина × ширина × толщина
cutlist-col-material = Материал
cutlist-col-grain = Волокна вдоль
cutlist-grain-length = длины
cutlist-grain-width = ширины
cutlist-grain-any = любое (можно повернуть)
cutlist-parts-note = Детали — это группы и компоненты, внутри которых ничего нет; одинаковые (имя, размер и материал) идут одной строкой. Настройки действуют на все детали с этим именем.
cutlist-page-sheet = { $stock }: лист { $n } из { $total }
cutlist-page-boards = { $stock }: доски { $from }–{ $to } из { $total }
cutlist-page-lengths = { $stock }: хлысты { $from }–{ $to } из { $total }
cutlist-sheet-no-page = Этой страницы раскроя больше нет
cutlist-board-label = { $n }: { $length }
cutlist-footer = { $doc } · раскрой · страница { $n } из { $total }
cutlist-page-of = Страница { $n } из { $total }
cutlist-use-offcuts = Брать мои обрезки ({ $n })
cutlist-use-offcuts-tip = Детали сначала кроятся из ваших обрезков (Настройки → Раскрой), начиная с самых маленьких, и только потом из покупных заготовок. Обрезки, оставшиеся от этого документа, в нём не используются.
cutlist-keep = Сохранить обрезки
cutlist-keep-tip = Когда раскрой выполнен: использованные обрезки убираются из ваших, а остатки этого раскроя (не меньше минимального размера) добавляются к ним для следующих проектов.
cutlist-keep-done = Сохранено. План остаётся таким, как раскроили, пока модель не изменится.
cutlist-kept = Обрезки сохранены: новых { $kept }, использовано { $used }.
cutlist-export = Экспорт:
cutlist-exported = Раскрой сохранён: { $path }
cutlist-export-failed = Не удалось сохранить раскрой: { $error }
cutlist-undo-part = Настройки детали в раскрое
cutlist-undo-settings = Настройки раскроя
cutlist-band = Кромка
cutlist-band-tip = Толщина кромочной ленты на облицованных кромках (их отмечают на вкладке «Детали»).
cutlist-band-price = цена за м
cutlist-band-subtract = Резать меньше на кромку
cutlist-band-subtract-tip = Детали режутся меньше на толщину кромки по каждой облицованной кромке, чтобы после облицовки выйти в размер.
cutlist-banding = Кромка { $thickness } для деталей { $width }: { $metres } м
cutlist-col-bands = Кромка
cutlist-bands-tip = Щёлкните кромку, чтобы облицевать её: длинные кромки снизу и сверху, короткие по бокам.
cutlist-cut-size = Рез { $length } × { $width } (за вычетом кромки)
cutlist-numbers = Номера на модели
cutlist-numbers-tip = Показывать номер каждой детали на модели, пока открыт раскрой.
cutlist-labels = Этикетки…
cutlist-labels-tip = PDF с этикетками, по одной на каждую деталь: номер, имя, размер, материал, облицованные кромки и направление волокон. A4, 3 × 8 этикеток 70 × 37 мм.
cutlist-label-copy = { $name } ({ $n }/{ $total })
cutlist-label-cut = рез { $length } × { $width }
cutlist-col-use = Учёт
cutlist-use-cut = резать
cutlist-use-hardware = фурнитура
cutlist-use-out = не учитывать
cutlist-tab-hardware = Фурнитура ({ $n })
cutlist-col-item = Позиция
cutlist-col-price = Цена за шт.
cutlist-col-cost = Сумма
cutlist-hardware-remove = Удалить позицию
cutlist-hardware-add = + Добавить позицию
cutlist-hardware-new = Винты
cutlist-hardware-note = Детали с отметкой «фурнитура» на вкладке «Детали» (или в контекстном меню модели) покупаются, а не режутся. То, чего нет в модели (винты, шканты, клей), добавьте вручную.
cutlist-hardware-line = Фурнитура: { $n } шт.
menu-cut-hardware = Фурнитура в раскрое
menu-cut-hardware-tip = Детали с именем «{ $name }» покупаются, а не режутся: они попадают в список фурнитуры.

unfold-title = Развёртка
unfold-selection = Выделение
unfold-everything = Всё в контексте
unfold-nothing = Развёртка: нечего развернуть (выделите грани или группы)
# $name — имя документа, $scale — вроде "1:10".
unfold-footer = { $name } - развёртка { $scale }
# $n — номер листа из $total.
unfold-footer-sheet = { $name } - развёртка { $scale } - лист { $n } из { $total }
# $oversize — число деталей, больших листа (0 = нет).
unfold-done = Развёрнуто: граней — { $faces }, деталей — { $pieces }, листов — { $sheets }{ $oversize ->
        [0] {""}
       *[other] {" "}(больше листа: { $oversize })
    }: { $path }
unfold-export-failed = Не удалось экспортировать: { $error }
# $what — "Выделение" или "Всё в контексте".
unfold-no-faces = { $what }: нет граней. Выделите грани, группы или компоненты.
unfold-summary = { $what }: граней — { $faces }, деталей — { $pieces }, листов — { $sheets }
unfold-oversize = Деталей больше листа: { $n } — выберите бумагу побольше или другой масштаб.
unfold-scale = Масштаб 1:
unfold-paper = Бумага
unfold-landscape = Альбомная
unfold-glue-tabs = Клапаны для клея
unfold-edge-numbers = Номера рёбер
unfold-soft-folds = Сгибы на сглаженных рёбрах
unfold-sheet-of = Лист { $n } из { $total }
unfold-legend-cut = ━ разрез
unfold-legend-valley = ╍ долина
unfold-legend-mountain = ━·━ гора
unfold-export = Экспорт:

## ======== Редактор шейдеров ========

## Встроенные пресеты
shader-preset-wood = Дерево
shader-preset-water = Вода
shader-preset-reeded-glass = Рифлёное стекло
shader-preset-glow = Свечение

## Подсказки по автодополнению и вызовам (описания вспомогательных функций, встроенных функций WGSL и полей Surface)
shader-fn-surface = шейдер: верните изменённую поверхность
shader-fn-hash3 = псевдослучайное значение 0..1 для точки
shader-fn-noise = гладкий шум значений 0..1
shader-fn-fbm = многослойный шум 0..1 (облака, мрамор, искажения)
shader-fn-srgb = цвет, выбранный в sRGB (0..1) → линейный
shader-fn-hsv = тон / насыщенность / яркость (0..1) → линейный цвет
shader-fn-tangent_of = единичный вектор поперёк n
shader-fn-bump = n, наклонённая по уклону поля высот
shader-fn-abs = абсолютное значение
shader-fn-min = меньшее
shader-fn-max = большее
shader-fn-clamp = x, ограниченное диапазоном lo..hi
shader-fn-saturate = x, ограниченное диапазоном 0..1
shader-fn-mix = a + (b − a)·t
shader-fn-step = 0 ниже edge, 1 начиная с него
shader-fn-smoothstep = плавный переход 0→1 между lo и hi
shader-fn-fract = x − floor(x)
shader-fn-floor = округление вниз
shader-fn-ceil = округление вверх
shader-fn-round = ближайшее целое
shader-fn-sign = −1, 0 или 1
shader-fn-sqrt = квадратный корень
shader-fn-pow = x в степени y
shader-fn-exp = e в степени x
shader-fn-log = натуральный логарифм
shader-fn-sin = синус (радианы)
shader-fn-cos = косинус (радианы)
shader-fn-tan = тангенс (радианы)
shader-fn-atan2 = угол (x, y), радианы
shader-fn-length = длина вектора
shader-fn-distance = length(a − b)
shader-fn-normalize = единичный вектор вдоль v
shader-fn-dot = скалярное произведение
shader-fn-cross = векторное произведение двух vec3
shader-fn-reflect = i, отражённый относительно нормали n
shader-fn-select = t при cond, иначе f
shader-fn-vec2 = 2-вектор
shader-fn-vec3 = 3-вектор
shader-fn-vec4 = 4-вектор
shader-surface-base = линейный основной цвет
shader-surface-alpha = непрозрачность
shader-surface-metallic = 0 = диэлектрик, 1 = металл
shader-surface-roughness = 0 = зеркало, 1 = матовый
shader-surface-transmission = стекло: 1 = свет проходит насквозь
shader-surface-emission = излучаемый свет, линейный, с учётом силы
shader-surface-normal = мировая нормаль, единичная, смотрит на зрителя
shader-surface-pos = положение в модели, мм (z = вверх)
shader-surface-uv = плоские координаты текстуры, мм
shader-surface-view = единичное направление к глазу
shader-surface-time = секунды
shader-param-tag = ползунок редактора материала, читается как p.name (не более 8)

## ======== Диалоги ========

## Восстановление автосохранения
dialog-untitled = Без имени
dialog-age-recently = недавно
dialog-age-under-minute = менее минуты назад
# $n — число
dialog-age-minutes = { $n ->
        [one] { $n } минуту назад
        [few] { $n } минуты назад
        [many] { $n } минут назад
       *[other] { $n } минуты назад
    }
dialog-age-hours = { $n ->
        [one] { $n } час назад
        [few] { $n } часа назад
        [many] { $n } часов назад
       *[other] { $n } часа назад
    }
dialog-age-days = { $n ->
        [one] { $n } день назад
        [few] { $n } дня назад
        [many] { $n } дней назад
       *[other] { $n } дня назад
    }

## Сохранение изменений
dialog-save-title = Сохранить изменения?
# $name — имя документа
dialog-save-question = Сохранить изменения, внесённые в «{ $name }»?
dialog-save-warning = Если не сохранить, изменения будут потеряны.
dialog-dont-save = Не сохранять
dialog-save = Сохранить

## Восстановление автосохранённой модели
dialog-recover-title = Восстановить автосохранённую модель?
# $name — имя файла; $age — текст dialog-age-* («5 минут назад»)
dialog-recover-copy = Varstotas нашёл автосохранённую копию «{ $name }» от { $age }, которая новее сохранённого файла.
dialog-recover-untitled = Varstotas нашёл автосохранённую безымянную модель от { $age }.
dialog-recover-note = В ней могут быть несохранённые изменения, например после сбоя.
dialog-discard = Удалить
dialog-discard-hint = Удалить автосохранённую копию
dialog-recover = Восстановить
dialog-not-now = Не сейчас
dialog-not-now-hint = Оставить автосохранённую копию на потом

## Изменение размера
dialog-resize-title = Изменить размер?
dialog-resize-model = Изменить размер модели?
dialog-resize-group = Изменить размер активной группы или компонента?
# $factor — коэффициент масштаба, например 1.2500
dialog-resize-scaled = Всё в ней масштабируется в { $factor } раз относительно начала координат.
dialog-resize-keys = Enter = Да · Esc = Нет
dialog-yes = Да
dialog-no = Нет

## Запрос значения
dialog-invalid-value = Неверное значение

## Общие подписи
dialog-width = Ширина
dialog-height = Высота

## Экспорт 2D-изображения
dialog-export-title = Экспорт 2D-изображения
dialog-export-aspect = Пропорции вида
dialog-export-viewport = Размер видового экрана
# $w и $h — число пикселей
dialog-export-viewport-hint = { $w } × { $h } px (экранные пиксели)
dialog-export-double = Удвоить размер изображения
dialog-export-transparent = Прозрачный фон
dialog-export-axes = Оси
dialog-export-guides = Направляющие
dialog-export-dims = Размеры
dialog-export-note = Выделение и предпросмотр инструментов никогда не экспортируются.
dialog-export = Экспорт…
dialog-filter-png = Изображение PNG

## Экспорт 2D-чертежа
dialog-drawing-title = Экспорт 2D-чертежа
dialog-drawing-intro = Текущий вид линиями: SVG или PDF.
dialog-drawing-scale = В масштабе 1:
dialog-drawing-perspective = Вид в перспективе: вписывается в страницу A4. Для масштаба: Камера → Параллельная проекция.
dialog-drawing-fitted = Вписывается в страницу A4.
dialog-drawing-profiles = Контуры (толстые линии)
dialog-drawing-hidden = Скрытые линии (пунктир)
dialog-drawing-dims = Размеры и текст
dialog-drawing-sections = Разрез сечением
dialog-drawing-pdf = PDF…
dialog-drawing-svg = SVG…

## Размещение объёмного текста
dialog-text3d-title = Разместить объёмный текст
dialog-text3d-font = Шрифт
dialog-text3d-filled = Заливка
dialog-text3d-extruded = Выдавленный
dialog-text3d-hint = Enter — новая строка, %CMD%+Enter — разместить текст.
dialog-text3d-place = Разместить

## Выбор шаблона
dialog-template-title = Выберите шаблон
dialog-template-intro = Новые модели начинаются с этих единиц. Значение по умолчанию можно изменить в Настройках.

## О программе
dialog-about-tagline = Лёгкий нативный 3D-редактор для рисования и построения.
# $version — версия приложения
dialog-about-version = Версия { $version }
app-start-failed-title = Не удалось запустить Varstotas
app-start-failed = Varstotas не смог открыть окно:

    { $error }

    Для 3D-вида нужен Vulkan, DirectX 12, Metal или OpenGL 3.3. Обновите драйвер видеокарты; в виртуальной машине включите 3D-ускорение. Чтобы попробовать другой графический API, задайте переменную окружения WGPU_BACKEND = gl, vulkan или dx12.
dialog-about-build = Сборка { $build } · { $commit }
dialog-about-dev-build = Сборка для разработки · { $commit }
dialog-about-licence = Свободное ПО под лицензией MIT.
dialog-about-licences = Лицензии…

## ======== Листы ========

sheets-title = Листы
sheets-custom = Свой
sheets-none-yet = Листов пока нет.
sheets-none-first = Нет листа: сначала добавьте (Окно → Листы)
sheets-nothing-to-export = Листы: нечего экспортировать (сначала добавьте лист)
# $i — номер листа, $n — число листов
sheets-exported-one = Экспортирован лист { $i } из { $n }: { $path }
sheets-exported-all = Экспортировано листов: { $n }: { $path }
sheets-export-failed = Не удалось экспортировать: { $error }
sheets-add-first = Добавьте лист, чтобы начать

## Шаги отмены
sheets-undo-add = Добавление листа
sheets-undo-edit = Правка листа
sheets-undo-duplicate = Дублирование листа
sheets-undo-delete = Удаление листа

## Список листов и свойства
sheets-add = + Лист
sheets-duplicate = Дублировать
sheets-name = Имя
sheets-paper = Бумага
sheets-landscape = Альбомная
sheets-title-block = Основная надпись
sheets-tb-title = Название
sheets-tb-author = Автор
sheets-tb-date = Дата

## Добавление элементов
sheets-add-viewport = + Видовой экран
sheets-add-viewport-hint = Текущий вид модели
sheets-add-text = + Текст
sheets-add-credits = + Источники
sheets-add-credits-hint = Источники скачанных моделей (название, автор, сайт, лицензия): текст с {"{credits}"}, заполняется при отрисовке
sheets-remove = Удалить
sheets-add-views = + Стандартные виды
sheets-add-views-hint = Спереди, сверху и слева в стандартном масштабе с габаритными размерами, и вид в изометрии
sheets-add-table = + Таблица деталей
sheets-add-table-hint = Группы и компоненты с размерами
sheets-add-cut = + Схема раскроя
sheets-add-cut-hint = Страница из Окно → Раскрой: один лист или ряд досок с номерами деталей
sheets-add-cut-none = Раскраивать пока нечего: задайте материалам заготовки (Редактор материала → Заготовки)

## Выбранный элемент
# $n — номер элемента
sheets-viewport-n = Видовой экран { $n }
sheets-text-n = Текст { $n }
sheets-table-n = Таблица деталей { $n }
sheets-cut-n = Схема раскроя { $n }
sheets-cut-page = Страница
sheets-shows = Показывает
sheets-captured = Сохранённый вид
sheets-update-view = Обновить по текущему виду
sheets-scale = Масштаб
sheets-scale-hint = Для масштаба нужна параллельная проекция
sheets-position = Положение
sheets-size = Размер
sheets-height = Высота
sheets-text-height = Высота текста
sheets-profiles = Контуры
sheets-hidden = Скрытые линии
sheets-dims = Размеры
sheets-sections = Сечения
sheets-frame = Рамка
sheets-caption = Подпись
sheets-overall = Габаритные размеры
sheets-hint = Щёлкните элемент в предпросмотре, чтобы править его; перетащите, чтобы переместить, потяните за угол, чтобы изменить размер.

## Нижняя панель
# $paper — название бумаги; остальное — количества
sheets-summary = { $paper } — видовых экранов: { $viewports }, в масштабе: { $scaled }, текстов: { $texts }
sheets-svg-hint = Этот лист
sheets-pdf-hint = Все листы, по странице на лист
sheets-export = Экспорт:

## ======== Параметры компонента ========

rig-title = Параметры компонента
rig-select-hint = Выделите компонент (не группу) и выберите Окно → Параметры компонента или «Править параметры…» в Свойствах объекта.
rig-state-has = — с параметрами
rig-state-none = — параметров пока нет: «Применить» добавит их
rig-no-parts = В компоненте нет частей для движения: сгруппируйте его детали внутри (дверь, ящик…) — откройте его и используйте «Сгруппировать».

## Блок «Свойства объекта»
# $name — имя компонента; $n — число копий
rig-info-copies = { $name } — { $n ->
        [one] { $n } копия
        [few] { $n } копии
        [many] { $n } копий
       *[other] { $n } копии
    }
rig-no-params = Параметров пока нет
rig-reset = Сбросить
rig-reset-hint = Вернуть значения по умолчанию
rig-edit = Править параметры…
rig-edit-hint = Окно → Параметры компонента: параметры, формулы, движения частей, скрипт
# $n — число движений
rig-info-summary = Компонент с параметрами (движений: { $n })

## Шаги отмены и сообщения
rig-undo-reset = Сброс параметров
rig-undo-edit = Правка параметров
rig-undo-add = Добавление параметров
rig-undo-remove = Удаление параметров
rig-select-rigged = Выделите компоненты с параметрами
rig-select-component = Выделите компонент, чтобы задать ему параметры
rig-err-group = У групп не может быть параметров: сначала сделайте её компонентом (Правка → Создать компонент)
rig-err-generated = Этот компонент построен генератором или параметрической группой
rig-applied = Параметры применены
rig-removed = Параметры удалены
rig-err-place-copy = Поместите в модель копию компонента, чтобы выбрать его
rig-pick-axis = Щёлкните две точки на оси вращения
rig-pick-slide = Щёлкните две точки: направление сдвига
rig-err-same-points = Обе точки совпадают

## Таблица параметров
rig-parameters = Параметры
rig-parameters-hint = Значения, которые есть у каждой копии (их показывают Свойства объекта)
rig-col-name = Имя
rig-col-label = Подпись
rig-col-type = Тип
rig-col-min = Мин.
rig-col-max = Макс.
rig-col-step = Шаг
rig-col-default = По умолч.
rig-col-options = Варианты
rig-as-drawn = (как нарисовано)
rig-remove-param = Удалить параметр
rig-add-param = + Параметр

## Формулы
rig-formulas = Формулы
rig-formulas-hint = Значения, вычисляемые из параметров (выражения Rhai); их можно использовать в движениях и скрипте
rig-add-formula = + Формула

## Движения
rig-motions = Движения
rig-motions-hint = Часть (группа / компонент прямо внутри) поворачивается, сдвигается, показывается, красится или получает значение параметра (Задать параметр: у части, у которой сами есть параметры — генерируемый компонент вроде цилиндра из Рисование → Фигуры или компонент с параметрами) по параметру. Поворот / сдвиг: диапазон параметра отображается на От..До (градусы / длина); без диапазона или с выражением значение и есть движение
rig-motion-rotate = Повернуть
rig-motion-slide = Сдвинуть
rig-motion-show = Показать / скрыть
rig-motion-paint = Покрасить
rig-motion-set = Задать параметр
rig-parameter = (параметр)
rig-part-no-params = У части нет параметров
rig-part-param-hint = Собственный параметр части получает это значение
rig-by = по
rig-expression = (выражение)
rig-from = от
rig-to = до
rig-remove-motion = Удалить движение
rig-axis-through = Ось через
rig-along = вдоль
rig-direction = Направление
rig-pick = Выбрать…
rig-pick-hint = Щёлкните две точки в модели
rig-axes-note = мм, оси компонента
rig-add-motion = + Движение

## Скрипт
rig-script = Скрипт
rig-script-hint = Необязательный код Rhai, выполняется после движений внутри компонента: fn update(p) {"{"} … {"}"} с полным API скриптов (rotate, move, paint, show / hide, set_params, box…); child(name) / child(name, true) находит часть по имени (на любой глубине), part.min / max / size / bounds — её габариты; p — значения и формулы
rig-add-script = + Скрипт

## Кнопки
rig-add = Добавить параметры
rig-revert = Вернуть
rig-remove = Удалить параметры
rig-remove-hint = Копии снова показывают компонент как нарисованный

## ======== Генераторы ========

## Редактор генераторов
gen-editor-title = Редактор генераторов
gen-new = Новый
gen-new-hint = Начать с шаблона
gen-open = Открыть…
gen-filter = Генератор
gen-save = Сохранить в библиотеку
gen-save-hint = Записать генератор в { DATA_DIR() }/generators (он появится в лотке «Компоненты»)
gen-apply-hint = Обновить этот генератор документа: все его компоненты будут перестроены (один шаг отмены)
gen-apply-doc = Применить к документу
gen-apply-doc-hint = Использовать этот исходник в документе: его компоненты будут перестроены (один шаг отмены)
gen-place = Поместить
gen-place-hint = Поместить компонент со значениями из предпросмотра
# $name — имя генератора в документе
gen-in-model = В модели: { $name }
gen-not-saved = не сохранён
gen-build-failed = Не удалось построить
gen-drag-turn = Перетащите, чтобы повернуть
gen-defaults = По умолчанию
gen-no-home = Нет домашней папки
# $path — путь к файлу; $error — сообщение системы
gen-saved = Сохранено в { $path }
gen-cannot-save = Не удалось сохранить { $path }: { $error }
gen-name-taken = В документе уже есть генератор с именем { $name }
gen-undo-update = Обновление генератора { $name }
# $old и $name — имена генераторов
gen-renamed-applied = { $old } → { $name }: применено к документу
gen-same-source = { $name }: документ уже использует этот исходник
gen-applied = { $name }: применено к документу

## Список генераторов и плитки
# $label — подпись параметра
gen-not-length = { $label }: не длина
gen-formulas = Формулы
gen-origin-bundled = Встроенный пример (Правка → Сохранить в библиотеку создаёт вашу копию)
gen-origin-document = В этом документе
gen-tile-error = Ошибка: { $error }
# $note — описание и строки происхождения
gen-tile-tip =
    Генератор — параметры в Свойствах объекта
    { $note }
gen-edit = Править генератор…
gen-show-in-finder = Показать в файловом менеджере
    .macos = Показать в Finder
    .windows = Показать в Проводнике
gen-delete = Удалить…
gen-error = Генератор: { $error }

## Блок «Свойства объекта»
# $name — имя генератора; $n — число компонентов
gen-info-components = { $name } — { $n ->
        [one] { $n } компонент
        [few] { $n } компонента
        [many] { $n } компонентов
       *[other] { $n } компонента
    }
gen-info-generator = Генератор: { $name }
gen-reset-hint = Вернуть значения генератора по умолчанию
gen-unique-generated = Копия его генератора для этого компонента («Shelf 2» в «В модели»): правка этого скрипта меняет только его
gen-unique-hint = Дать этому компоненту собственное определение: его правка больше не меняет остальные копии
gen-unique-already = Эта копия уже уникальна: ничто другое не делит её определение или генератор
gen-unique-select = Выделите один компонент
gen-make-unique = Сделать уникальным
gen-mixed = Выделенные компоненты различаются; изменение задаст значение всем
gen-select-generated = Выделите сгенерированные компоненты

## ======== Сведения о документе ========

model-info-title = Сведения о документе
model-info-scene = В сцене
model-info-scene-hint = Что нарисовано: учтено содержимое каждой размещённой копии
model-info-file = В файле
model-info-file-hint = Что хранит документ: каждое определение один раз, используется оно или нет
model-info-libraries = Библиотеки
# $n — уже отформатированное число
model-info-files = файлов: { $n }
# $used, $all и $unused — отформатированные числа
model-info-used-of = { $used } из { $all } (не используется: { $unused })

## Строки
model-info-row-polygons = Полигоны (грани)
model-info-row-triangles = Треугольники
model-info-row-edges = Рёбра
model-info-row-groups-placed = Размещено групп
model-info-row-components-placed = Размещено компонентов
model-info-row-components = Компоненты
model-info-row-groups = Группы
model-info-row-materials = Материалы
model-info-row-textures = Текстуры
model-info-row-tags = Теги
model-info-row-scenes = Сцены
model-info-row-sheets = Листы компоновки
model-info-row-generators = Генераторы
model-info-row-lights = Источники света
model-info-row-guides = Направляющие / размеры
model-info-row-file-size = Размер файла (сохранённого)
model-info-row-lib-components = Библиотека компонентов
model-info-row-lib-materials = Библиотека материалов
model-info-preview-image = Изображение предпросмотра
model-info-none = нет
model-info-show = Показать…
model-info-show-hint = Файл → Изображение предпросмотра → Показать…

## Окно «Изображение предпросмотра»
model-info-preview-title = Изображение предпросмотра
model-info-no-preview = Нет изображения предпросмотра — библиотека компонентов рисует модель сама
model-info-preview-note = Сохраняется с файлом; библиотека компонентов показывает его для этого файла.
model-info-from-file = Из файла изображения…
model-info-from-view = Из текущего вида

## Удалить неиспользуемое
model-info-purge = Удалить неиспользуемое
model-info-purge-note = Удаляет то, что не используется в сцене (Правка → Отменить вернёт это).
# $n — число неиспользуемых определений / материалов
model-info-purge-defs = Компоненты и группы ({ $n })
model-info-purge-mats = Материалы ({ $n })
model-info-nothing-to-purge = Нет неиспользуемого для удаления
model-info-purge-failed = Не удалось удалить
# $n — количество
model-info-purged-components = компонентов: { $n }
model-info-purged-groups = групп: { $n }
model-info-purged-materials = материалов: { $n }
# $parts — список из трёх текстов выше через запятую
model-info-purged = Удалено: { $parts }

## ======== Плагины ========

## Ошибки заголовка плагина (показываются авторам плагинов)
# $label — подпись параметра; $value — введённый текст
ext-param-invalid = { $label }: недопустимое значение: { $value }
# $text — строка @param
ext-param-needs-default = @param требует `name = default`: { $text }
ext-param-bad-name = недопустимое имя параметра `{ $key }`
ext-param-unclosed-quote = незакрытая кавычка: { $text }
ext-param-unknown-type = неизвестный тип параметра `{ $ty }` (length, int, number, text, bool)
ext-pick-needs-number = @pick требует число точек: { $text }
ext-pick-range = @pick: от 1 до 16 точек
ext-pick-default = { $n ->
        [one] Щёлкните { $n } точку
        [few] Щёлкните { $n } точки
        [many] Щёлкните { $n } точек
       *[other] Щёлкните { $n } точки
    }
ext-select-default = Выделите объекты
ext-empty-name = пустой @name

## Плагины материалов
ext-mat-plugin-failed = плагин завершился с ошибкой
ext-mat-no-answer = плагин не вывел ответ
# $error — сообщение разборщика JSON; $answer — начало ответа плагина
ext-mat-not-json = его ответ не JSON ({ $error }): { $answer }
ext-mat-catalogue-array = catalogue() должна вернуть массив материалов
ext-mat-no-home = нет домашней папки
ext-mat-default-name = Материал
ext-mat-import-nothing = import_material() ничего не вернула
ext-tex-bad-size = Встроенное изображение: "size" — одна сторона или [ширина, высота] от 1 до 8192 пикселей
ext-tex-no-pixels = Встроенное изображение: передайте пиксели в "rgb", "rgba" или "grey" (массив 0..255, base64 или blob)
ext-tex-pixel-count = Встроенное изображение: ожидалось байт: { $want }, передано: { $got }
ext-mat-spec-object = описание материала должно быть объектом

## Меню «Плагины» и диалоги плагинов
ext-script-undo = Скрипт
# $error — сообщение разборщика заголовка плагина
ext-header-error = заголовок: { $error }
ext-no-plugins = Нет плагинов
# $n — число плагинов
ext-failed-to-load = Не загружено плагинов: { $n }…
ext-reload = Перезагрузить плагины
ext-open-folder = Открыть папку плагинов
ext-script-gone = скрипт пропал
# $name — имя плагина или скрипта (строки журнала)
ext-run-script = скрипт { $name }
ext-run-plugin = плагин { $name }
ext-plugin-failed = плагин завершился с ошибкой
ext-done = { $name }: готово
ext-exited = { $name }: завершился с ошибкой ({ $status })
ext-python-no-bridge = Python-скриптам нужен мост (выключен через VARSTOTAS_NO_BRIDGE)
ext-python-busy = Python-скрипт уже выполняется
ext-python-cant-start = Не удалось запустить { $python }: { $error } (задайте VARSTOTAS_PYTHON)
ext-pick-points = Выбор точек
ext-select-objects = Выбор объектов
ext-run = Запустить
## ======== Консоль скриптов и Справочник по скриптам ========

## Консоль скриптов
console-title = Консоль скриптов
console-script-failed = скрипт завершился с ошибкой
# $error — текст ошибки скрипта
console-script-error = Ошибка скрипта: { $error }
# $name — имя файла скрипта; это название шага отмены
console-script-undo = Скрипт { $name }
# $path — путь к файлу; $error — сообщение системы
console-cannot-read = Не удаётся прочитать { $path }: { $error }
console-hint = Скрипты Rhai над моделью. Попробуйте: box(ORIGIN, 1000, 600, 18)  — %CMD%+Enter запускает
console-run = Запуск
console-run-hint = %CMD%+Enter
console-open = Открыть…
console-save = Сохранить…
console-stop-python = Остановить Python
console-clear = Очистить вывод
# $port — номер TCP-порта
console-bridge = Мост Python / MCP: 127.0.0.1:{ $port }
console-bridge-hint = import varstotas; m = varstotas.connect()  — см. { DATA_DIR() }/python
console-filter-rhai = Скрипт Rhai

## Окно Справочника по скриптам
console-ref-title = Справочник по скриптам
console-search = Поиск
console-search-hint = функция, свойство или слово
console-fulltext = Полный текст: docs/SCRIPTING.md, docs/GENERATORS.md
console-copy-example = Копировать пример
console-click-copy = Щёлкните, чтобы копировать
console-topic-basics = Основы
console-topic-console = Консоль и плагины
console-topic-generators = Генераторы
console-topic-component = Скрипт компонента
console-topic-functions = Функции
console-topic-properties = Свойства
console-param-types = Типы параметров
console-generators-note = Внутри `build` работает весь API (см. Функции). Скрипт выполняется в пустой модели с материалами и тегами документа; всё, что он построит, становится содержимым компонента.
console-functions-intro = Все функции API по разделам. Щёлкните сигнатуру, чтобы копировать. `x` — объект, массив объектов или имя дочерней группы.
console-properties-intro = После точки: `g.name`, `f.normal.x`, `door.size.z`, `[1, 2].len()`. Свойства с пометкой «присваиваемое» можно менять: `g.name = "Crate"`, `g.hidden = true`.
console-kind-objects = Объекты (грани, рёбра, группы, компоненты)
console-kind-vectors = Векторы
console-kind-bounds = Границы — bounds(x), x.bounds
console-kind-arrays = Массивы
console-kind-strings = Строки
console-nothing-found = Ничего не найдено
console-members = Свойства и методы
console-tags = Теги заголовка
console-reference = ? Справочник
console-reference-hint = Справка → Справочник по скриптам: функции, свойства, примеры (F1 в редакторе: слово под курсором)

## Страница: основы
console-ref-basics-1 = Скрипты пишутся на Rhai — небольшом языке с синтаксисом, близким к Rust / JavaScript. Координаты — мировые миллиметры; длины с единицами — строки: `length("1.2m")`.
console-ref-basics-2 = Язык
console-ref-basics-4 = Строки: `"text"`, склеиваются через `+`: `"w = " + w`. Методы строк `trim()`, `to_upper()` меняют строку на месте и ничего не возвращают: `s.trim(); if s == "" {"{"} … {"}"}`.
console-ref-basics-5 = Константы: `ORIGIN`, `X`, `Y`, `Z` (единичные оси). Векторы можно складывать, вычитать и масштабировать: `ORIGIN + X * 100`; `.x .y .z`, `length()`, `normalize()`, `dot(b)`, `cross(b)`, `distance(b)`.
console-ref-basics-6 = Объекты
console-ref-basics-7 = Грани, рёбра, группы и компоненты — это ссылки: `let g = box(ORIGIN, 100, 100, 100); g.name = "Crate";`. После операций, которые сливают или разрезают геометрию, старая ссылка может стать недействительной — проверяйте `e.valid`.
console-ref-basics-8 = Везде, где ожидается объект, строка задаёт имя группы / компонента текущего контекста: `move("Door", vec3(0, 10, 0))`. `child("Knob", true)` ищет и во вложенных группах.
console-ref-basics-9 = У каждого объекта есть `kind`, `bounds`, `size`, `min`, `max`, `center`, `hidden`; у групп ещё `name`, `definition`, `material`, `tag` (см. Свойства).
console-ref-basics-10 = Контекст и отмена
console-ref-basics-11 = Новая геометрия попадает в текущий контекст: группу, открытую при запуске скрипта, или ту, в которую вошли через `enter(g)` (выход — `exit()`).
console-ref-basics-12 = Запуск — один шаг отмены. Скрипт с ошибкой ничего не меняет; ошибка со строкой показывается и сохраняется в Окно → Журнал.
console-ref-basics-13 = `print(x)` пишет в вывод; значение последнего выражения тоже печатается. Скрипты прерываются через 30 с (генераторы — через 10 с).
## Страница: консоль
console-ref-console-1 = Окно → Консоль скриптов выполняет код над открытой моделью. %CMD%+Enter запускает, %CMD%+↑ / ↓ листает прежние запуски, Открыть… / Сохранить… работают с файлами `.rhai`. Файл → Выполнить скрипт… запускает файл.
console-ref-console-2 = Автодополнение показывает функции API при вводе, свойства после точки (по типу слева) и сигнатуру вызываемой функции над курсором. F1 открывает этот справочник на слове под курсором.
console-ref-console-3 = Заголовок: диалоги и пункты меню
console-ref-console-4 = Строки `// @tag …` в начале превращают скрипт в плагин: при запуске сначала спрашиваются параметры, точки и объекты. Файл в `{ DATA_DIR() }/plugins` становится пунктом меню «Плагины».
console-ref-console-6 = Теги
## Страница: генераторы
console-ref-generators-1 = Генератор — это компонент с параметрами: скрипт, у которого `build(p)` строит содержимое компонента в его собственных осях (начало координат — точка вставки). У каждой копии свои значения в разделе «Свойства объекта»; копии с равными значениями используют одно определение.
console-ref-generators-2 = Окно → Редактор генераторов (или Править генератор… на плитке): слева исходник, справа живой предпросмотр с полями параметров; «Применить» обновляет документ, «Сохранить в библиотеку» записывает файл.
console-ref-generators-4 = Заголовок
console-ref-generators-5 = обязательно; ключ генератора в документе
console-ref-generators-6 = значение в `p`; любая часть диапазона необязательна
console-ref-generators-7 = значение, вычисляемое из параметров (только чтение в «Свойствах объекта»)
console-ref-generators-8 = значения, для которых выражение ложно, отклоняются
console-ref-generators-9 = Типы параметров
## Страница: компонент
console-ref-component-1 = Окно → Параметры компонента добавляет любому компоненту параметры: строки значений, формулы, движения его частей (поворот, сдвиг, показ, окраска, установка параметра части) и необязательный скрипт.
console-ref-component-2 = Скрипт определяет `update(p)`. Он выполняется после движений, внутри компонента, с полным API скриптов — теми же функциями, что в консоли и генераторах. В `p` лежат значения и формулы.
console-ref-component-4 = Поиск частей
console-ref-component-5 = группа / компонент прямо внутри компонента
console-ref-component-6 = ищется и во вложенных группах (побеждает ближайшая)
console-ref-component-7 = внутри части
console-ref-component-8 = все части (и вложенные)
console-ref-component-9 = текущие габариты части (мировые)
console-ref-component-10 = скрыть, как hide(part)
console-ref-component-11 = Как это работает
console-ref-component-12 = Скрипт меняет копию (вариант для этих значений), но не оригинал: изменённые им части копируются в вариант, нетронутые остаются общими. Копии с равными значениями используют один вариант.
console-ref-component-13 = Правки внутри копии (двойной щелчок) возвращаются в оригинал без того, что сделал скрипт: добавленное отбрасывается, перемещения / скрытие / окраска отменяются.
console-ref-component-14 = Ошибки показываются при нажатии «Применить» и сохраняются в Окно → Журнал.

## Редактор кода
# $sel — номер выбранного варианта, $n — число вариантов
console-completion-count = { $sel } из { $n }

## ======== Журнал ========

log-level-error = ошибка
log-level-warning = предупреждение
log-title = Журнал
# $n — количество
log-errors = Ошибки { $n }
log-warnings = Предупреждения { $n }
log-filter = фильтр
log-clear = Очистить
log-copy = Копировать
log-copy-hint = Копировать показанные записи
log-empty = В этом сеансе нет ошибок и предупреждений.
log-no-match = Ничего не подходит под фильтры.
# $n — число ещё не просмотренных ошибок
log-badge-errors = ● { $n ->
        [one] { $n } ошибка
        [few] { $n } ошибки
        [many] { $n } ошибок
       *[other] { $n } ошибки
    }
log-badge = Журнал
log-badge-hint = Окно → Журнал: ошибки и предупреждения отовсюду (скрипты, импорт / экспорт, библиотеки …)

## ======== Лицензии ========

licences-tab-varstotas = Varstotas
licences-tab-components = Открытый код
licences-tab-formats = Форматы файлов
licences-tab-libraries = Онлайн-библиотеки

licence-cc0 = CC0
licence-cc-by = CC-BY
licence-cc-by-sa = CC-BY-SA
licence-cc-by-nc = CC-BY-NC
licence-cc-by-nd = CC-BY-ND
licence-standard = Стандартная / личное использование
licence-unknown = Неизвестна

licence-cc-by-nc-note = некоммерческая: не продавайте то, что с ней сделано
licence-cc-by-nd-note = без производных: не распространяйте изменённую копию
licence-cc-by-sa-note = на тех же условиях: изменённые копии распространяйте под той же лицензией
licence-standard-note = собственная лицензия сайта (бесплатно / личное использование / условия использования): часто нельзя коммерчески или распространять сам файл модели — проверьте
licence-unknown-note = лицензия неизвестна: только для личного использования

# Окно лицензий: подписи и пояснения к форматам и онлайн-библиотекам (сами тексты лицензий не переводятся).

## Окно
licences-title = Лицензии
licences-count = { $n ->
        [one] { $n } компонент
        [few] { $n } компонента
        [many] { $n } компонентов
       *[other] { $n } компонента
    }
licences-filter = Фильтр: название или лицензия
licences-save = Сохранить как текст…
licences-save-hint = Все компоненты и тексты их лицензий в одном файле
licences-saved = Сохранено: { $path }
licences-pick = Выберите компонент, чтобы увидеть его лицензию.
licences-all-text-title = Varstotas использует следующие компоненты с открытым кодом.
licences-applies-to = Относится к: { $users }

## Вкладка Varstotas
licences-own = Varstotas — свободное ПО под лицензией MIT: его можно использовать для личной и коммерческой работы, копировать, изменять и распространять.
licences-programs = Программы, которые используются, если они у вас есть
licences-yours = Ваши модели принадлежат вам. Модели и материалы, скачанные из онлайн-библиотек, сохраняют свои лицензии (Онлайн-библиотеки; Файл → Экспорт → Авторство).

## Программы


licence-program-oidn-use = ИИ-шумоподавление рендера
licence-program-oidn-licence = Apache-2.0 — не входит в поставку, загружается из вашей установки

licence-program-python-use = плагины и скрипты на Python
licence-program-python-licence = лицензия PSF — не входит в поставку, ваша установка

## Вкладка «Форматы файлов»
licence-formats-col-format = Формат
licence-formats-col-varstotas = Varstotas
licence-formats-col-how = Как
licence-formats-col-note = Формат

licence-format-varstotas-what = открытие, сохранение
licence-format-varstotas-how = собственный формат Varstotas
licence-format-varstotas-note = MIT, как и Varstotas

licence-format-gltf-what = импорт, экспорт
licence-format-gltf-how = собственный читатель / писатель; сетки Draco через draco-core (Apache-2.0)
licence-format-gltf-note = открытый стандарт Khronos, без лицензионных платежей

licence-format-obj-what = импорт, экспорт
licence-format-obj-how = собственный читатель / писатель
licence-format-obj-note = открытая публичная спецификация

licence-format-stl-what = импорт, экспорт
licence-format-stl-how = собственный читатель / писатель
licence-format-stl-note = открытая публичная спецификация

licence-format-ply-what = импорт, экспорт
licence-format-ply-how = собственный читатель / писатель
licence-format-ply-note = открытая публичная спецификация

licence-format-3mf-what = импорт, экспорт
licence-format-3mf-how = собственный читатель / писатель (zip + XML)
licence-format-3mf-note = спецификация 3MF Consortium, без лицензионных платежей

licence-format-collada-what = импорт, экспорт
licence-format-collada-how = собственный читатель / писатель (изображения — файлами рядом)
licence-format-collada-note = открытый стандарт Khronos, без лицензионных платежей

licence-format-fbx-what = импорт
licence-format-fbx-how = ufbx (MIT / Unlicense), независимый читатель
licence-format-fbx-note = формат Autodesk; FBX — торговая марка Autodesk, Varstotas с ней не связан

licence-format-dxf-what = импорт, экспорт
licence-format-dxf-how = собственный читатель / писатель
licence-format-dxf-note = открыто опубликованный формат Autodesk; DXF — торговая марка Autodesk

licence-format-svg-pdf-what = экспорт (чертежи, развёртки, листы)
licence-format-svg-pdf-how = собственные писатели
licence-format-svg-pdf-note = открытые стандарты W3C / ISO

licence-format-images-what = текстуры, изображения, окружения
licence-format-images-how = крейт image (MIT / Apache-2.0)
licence-format-images-note = открытые форматы

licence-format-openexr-what = экспорт рендера
licence-format-openexr-how = собственный писатель
licence-format-openexr-note = Academy Software Foundation, BSD-3-Clause

licence-format-csv-json-what = импорт точек, экспорт граней (плагины)
licence-format-csv-json-how = собственный код
licence-format-csv-json-note = открытые форматы

licence-format-gif-what = экспорт анимации
licence-format-gif-how = крейт image (MIT / Apache-2.0)
licence-format-gif-note = открытый формат

## Вкладка «Онлайн-библиотеки»
licence-libs-intro = Каждая скачанная модель хранит свой источник, автора и лицензию; Varstotas предупреждает, если лицензия размещаемой или экспортируемой модели ограничивает её использование (некоммерческая, без производных, на тех же условиях, собственная лицензия сайта, неизвестная).
licence-libs-col-library = Библиотека
licence-libs-col-assets = Ресурсы
licence-libs-col-licence = Лицензия
licence-libs-col-terms = Условия

licence-lib-poly-haven-what = модели, материалы
licence-lib-poly-haven-licence = CC0
licence-lib-poly-haven-terms = Бесплатный открытый API, в том числе для коммерческих приложений; просят показывать «Powered by Poly Haven» (это сделано над ресурсами) и уникальный User-Agent (Varstotas).

licence-lib-ambientcg-what = материалы
licence-lib-ambientcg-licence = CC0
licence-lib-ambientcg-terms = Бесплатные API и загрузки; упоминание автора приветствуется, но не обязательно.

licence-lib-smithsonian-what = модели
licence-lib-smithsonian-licence = в основном CC0; у некоторых «действуют условия использования» (некоммерческие)
licence-lib-smithsonian-terms = API не сообщает, какие именно: нужно смотреть страницу модели. Varstotas помечает такие модели как «проверьте условия» и предупреждает при размещении или экспорте.

licence-lib-nasa-what = модели
licence-lib-nasa-licence = не охраняется авторским правом (общественное достояние США)
licence-lib-nasa-terms = Эмблемы и логотипы NASA защищены; использование не должно подразумевать одобрение со стороны NASA.

licence-lib-kenney-what = наборы моделей
licence-lib-kenney-licence = CC0
licence-lib-kenney-terms = Читается с общедоступных страниц ресурсов kenney.nl; логотип Kenney не под CC0.

licence-lib-quaternius-what = наборы моделей
licence-lib-quaternius-licence = CC0
licence-lib-quaternius-terms = Скачивается вами в браузере.

licence-lib-search-name = Поиск на сайтах (Printables, Thingiverse, Sketchfab, …)

licence-lib-search-what = модели
licence-lib-search-licence = у каждой модели своя
licence-lib-search-terms = Varstotas лишь открывает поиск сайта в браузере; скачиваете вы сами и указываете лицензию в карточке импорта.

licence-lib-plugins-name = Плагины

licence-lib-plugins-what = модели, материалы
licence-lib-plugins-licence = у каждого плагина своя
licence-lib-plugins-terms = Плагин-источник показывает свои условия (строку @notice) над своими ресурсами; например, для Sketchfab нужен ваш собственный токен учётной записи и указание автора и Sketchfab.

## ======== Руководство ========

help-topic-start = Начало работы
help-topic-navigation = Навигация по виду
help-topic-selecting = Выделение
help-topic-drawing = Рисование
help-topic-shapes = Объёмные фигуры
help-topic-modifying = Изменение
help-topic-inference = Привязки
help-topic-measurements = Поле значений
help-topic-groups = Группы и компоненты
help-topic-library = Библиотека компонентов
help-topic-materials = Материалы и текстуры
help-topic-organizing = Теги, Структура, Свойства объекта
help-topic-construction = Направляющие, размеры, сечения
help-topic-styles = Стили и рендер
help-topic-scenes = Сцены, листы, чертежи
help-topic-files = Файлы, импорт и экспорт
help-topic-scripts = Скрипты и плагины
help-topic-shortcuts = Клавиатура и мышь
help-topic-problems = Если что-то пошло не так
## start
help-start-1 = Varstotas — 3D-редактор: вы рисуете рёбра, замкнутые рёбра образуют грани, грани вытягиваются и вдавливаются в тела. Всё слипается, пока вы не сгруппируете.
help-start-2 = Окно
help-start-3 = Строка меню
help-start-4 = Файл, Правка, Вид, Камера, Рисование, Инструменты, Плагины, Окно, Справка
help-start-5 = Панель инструментов
help-start-6 = инструменты (подсказка называет каждый и его клавишу), навигация и привязки; виды и стили — в правом верхнем углу видового экрана
help-start-7 = Видовой экран
help-start-8 = модель; красная, зелёная и синяя линии — оси, под ними земля
help-start-9 = Боковая панель
help-start-10 = карточки справа: Свойства объекта, Материалы, Компоненты, Теги, Структура, Сцены, Тени, Окружение, Сглаживание рёбер, Подсказки (Окно → Боковая панель включает и выключает карточки)
help-start-11 = Строка состояния
help-start-12 = что ожидает активный инструмент, сообщения и ошибки, кнопка «Журнал», поле значений
help-start-13 = Первая модель
help-start-14 = 1. Нажмите `R` (Прямоугольник), щёлкните угол на земле, затем противоположный угол — или введите `1000,600` и Enter.
help-start-15 = 2. Нажмите `P` (Тяни/Толкай), щёлкните прямоугольник, потяните вверх и щёлкните — или введите `750` и Enter.
help-start-16 = 3. Нажмите `Space` (Выделение), трижды щёлкните параллелепипед, чтобы выделить его целиком, и нажмите `G`, чтобы сгруппировать.
help-start-17 = 4. Нажмите `B` (Заливка), выберите материал в карточке «Материалы», щёлкайте грани.
help-start-18 = 5. `Shift+Z` вписывает модель в вид; Файл → Сохранить записывает файл `.vrst`.
help-start-19 = Помощь по ходу работы
help-start-20 = Карточка «Подсказки» объясняет активный инструмент; F1 над моделью открывает это руководство на нём. Готовые модели для просмотра (обставленный дом, галерея материалов) лежат в папке { DATA_DIR() }/examples: Файл → Открыть.
## navigation
help-navigation-1 = Мышь и трекпад
help-navigation-2 = Перетаскивание средней кнопкой
help-navigation-3 = вращение вокруг точки под курсором
help-navigation-4 = Shift + перетаскивание средней кнопкой
help-navigation-5 = панорама
help-navigation-6 = Колесо
help-navigation-7 = масштаб к курсору
help-navigation-8 = Трекпад: смахивание двумя пальцами
help-navigation-9 = вращение; с Shift — панорама
help-navigation-10 = Трекпад: щипок или %CMD% + смахивание
help-navigation-11 = масштаб
help-navigation-12 = Это работает в любом инструменте, поэтому осмотреться можно, не прерывая рисования.
help-navigation-13 = Окно → Настройки → Навигация меняет их: скорость вращения и масштаба, обратное колесо, масштаб к центру вида вместо курсора, панорама средней кнопкой (вращение с Shift), панорама или масштаб двумя пальцами.
help-navigation-14 = Виды
help-navigation-15 = Камера → Стандартные виды (и выпадающий список вида в углу видового экрана): Изометрия, Сверху, Снизу, Спереди, Сзади, Слева, Справа. Камера → Параллельная проекция / Перспектива. `Shift+Z` — Показать всё: вся модель в виде.
help-navigation-16 = Сохранённые виды — это сцены; см. «Сцены, листы, чертежи».
help-navigation-17 = Собственные элементы управления видового экрана
help-navigation-18 = Три выпадающих списка со значками в правом верхнем углу видового экрана содержат всё о том, как он показывает модель (наведите курсор, чтобы увидеть текущее значение):
help-navigation-19 = Вид
help-navigation-20 = стандартные виды, Перспектива / Параллельная проекция, Поле зрения, Показать всё, Видовые экраны 1 / 2 / 4
help-navigation-21 = Стиль
help-navigation-22 = С заливкой … Реалистичный; Рентген, Рёбра, Контуры
help-navigation-23 = Показать (глаз)
help-navigation-24 = оси, земля, небо (выкл. = простой светлый фон), сетка этого вида; направляющие, тени, секущие плоскости / разрезы / заливка сечений модели (для всех видов)
help-navigation-25 = Переключатели не закрывают меню; щелчок снаружи закрывает его.
help-navigation-26 = Несколько видовых экранов
help-navigation-27 = Вид → Видовые экраны (или выпадающий список вида) показывает 1, 2 (рядом) или 4 (2 × 2) вида модели. У каждого своя камера, стиль и переключатели вида; новые начинаются сверху, спереди и справа, в параллельной проекции. Щелчок по виду делает его активным (синяя рамка): меню, панели и инструменты действуют на него, и сам щелчок уже срабатывает там.
help-navigation-28 = Пресеты (Вид → Видовые экраны или «Пресеты видовых экранов» в выпадающем списке вида) сразу задают типовые раскладки, например Спереди + Изометрия или Спереди, Слева, Сверху + Изометрия: плоские виды в параллельной проекции, изометрия — в перспективе.
## selecting
help-selecting-1 = Правка → Выделить всё (`%CMDK%A`), Снять выделение (`%CMDK%T`). Delete или Backspace стирает выделение; Правка → Скрыть скрывает его, «Показать всё скрытое» возвращает всё.
help-selecting-2 = Щёлкните модель правой кнопкой для контекстного меню: Стереть, Скрыть, Сгруппировать, Создать компонент…, Разбить, Пересечь грани, Обратить / Ориентировать грани, Отразить по, Текстура → Положение, Развёртка…, а также команды секущих плоскостей и осей.
help-selecting-3 = Карточка «Структура» показывает группы и компоненты деревом: щелчок выделяет, двойной щелчок приближает к объекту.
## drawing
help-drawing-1 = Инструменты рисования работают на грани под курсором либо на земле / активной рабочей плоскости. Замкнутые контуры из рёбер становятся гранями; линия через грань разрезает её.
help-drawing-2 = Пока инструмент работает, значение можно вводить в любой момент (щёлкать по полю значений не нужно) и нажимать Enter — см. «Поле значений».
## shapes
help-shapes-1 = Рисование → Фигуры строит целое тело одной операцией. Результат — генерируемый компонент: выделите его и меняйте размеры и сегменты в «Свойствах объекта»; «Разбить» превращает его в обычную геометрию.
## modifying
help-modifying-1 = Перемещение, Поворот и Масштаб работают с выделением или с тем, что вы щёлкнули, если ничего не выделено. Перемещение вершины, ребра или грани растягивает геометрию вокруг неё.
help-modifying-2 = Копирование
help-modifying-3 = Перемещение / Поворот с %COPY% создаёт копию
help-modifying-4 = Массивы
help-modifying-5 = после копии введите `5x` (пять копий с тем же шагом) или `/5` (пять копий между)
help-modifying-6 = Меню «Правка» и контекстное меню
help-modifying-7 = Пересечь грани
help-modifying-8 = с моделью, выделением или контекстом: рёбра там, где грани пересекаются
help-modifying-9 = Разрез
help-modifying-10 = плоскость разрезает выделение или контекст; с %COPY% (разделение) каждое разрезанное тело становится двумя замкнутыми группами
help-modifying-11 = Обратить / Ориентировать грани
help-modifying-12 = повернуть лицевую сторону (белую) наружу
help-modifying-13 = Отразить по
help-modifying-14 = отразить выделение вдоль красного, зелёного или синего направления
help-modifying-15 = Карточка «Сглаживание рёбер»
help-modifying-16 = смягчать / сглаживать рёбра по углу
help-modifying-17 = Окно → Развёртка
help-modifying-18 = развернуть грани или группы в выкройку (SVG / PDF / DXF)
## inference
help-inference-1 = При рисовании курсор привязывается к точкам и направлениям модели; маркер и подсказка сообщают, что найдено.
help-inference-2 = Конечная точка, Середина
help-inference-3 = зелёная / голубая точка
help-inference-4 = На ребре, На грани
help-inference-5 = красный квадрат / синий ромб
help-inference-6 = Центр, Пересечение
help-inference-7 = окружностей и дуг, рёбер
help-inference-8 = На красной / зелёной / синей оси
help-inference-9 = линия цвета оси
help-inference-10 = Параллельно / Перпендикулярно
help-inference-11 = пурпурная линия вдоль ребра
help-inference-12 = От точки
help-inference-13 = пунктирная линия от точки вдоль оси
help-inference-14 = Фиксация
help-inference-15 = Удерживайте `Shift`, чтобы зафиксировать найденное направление; стрелки фиксируют ось: → красная, ← зелёная, ↑ синяя, ↓ параллельно / перпендикулярно последнему ребру.
help-inference-16 = Меню «Привязки»
help-inference-17 = Кнопка-магнит (и меню «Привязки») включает и выключает каждую привязку; `Shift+S` выключает и включает привязки целиком. Шаги округляют свободные перемещения (10 мм), углы (15°) и коэффициенты масштаба — привязки к геометрии и введённые значения важнее их.
help-inference-18 = Вид → Сетка показывает сетку со своими привязками; шаг задаётся в Окно → Настройки → Рисование, а при активном «Выделении» ввод длины задаёт его.
help-inference-19 = Рабочие плоскости
help-inference-20 = Инструмент «Направляющая плоскость» (меню «Инструменты») и Окно → Рабочие плоскости создают плоскость для рисования в воздухе: точки в пустом пространстве ложатся на неё, оси и сетка следуют за ней.
## measurements
help-measurements-1 = Поле значений справа в строке состояния показывает текущую длину, угол или размер. Просто вводите значение во время работы инструмента и нажмите Enter; Backspace исправляет, Esc очищает.
help-measurements-2 = 1200, 1.2m, 120cm, 4', 1'6"
help-measurements-3 = длины: единица модели или названная вами единица
help-measurements-4 = 1000,600
help-measurements-5 = два значения: стороны прямоугольника, основание параллелепипеда
help-measurements-6 = 45
help-measurements-7 = угол (Поворот, Транспортир, дуги)
help-measurements-8 = 24s
help-measurements-9 = сегменты окружности, дуги или фигуры; стороны многоугольника
help-measurements-10 = 5x, /5
help-measurements-11 = копии в ряд после копии Перемещением / Поворотом
help-measurements-12 = 2, 0.5
help-measurements-13 = коэффициент масштаба
help-measurements-14 = Повторный ввод сразу после операции меняет эту операцию (другая длина, больше копий).
help-measurements-15 = Единица модели берётся из шаблона (Файл → Создать из шаблона); в качестве десятичного знака подходят и `,`, и `.`.
## groups
help-groups-1 = Геометрия слипается: касающиеся рёбра и грани сливаются и разрезают друг друга. Чтобы часть осталась отдельной, сделайте её группой или компонентом.
help-groups-2 = Сгруппировать (G)
help-groups-3 = группа: один объект; копии независимы
help-groups-4 = Создать компонент…
help-groups-5 = компонент: копии используют одно определение — правка одной меняет все
help-groups-6 = Двойной щелчок
help-groups-7 = войти в группу / компонент для правки (остальное затемняется); Esc или Правка → Закрыть группу выходит
help-groups-8 = Разбить
help-groups-9 = обратно в россыпь геометрии
help-groups-10 = Сделать уникальным (правый щелчок, Свойства объекта)
help-groups-11 = копия компонента получает собственное определение («Box 2»): правка больше не меняет остальные копии; у генерируемого копируется и генератор («Shelf 2» в разделе «В модели»)
help-groups-12 = Повторная вставка из библиотеки
help-groups-13 = копия в модели используется, пока не изменена; после её правки оригинал из библиотеки приходит как новое определение («Chair 2»)
help-groups-14 = Компоненты с параметрами
help-groups-15 = Правка → Сделать параметрическим даёт компоненту размеры X / Y / Z (растяжение или масштаб) и ряд копий в «Свойствах объекта».
help-groups-16 = Окно → Параметры компонента добавляет любому компоненту параметры: значения и формулы, которые поворачивают, сдвигают, показывают или красят его части, плюс необязательный скрипт; у каждой копии свои значения (ползунки в «Свойствах объекта»).
help-groups-17 = Генераторы — это компоненты, построенные скриптом с типизированными параметрами (Окно → Редактор генераторов). Тела из Рисование → Фигуры тоже генераторы.
help-groups-18 = Скрипт компонента в Справочнике по скриптам
help-groups-19 = Генераторы в Справочнике по скриптам
## library
help-library-1 = Карточки «Компоненты» и «Материалы» устроены одинаково: верхний ряд (Поиск, Обновить, + для добавления, кнопка-облако для онлайн-библиотеки, … для папки библиотеки, что показывать и размер плиток: Мелкие / Крупные / Список), затем разделы «В модели» (открытый документ), «Избранное», «Недавние», «Моя библиотека» с её папками и «Примеры». Раздел сворачивается стрелкой; имя папки открывает её в Finder.
help-library-2 = Правый щелчок по плитке → Добавить в избранное
help-library-3 = плитка с ☆ также показывается в «Избранном» вверху («Убрать из избранного» убирает её); «Недавние» перечисляют то, что вы недавно вставляли или выбирали для окраски
help-library-4 = Правый щелчок по имени папки
help-library-5 = Новая папка…; для вложенной ещё Переименовать папку… и Удалить папку (только пустую)
help-library-6 = Правый щелчок по плитке → Переместить в папку
help-library-7 = переместить файл в другую папку библиотеки (избранное следует за ним)
help-library-8 = Перетащите плитку на видовой экран
help-library-9 = компонент встаёт туда, куда вы его бросили; материал красит грань под курсором
help-library-10 = «Компоненты» перечисляют вашу библиотеку (`{ DATA_DIR() }/components` или папку, выбранную через …), генераторы, встроенные «Примеры» и компоненты открытого документа.
help-library-11 = Щёлкните плитку
help-library-12 = вставить: копия следует за курсором, щелчок ставит её на место
help-library-13 = + → Сохранить выделение в библиотеку…
help-library-14 = записать выделение новым файлом `.vrst`
help-library-15 = Правый щелчок по плитке
help-library-16 = Вставить, Задать превью из вида, Задать изображение превью…, Убрать изображение превью, Переименовать…, Показать в Finder, Удалить…; для компонента документа ещё Выделить экземпляры, Сохранить в библиотеку, Параметры…, Удалить из документа…
help-library-17 = Картинки
help-library-18 = Плитка показывает картинку файла, если она есть, иначе — отрисованную миниатюру. «Задать превью из вида» берёт видовой экран как есть — сначала скадрируйте компонент. Компонент, вставленный из библиотеки, берёт картинку файла с собой; Файл → Изображение превью задаёт картинку открытого документа.
help-library-19 = Онлайн-библиотека 3D
help-library-20 = Кнопка-облако в карточке «Компоненты» (или Окно → Онлайн-библиотека 3D) скачивает модели: Poly Haven, Smithsonian, NASA, наборы Kenney, по URL или из плагинов-источников. Скачанная модель становится компонентом библиотеки с указанием автора (Свойства объекта → Источник, Файл → Экспорт → Авторство). Фильтры по лицензии, формату и размеру; избранное и история загрузок.
help-library-21 = Каждый источник показывает свои условия над моделями (строка @notice плагина). Модель, лицензия которой ограничивает использование (некоммерческая, без производных, на тех же условиях, собственная лицензия сайта, неизвестная), получает предупреждение при вставке и при экспорте документа. Справка → Лицензии… перечисляет лицензию Varstotas, компоненты с открытым кодом, форматы файлов и онлайн-библиотеки с их условиями.
## materials
help-materials-1 = Карточка «Материалы» показывает материалы модели (В модели), вашу библиотеку (`{ DATA_DIR() }/materials` или папку, выбранную через …) и «Примеры»; правый щелчок по плитке — Использовать, Переименовать…, Сохранить в библиотеку, Показать в Finder, Экспорт…, Удалить…. Выбор материала из библиотеки лишь готовит «Заливку»: материал попадает в модель (В модели) с первой окрашенной гранью. «Удалить» на плитке «В модели» убирает материал из модели, его грани возвращаются к умолчанию (Правка → Отменить вернёт его). Материал правится там же: цвет, непрозрачность, изображение текстуры и её размер, физические свойства для стиля «Реалистичный» (металл, шероховатость, стекло, свечение, карты, лак, ворс).
help-materials-2 = Сохранить в библиотеку / Экспорт… / + → Импорт…
help-materials-3 = передать материал между документами (`.vrstmat`)
help-materials-4 = Кнопка-облако (карточка «Материалы»)
help-materials-5 = скачивать PBR-сканы из Poly Haven и ambientCG или из плагинов-источников
help-materials-6 = Шейдер
help-materials-7 = у материала может быть свой шейдер WGSL с параметрами (у редактора шейдеров есть собственный справочник)
help-materials-8 = У граней есть лицевая (белая) и обратная (серо-голубая) стороны; окраска группы красит внутри неё грани, у которых нет собственного материала.
## organizing
help-organizing-1 = Свойства объекта
help-organizing-2 = свойства выделения: площадь, длина, объём тел, тег, материал, имя, параметры компонентов и генераторов
help-organizing-3 = Теги
help-organizing-4 = видимость, цвет и имя тегов (слоёв); при удалении тега его содержимое переходит в «Без тега»
help-organizing-5 = Структура
help-organizing-6 = дерево групп и компонентов: Приблизить, Править, Скрыть, Разбить, Вынести из группы, Удалить; перетаскивайте строки, чтобы переносить объекты между группами
help-organizing-7 = Скрыть / Показать всё скрытое
help-organizing-8 = Окно → Сведения о документе
help-organizing-9 = статистика документа (грани, треугольники, компоненты, материалы, текстуры …) и «Удалить неиспользуемое»
## construction
help-construction-1 = Направляющие выделяются, перемещаются и удаляются как геометрия; Правка → Удалить направляющие убирает их все, Вид → Направляющие скрывает. Инструменты → Объёмный текст… создаёт текст в виде геометрии.
help-construction-2 = Вид → Секущие плоскости / Разрезы сечением / Заливка сечений управляют показом сечений; активный разрез сохраняется в сценах.
## styles
help-styles-1 = Меню «Вид» (и выпадающий список «Стиль» видового экрана)
help-styles-2 = С заливкой, С текстурами, Монохром
help-styles-3 = грани цветами материалов, с текстурами, одним цветом
help-styles-4 = Скрытые линии, Каркас
help-styles-5 = только рёбра (скрытые убраны или нет)
help-styles-6 = Реалистичный
help-styles-7 = физические материалы, освещение окружением, солнце, эффекты — карточка «Окружение» задаёт небо, HDRI, экспозицию и эффекты
help-styles-8 = Рентген, Рёбра, Контуры
help-styles-9 = видеть сквозь грани; рёбра и толстые контуры включаются и выключаются
help-styles-10 = Тени
help-styles-11 = тени от солнца; время и дата — в карточке «Тени»
help-styles-12 = Рендер
help-styles-13 = Окно → Рендер рисует вид трассировкой путей на GPU: стекло, отражения, источники света, мягкие тени; пресеты качества, размер кадра, глубина резкости, шумоподавление (если установлен Intel OIDN), цветокоррекция; PNG / EXR или сразу все сцены.
help-styles-14 = Источники света (точечный, прожектор, площадной) ставятся из окна «Рендер»; их выделяют щелчком по маркеру, перемещают и поворачивают как другие объекты.
help-styles-15 = Файл → Экспорт → 2D-графика (PNG) сохраняет вид как есть; Файл → Экспорт → Анимация… делает анимированный GIF или кадры PNG по сценам или облёт.
## scenes
help-scenes-1 = Карточка «Сцены»
help-scenes-2 = сохранённые виды со стилем, видимыми тегами и сечением; вкладки над видовым экраном переключают их
help-scenes-3 = Вид → Анимация
help-scenes-4 = Проиграть сцены; PgDn / PgUp — следующая / предыдущая сцена
help-scenes-5 = Окно → Листы
help-scenes-6 = страницы компоновки: видовые экраны сцен в масштабе 1:N, тексты, основная надпись, стандартные виды с размерами, таблица деталей; многостраничный PDF
help-scenes-7 = Файл → Экспорт → 2D-чертёж
help-scenes-8 = текущий вид как векторный SVG / PDF с контурами, скрытыми линиями, сечениями и размерами
help-scenes-9 = Файл → Экспорт → Отчёт
help-scenes-10 = детали с размерами для раскроя, площади по материалам (CSV)
## files
help-files-1 = Документы — файлы `.vrst` (текстуры и картинки сохраняются внутри). Файл → Создать из шаблона выбирает единицы; Окно → Настройки задаёт шаблон по умолчанию и интервал автосохранения.
help-files-2 = Окно → Настройки имеет разделы слева и поиск: Общие (автосохранение, шаблон для Файл → Создать, сколько шагов отмены хранится), Интерфейс (светлая, тёмная или системная тема, масштаб меню и панелей, размер значков панели инструментов), Рисование (сетка, привязки и шаги — те же переключатели, что в меню «Привязки»), Библиотеки (папки компонентов и материалов, Примеры, размер плиток), Загрузки (отслеживаемая папка), Навигация (мышь и трекпад), Сочетания клавиш (щёлкните сочетание и нажмите новые клавиши; Backspace убирает его, ↺ возвращает умолчание), Рендер (настройки рендера новых документов; где находится библиотека Open Image Denoise) и Вид (как выглядит новый документ: стиль, рёбра, оси, земля, небо, сетка, тени, проекция, поле зрения; «Взять текущий вид» / «Применить к текущему виду»). Изменения применяются сразу; «Сбросить» возвращает раздел к умолчаниям.
help-files-3 = Автосохранение записывает несохранённые изменения в `{ DATA_DIR() }/autosave`; после сбоя Varstotas предложит их восстановить.
help-files-4 = Импорт
help-files-5 = OBJ, STL, DXF, glTF / GLB, 3MF, COLLADA (DAE), FBX, PLY, точки из CSV и форматы, добавленные плагинами; файлы можно также перетащить на окно
help-files-6 = Экспорт
help-files-7 = OBJ, STL, DXF, glTF (GLB), 3MF, COLLADA (DAE), PLY, грани в JSON, PNG, чертежи SVG / PDF, листы, выкройки развёрток, анимация, отчёт, авторство
help-files-8 = Правка → Вырезать / Копировать / Вставить / Вставить на то же место
help-files-9 = «Вставить на то же место» сохраняет положение копии
help-files-10 = Файл → Изображение превью
help-files-11 = картинка, которую библиотека показывает для этого файла
## scripts
help-scripts-1 = Окно → Консоль скриптов выполняет скрипты Rhai над моделью: строить, менять, выделять, измерять. Файл → Выполнить скрипт… запускает файл `.rhai` (или `.py`). Запуск — один шаг отмены.
help-scripts-2 = Плагины — это скрипты в `{ DATA_DIR() }/plugins`: они становятся пунктами меню «Плагины», с диалогами для параметров и указания точек; они могут добавлять импортёры, экспортёры и источники материалов / моделей.
help-scripts-3 = Python и ИИ-агенты могут управлять Varstotas через локальный мост (клиент Python, сервер MCP).
help-scripts-4 = Открыть Справочник по скриптам
help-scripts-5 = Консоль и плагины
help-scripts-6 = Все функции
## shortcuts
help-shortcuts-1 = Клавиши инструментов работают, когда фокус не в текстовом поле. Буквы, введённые во время операции, попадают в поле значений.
## problems
help-problems-1 = Окно → Журнал
help-problems-2 = каждая ошибка и предупреждение сеанса — скрипты, импорт / экспорт, библиотеки, загрузки; красный счётчик в строке состояния означает новые ошибки
help-problems-3 = Правка → Отменить
help-problems-4 = любую операцию можно отменить; неудавшаяся операция ничего не меняет
help-problems-5 = Окно → Сведения о документе → Удалить неиспользуемое
help-problems-6 = убрать неиспользуемые компоненты, материалы и теги, чтобы файл стал меньше
help-problems-7 = Грани выглядят серо-голубыми
help-problems-8 = вы видите их обратную сторону: выделите их, правый щелчок → Обратить грани или Ориентировать грани
help-problems-9 = Нельзя рисовать там, где ожидается
help-problems-10 = проверьте фиксации Shift / стрелками, активную рабочую плоскость (Окно → Рабочие плоскости) и не находитесь ли вы внутри группы (Esc выходит из неё)
help-problems-11 = Чего-то не хватает
help-problems-12 = Правка → Показать всё скрытое, карточка «Теги» (скрытые теги), Вид → Секущие плоскости

## Окно, поиск, сочетания клавиш
help-window-title = Руководство
help-search = Поиск
help-search-hint = инструмент, команда или слово
help-nothing-found = Ничего не найдено
help-in-the-guide = В руководстве
help-tools = Инструменты
help-commands = Команды
help-use = Выбрать
help-use-hint = Выбрать этот инструмент
help-shortcuts-note = Меняются в Окно → Настройки → Сочетания клавиш. Мышь и трекпад: см. «Навигация по виду». Модификаторы выделения: см. «Выделение».
help-key-cut-copy-paste-keys = %CMDK%X / %CMDK%C / %CMDK%V
help-key-cut-copy-paste = Вырезать / Копировать / Вставить
help-key-delete = Delete, Backspace
help-key-delete-what = стереть выделение
help-key-esc = Esc
help-key-esc-what = отменить операцию; выйти из редактируемой группы
help-key-shift = Shift (удерживать)
help-key-shift-what = зафиксировать направление привязки
help-key-arrows = → ← ↑ ↓
help-key-arrows-what = зафиксировать красную / зелёную / синюю ось, параллельно последнему ребру
help-key-enter = Enter
help-key-enter-what = применить значение, введённое в поле значений
help-key-f1 = F1
help-key-f1-what = это руководство на активном инструменте; в редакторе скриптов — Справочник по скриптам

## ======== Скрипты ========

## Функции
script-fn-vec3 = Точка / вектор (подойдёт и массив [x, y, z]).
script-fn-line = Рёбра от a до b.
script-fn-polyline = Рёбра через точки; замкнутые и плоские → грань.
script-fn-face = Грань по точкам.
script-fn-rect = Прямоугольная грань в плоскости XY.
script-fn-circle = Грань-окружность (или рёбра).
script-fn-polygon = Грань — правильный многоугольник.
script-fn-arc = Рёбра дуги вокруг центра.
script-fn-box = Новая группа-параллелепипед (начало = угол с минимумом).
script-fn-cylinder = Новая группа-цилиндр (низ в центре).
script-fn-cone = Новая группа-конус (основание в центре, вершина на h выше).
script-fn-pyramid = Новая группа-пирамида: основание w × d от origin, вершина на h выше его середины.
script-fn-sphere = Новая группа-сфера вокруг центра.
script-fn-torus = Новая группа-тор вокруг центра: r до середины трубки, tube = её радиус.
script-fn-pushpull = Вытянуть / вдавить грань; возвращает верхнюю грань.
script-fn-offset = Сместить контур грани (d < 0 — внутрь); возвращает новую грань.
script-fn-follow-me = Протянуть грань-профиль вдоль пути.
script-fn-erase = Стереть объект или массив объектов.
script-fn-move = Переместить объекты на вектор.
script-fn-copy = Скопировать объекты на вектор; возвращает копии.
script-fn-rotate = Повернуть объекты вокруг оси.
script-fn-scale = Масштабировать объекты относительно точки.
script-fn-group = Сгруппировать объекты; возвращает группу.
script-fn-component = Создать компонент; возвращает его экземпляр.
script-fn-explode = Разбить группу / компонент.
script-fn-make-unique = Дать экземпляру компонента собственное определение.
script-fn-union = Объединение двух твёрдых групп.
script-fn-subtract = Твёрдое a минус b.
script-fn-intersect = Пересечение тел.
script-fn-trim = a минус b, b остаётся.
script-fn-split = Разделить два тела на три.
script-fn-outer-shell = Внешняя оболочка тел.
script-fn-reverse = Обратить грани.
script-fn-soften = Сгладить рёбра до угла излома.
script-fn-material = Создать / перекрасить материал; возвращает его имя.
script-fn-paint = Покрасить грани / группы (() снимает).
script-fn-tag = Назначить тег (создаётся, если нет).
script-fn-make-parametric = Сделать группу параметрической.
script-fn-set-params = Задать значения генерируемого компонента или размер / копии параметрической группы.
script-fn-params = Значения генерируемого компонента / параметры параметрической группы или ().
script-fn-reset-params = Генерируемый компонент обратно к умолчаниям генератора.
script-fn-generators = Имена генераторов документа.
script-fn-add-generator = Добавить / обновить генератор из исходника; возвращает его имя.
script-fn-generate = Поставить генерируемый компонент в точку.
script-fn-dimension = Линейный размер между точками.
script-fn-note = Текстовая заметка с выноской.
script-fn-enter = Строить внутри группы с этого момента.
script-fn-exit = Выйти из последней открытой группы.
script-fn-faces = Грани текущего контекста.
script-fn-edges = Рёбра текущего контекста.
script-fn-groups = Группы / компоненты текущего контекста.
script-fn-all = Всё в текущем контексте.
script-fn-entities = Всё внутри группы.
script-fn-child = Группа / компонент с именем name в текущем контексте (или внутри g); deep = искать и во вложенных группах. () если нет.
script-fn-children = Группы / компоненты текущего контекста (или внутри g); deep = и вложенные.
script-fn-show = Показать объекты (on = false: скрыть).
script-fn-hide = Скрыть объекты.
script-fn-selection = Выделение.
script-fn-select = Задать выделение (() очищает).
script-fn-bounds = #{ "{" } min, max, size, center { "}" } в мировых координатах.
script-fn-model-bounds = Границы всей модели.
script-fn-length = Разобрать длину с единицами ("1.2m") → мм.
script-fn-units = Единица длины модели.
script-fn-materials = Имена материалов.
script-fn-cut-list = Раскрой: детали (name, count, length, width, thickness, material, grain, exclude, bands), заготовки (material, kind, размеры, count, buy, отходы в %, cost, layouts с деталями), кромка по толщинам деталей, проблемы и общая стоимость.
script-fn-set-stock = Указать, в каком виде покупается материал: лист (length, width, thickness, grain, цена за лист) или доска (width, thickness, lengths в продаже, цена за метр или prices — за штуку, по одной на длину) или погонаж (kind "linear": name, необязательное сечение width × thickness, lengths и цены как у доски); список для нескольких толщин, сечений или форматов листа, () — без заготовок.
script-fn-cut-settings = Настройки раскроя: пропил, обрезка кромок, наименьший сохраняемый остаток, допуск по толщине, толщина кромочной ленты, её цена за метр и вычитать ли её из размера реза.
script-fn-cut-part = Настройки деталей с этим именем: grain ("length", "width" или "any"), exclude, bands (четыре true / false: длинные кромки, затем короткие), hardware (покупается, а не режется) и price (цена за штуку).
script-fn-cut-hardware = Фурнитура, которой нет в модели (винты, шканты…): список map с name, count и price, заменяет прежний.
script-fn-all-faces = Все грани, включая вложенные группы.
script-fn-read-text = Прочитать текстовый файл.
script-fn-lang = Код языка интерфейса ("en", "ru", …): выбирать тексты, которые плагин печатает или показывает.
script-fn-platform = "macos", "windows" или "linux".
script-fn-write-text = Записать текстовый файл.
script-fn-file-exists = Файл или папка существуют.
script-fn-list-files = Файлы в dir (и в подпапках) с расширением ("" = любое), по порядку.
script-fn-list-dirs = Папки в dir, по порядку.
script-fn-parse-json = Текст JSON → словари, массивы, числа, строки.
script-fn-fetch-text = Скачать веб-страницу / ответ API как текст (HTTP(S)); headers: #{ "{" } "X-Auth-Token": key { "}" }.
script-fn-fetch-file = Скачать url в файл path.
script-fn-url-encode = Закодировать текст для URL (поисковый запрос).
script-fn-print = Записать в журнал консоли.
script-fn-len = Длина массива / строки.
script-fn-range = Целые от from до to (to не включается), для циклов.
script-fn-min = Меньшее число.
script-fn-max = Большее число.
script-fn-clamp = x, удержанный в пределах lo..hi.
script-fn-ceil = Округлить вверх до целого (как float).
script-fn-noise = Гладкий шум 0..1 в точке x, y (одна ячейка = 1). При period > 0 повторяется каждые period ячеек (бесшовные плитки); seed выбирает узор.
script-fn-fbm = Фрактальный шум 0..1: октавы шума, каждая вдвое мельче и вдвое слабее; period и seed — как в noise.

## Разделы
script-group-points-and-vectors = Точки и векторы
script-group-drawing = Рисование
script-group-solids = Тела
script-group-editing = Правка
script-group-groups-and-components = Группы и компоненты
script-group-finding-objects = Поиск объектов
script-group-visibility-materials-tags = Видимость, материалы, теги
script-group-cut-list = Раскрой
script-group-solid-tools = Твердотельные операции
script-group-parameters-and-generators = Параметры и генераторы
script-group-dimensions-and-notes = Размеры и заметки
script-group-files-and-network = Файлы и сеть
script-group-output-and-numbers = Вывод и числа

## Члены объектов, векторов, массивов, строк
script-member-kind = "face", "edge", "group", "component" …
script-member-valid = объект ещё существует
script-member-area = площадь грани, мм²
script-member-normal = нормаль грани (мировая)
script-member-edges = рёбра грани
script-member-faces = грани ребра / соседи грани
script-member-length = длина ребра, мм
script-member-soft = ребро сглажено
script-member-center = центр границ (мировой)
script-member-vertices = точки грани / ребра (мировые)
script-member-name = имя группы / компонента (присваиваемое)
script-member-hidden = объект скрыт (присваиваемое)
script-member-bounds = #{ "{" } min, max, size, center { "}" } (мировые)
script-member-size = размер границ по x, y, z
script-member-min = минимальный угол границ (мировой)
script-member-max = максимальный угол границ (мировой)
script-member-definition = имя определения компонента
script-member-material = имя материала или ()
script-member-tag = имя тега или ()
script-member-x = координата x (присваиваемое)
script-member-y = координата y (присваиваемое)
script-member-z = координата z (присваиваемое)
script-member-length-2 = длина вектора
script-member-normalize = единичный вектор
script-member-dot = скалярное произведение
script-member-cross = векторное произведение
script-member-distance = расстояние до b
script-member-min-2 = минимальный угол
script-member-max-2 = максимальный угол
script-member-size-2 = размер по x, y, z
script-member-center-2 = центр
script-member-len = число элементов
script-member-is-empty = нет элементов
script-member-push = добавить элемент
script-member-pop = убрать и вернуть последний элемент
script-member-insert = вставить элемент
script-member-remove = убрать и вернуть элемент
script-member-contains = массив / строка содержит значение
script-member-index-of = индекс значения или -1
script-member-filter = элементы, для которых замыкание истинно
script-member-map = новый массив из результатов замыкания
script-member-reduce = свернуть элементы
script-member-len-2 = число символов
script-member-to-upper = верхний регистр (на месте)
script-member-to-lower = нижний регистр (на месте)
script-member-trim = убрать пробелы по краям (на месте)
script-member-split = разбить в массив
script-member-starts-with = строка начинается с text
script-member-to-string = как текст

## Теги заголовка плагинов
script-tag-name = имя в меню и в заголовке диалога
script-tag-menu = путь в меню «Плагины»
script-tag-description = подсказка и текст диалога
script-tag-param = значение, спрашиваемое в диалоге перед запуском
script-tag-pick = N точек, выбранных щелчками перед запуском → points
script-tag-select = объекты, выделенные перед запуском → objects
script-tag-import = пункт Файл → Импорт; файл в path
script-tag-export = пункт Файл → Экспорт; файл в path
script-tag-generator = генератор: делает этот файл генератором (см. docs/GENERATORS.md)
script-tag-formula = генератор: значение, вычисляемое из параметров, в p
script-tag-check = генератор: значения, для которых выражение ложно, отклоняются

## Типы параметров (@param key:type)
script-param-type-length = длина с единицами ("1.2m"), в скрипте мм — умолчание для чисел
script-param-type-int = целое число
script-param-type-number = дробное число
script-param-type-text = текст — умолчание для значения в кавычках
script-param-type-bool = флажок — умолчание для true / false
script-param-type-choice = генератор: один из choice(a|b|c)
script-param-type-angle = генератор: градусы, ползунок
script-param-type-percent = генератор: 0–100, ползунок
script-param-type-material = генератор: имя материала документа

script-type-face = грань
script-type-edge = ребро
script-type-group = группа
script-type-vector = вектор
script-type-num = число
script-type-bool = bool
script-type-str = строка
script-type-bounds = границы
script-type-faces = массив граней
script-type-edges = массив рёбер
script-type-groups = массив групп
script-type-vecs = массив точек
script-type-array = массив
script-type-any = объект / вектор

## Типы параметров без вступления «генератор: » (Справочник по скриптам, тема генераторов)
script-param-type-choice-plain = один из choice(a|b|c)
script-param-type-angle-plain = градусы, ползунок
script-param-type-percent-plain = 0–100, ползунок
script-param-type-material-plain = имя материала документа

# Ошибки, которые видит автор скрипта (API, генераторы, параметры компонентов)

## api.rs
script-err-k-expected-a-number-bool-or = { $k }: ожидалось число, bool или строка, получено { $type_name }
script-err-set-params-unknown-key-k-x = set_params: неизвестный ключ { $k } (x, y, z, count, axis, gap, stretch)
script-err-set-params-stretch-is-true-false = set_params: stretch — это true / false
script-err-cut-kind = Вид заготовки «{ $kind }»: используйте "sheet" или "board"
script-err-cut-lengths = Длины доски — это список чисел, получено { $t }
script-err-cut-size = У заготовки должны быть толщина, ширина и длина (лист) или длины (доска) больше нуля
script-err-cut-stock = set_stock: ожидается карта заготовки, их список или ()
script-err-cut-no-material = Нет материала «{ $material }»
script-err-cut-setting = Неизвестный параметр раскроя «{ $key }»
script-err-cut-grain = Волокна «{ $grain }»: используйте "length", "width" или "any"
script-err-cut-exclude = exclude — это true / false
script-err-cut-extra = каждая позиция — это map с name, count и price
script-err-cut-hardware = hardware — это true / false
script-err-cut-subtract = band_subtract — это true / false
script-err-cut-bands = bands — это четыре true / false: длинные кромки, затем короткие
script-err-set-params-not-parametric-call-make = set_params: не параметрический (сначала вызовите make_parametric)
script-err-paint-no-material-n-create-it = paint: нет материала { $n } (создайте его через material(name, colour))
script-err-paint-expected-a-material-name-got = paint: ожидалось имя материала, получено { $t }
script-err-show-hide-groups-and-components-only = show / hide: только группы и компоненты
script-err-outer-shell-solids-must-be-in = outer_shell: тела должны быть в одном контексте
script-err-group-entities-must-be-in-one = group: объекты должны быть в одном контексте
script-err-scale-zero-factor = scale: нулевой коэффициент
script-err-rotate-zero-axis = rotate: нулевая ось
script-err-follow-me-the-path-must-be = follow_me: путь должен быть в контексте грани
script-err-note-not-added = note: не добавлена
script-err-dimension-not-added = dimension: не добавлен
script-err-what-sizes-must-be-positive-a = { $what }: размеры должны быть положительными (трубка тора тоньше его радиуса), сегментов 3..1000
script-err-cylinder-no-face = cylinder: нет грани
script-err-cylinder-radius-and-height-must-be = cylinder: радиус и высота не должны быть нулевыми, сегментов 3..1000
script-err-box-sizes-must-be-non-zero = box: размеры не должны быть нулевыми
script-err-circle-radius-must-be-positive-segments = circle: радиус должен быть положительным, сегментов 3..1000
script-err-face-points-are-not-a-valid = face: точки не образуют допустимый плоский многоугольник
script-err-soft-not-an-edge = soft: не ребро
script-err-faces-not-an-edge-or-face = faces: не ребро и не грань
script-err-length-not-an-edge = length: не ребро
script-err-time-limit-of-secs-s-exceeded = превышен лимит времени { $secs } с
script-err-both-solids-must-be-in-the = оба тела должны быть в одном контексте
script-err-expected-a-colour-rrggbb-or-r = ожидался цвет: "#RRGGBB" или [r, g, b]
script-err-bad-colour-s-use-rrggbb = неверный цвет { $s } (используйте #RRGGBB)
script-err-bad-colour-s = неверный цвет { $s }
script-err-curve-not-added = кривая не добавлена
script-err-degenerate-curve = вырожденная кривая
script-err-expected-edges-got-e = ожидались рёбра, получено { $e }
script-err-expected-a-group-or-component-got = ожидалась группа или компонент, получено { $e }
script-err-expected-a-face-got-e = ожидалась грань, получено { $e }
script-err-entity-no-longer-exists-merged-split = объекта больше нет (слит, разрезан или стёрт)
script-err-context-no-longer-exists = контекста больше нет
script-err-expected-one-entity-got-len = ожидался один объект, получено { $len }
script-err-expected-an-entity-or-an-array = ожидался объект или массив объектов, получено { $type_name }
script-err-expected-an-entity-got-did-the = ожидался объект, получено () (операция не удалась?)
script-err-no-group-component-named-name-here = здесь нет группы / компонента с именем `{ $name }`
script-err-expected-an-array-of-points = ожидался массив точек
script-err-expected-a-point-vec-or-x = ожидалась точка (vec3 или [x, y, z]), получено { $type_name }
script-err-expected-a-number-got-type-name = ожидалось число, получено { $type_name }
script-err-not-a-length-s = не длина: { $s }
script-err-no-model = нет модели

## generator.rs
script-err-no-such-instance = нет такого экземпляра
script-err-not-a-generated-component = не генерируемый компонент
script-err-no-such-context = нет такого контекста
script-err-no-generator-name-in-the-document = в документе нет генератора `{ $name }`
script-err-name-generators-nested-too-deep = { $name }: генераторы вложены слишком глубоко
script-err-name-check-expr-is-not-true = { $name }: проверка `{ $expr }` не равна true / false
script-err-expected-a-number-got-t = ожидалось число, получено { $t }

## json.rs
script-err-bad-entity-id-s = неверный id объекта { $s }

## rig.rs
script-err-part-has-no-parameter-key = у `{ $part }` нет параметра `{ $key }`
script-err-part-has-no-parameters = у `{ $part }` нет параметров
script-err-no-part-part-in-the-component = в компоненте нет части `{ $part }`
script-err-part-no-parameter-k-it-has = { $part }: нет параметра `{ $k }` (есть { $keys })
script-err-part-the-part-has-no-parameters = { $part }: у части нет параметров (они есть только у генерируемых компонентов и компонентов с параметрами)
script-err-part-show-hide-needs-a-yes = { $part }: для показа / скрытия нужен параметр да / нет или число
script-err-part-choose-the-part-s-parameter = { $part }: выберите параметр части
script-err-part-the-direction-is-zero = { $part }: направление нулевое
script-err-part-the-axis-has-no-direction = { $part }: у оси нет направления
script-err-part-choose-a-parameter-or-write = { $part }: выберите параметр или напишите выражение
script-err-a-joint-without-a-part = сочленение без части
script-err-k-twice-parameters-and-formulas-share = `{ $k }` дважды (параметры и формулы используют общие имена)
script-err-no-such-component = нет такого компонента
script-err-this-component-is-built-by-a = этот компонент построен генератором или параметрической группой
script-err-groups-can-t-have-parameters-make = у групп не бывает параметров: сначала сделайте её компонентом
script-err-not-a-component-with-parameters = не компонент с параметрами
script-err-part-the-part-has-no-parameters-2 = { $part }: у части нет параметров
script-err-part-no-such-part = { $part }: нет такой части
script-err-what-the-value-is-not-a = { $what }: значение — не число, флаг и не текст
script-err-what-the-value-is-not-a-2 = { $what }: значение — не имя материала
script-err-what-the-value-is-not-true = { $what }: значение — не true / false
script-err-what-the-value-is-not-a-3 = { $what }: значение — не число

## ======== Сообщения ядра ========

## curves.rs
kernel-err-the-curve-bounds-faces-outside-its = кривая ограничивает грани вне своей плоскости
kernel-err-other-edges-are-attached-to-the = к кривой присоединены другие рёбра
kernel-err-too-few-points = слишком мало точек
kernel-err-segments-must-be-min-max-curve = сегментов должно быть { $min }..{ $MAX_CURVE_SEGMENTS }
kernel-err-radius-must-be-positive = радиус должен быть положительным
kernel-err-not-an-arc-circle-or-polygon = не дуга, не окружность и не многоугольник

## chamfer.rs
kernel-err-the-edges-at-a-corner-are = рёбра в углу лежат на одной прямой
kernel-err-a-corner-of-the-edge-is = угол ребра непростой (слишком много рёбер или граней)
kernel-err-the-chamfer-is-too-big-for = фаска слишком велика для ребра
kernel-err-the-chamfer-is-too-big-for-2 = фаска слишком велика для соседних рёбер
kernel-err-an-end-lies-along-the-edge = конец лежит вдоль ребра
kernel-err-the-radius-must-be-positive = радиус должен быть положительным
kernel-err-the-distance-must-be-positive = расстояние должно быть положительным
kernel-err-the-faces-are-coplanar = грани компланарны
kernel-err-broken-face-loop = разорванный контур грани
kernel-err-the-edge-is-too-short = ребро слишком короткое
kernel-err-the-edge-must-be-shared-by = ребро должно принадлежать ровно двум граням

## follow_me.rs
kernel-err-path-reverses = путь разворачивается
kernel-err-profile-is-parallel-to-the-path = профиль параллелен пути
kernel-err-path-is-not-connected = путь несвязный
kernel-err-path-branches = путь ветвится
kernel-err-empty-path = пустой путь

## mod.rs
kernel-err-nothing-to-flip = нечего отражать
kernel-err-flip-axis-is-zero = ось отражения нулевая

## push_pull.rs
kernel-err-through-hole-cap-vertex-missing = сквозное отверстие: нет вершины крышки
kernel-err-push-pull-would-collapse-geometry = «Тяни/Толкай» схлопнет геометрию
kernel-err-face-loop-without-edge = контур грани без ребра
kernel-err-distance-is-not-finite = расстояние не является конечным числом

## params.rs
kernel-err-empty-base = пустое основание
kernel-err-size-must-be-positive = размер должен быть положительным
kernel-err-not-parametric = не параметрический
kernel-err-empty-group = пустая группа
kernel-err-already-parametric = уже параметрический

## spline.rs
kernel-err-the-curve-would-cross-itself = кривая пересекла бы сама себя
kernel-err-segments-must-be-max-curve-segments = сегментов должно быть 1..{ $MAX_CURVE_SEGMENTS }
kernel-err-not-a-curve-that-can-be = это не кривая, которую можно править

## stats.rs
kernel-err-an-empty-name = пустое имя

## text3d.rs
kernel-err-no-outlines = нет контуров

## generator.rs
kernel-gen-name-no-parameter-k-join = { $name }: нет параметра `{ $k }` ({ $join })
kernel-gen-empty-generator-name = пустое имя @generator
kernel-gen-not-a-generator-the-header-needs = не генератор: в заголовке нужна строка `// @generator Name`
kernel-gen-key-twice-formulas-and-parameters-share = `{ $key }` дважды (формулы и параметры используют общие имена)
kernel-gen-key-bool-default-is-true-false = { $key }: умолчание bool — true / false
kernel-gen-unknown-parameter-type-t-length-int = неизвестный тип параметра `{ $t }` (length, int, number, bool, choice, angle, percent, material)
kernel-gen-key-choice-needs-options-choice-a = { $key }: для choice нужны варианты: choice(a|b|c)
kernel-gen-bad-parameter-name-key = неверное имя параметра `{ $key }`
kernel-gen-check-failed-expr = проверка не пройдена: { $expr }
kernel-gen-key-empty-formula = { $key }: пустая формула
kernel-gen-key-a-formula-is-length-int = { $key }: формула — это length, int, number, bool, angle, percent или text, но не `{ $t }`
kernel-gen-bad-formula-name-key = неверное имя формулы `{ $key }`
kernel-gen-key-bad-number-s-in-the = { $key }: неверное число `{ $s }` в диапазоне
kernel-gen-key-not-a-name-s = { $key }: не { $name }: { $s }
kernel-gen-key-s-is-not-one-of = { $key }: `{ $s }` не входит в { $join }

## model.rs
kernel-err-the-generator-s-script-can-t = скрипт генератора нельзя скопировать
kernel-err-nothing-selected = ничего не выделено

## solid.rs
kernel-err-pick-at-least-two-solids = выберите минимум два тела
kernel-err-the-second-solid-has-a-degenerate = у второго тела вырожденное преобразование
kernel-err-the-solids-do-not-overlap = тела не пересекаются
kernel-err-pick-two-different-solids = выберите два разных тела
# $name — имя группы / компонента
kernel-err-is-not-a-solid-every-edge = «{ $name }» не является телом (каждое ребро должно ограничивать ровно две грани, без вложенных групп)

## OpError
kernel-err-entity-not-found = объект не найден
kernel-err-degenerate-geometry = вырожденная геометрия
# $why — причина
kernel-err-operation-not-possible = операция невозможна: { $why }

## ======== Импорт и экспорт ========

## dae.rs
kernel-io-not-a-collada-document = не документ COLLADA
kernel-io-nodes-nested-too-deep = узлы вложены слишком глубоко

## dxf.rs
kernel-io-no-supported-entities-line-polyline-dface = нет поддерживаемых объектов (LINE, POLYLINE, 3DFACE, CIRCLE, ARC)
kernel-io-not-a-dxf-file-no-section = не файл DXF (нет SECTION)
kernel-io-not-a-dxf-file = не файл DXF

## fbx.rs
kernel-io-fbx-description = FBX: { $description }

## gltf.rs
kernel-io-index-out-of-range = индекс вне диапазона
kernel-io-no-meshes-in-the-gltf-scene = в сцене glTF нет мешей
kernel-io-node-hierarchy-too-deep = иерархия узлов слишком глубокая
kernel-io-draco-e = Draco: { $e }
kernel-io-draco-primitive-without-bufferview = примитив Draco без bufferView
kernel-io-accessor-out-of-range = accessor вне диапазона
kernel-io-unsupported-component-type = неподдерживаемый тип компонента
kernel-io-unsupported-accessor-type = неподдерживаемый тип accessor
kernel-io-accessor-without-count = accessor без count
kernel-io-bufferview-out-of-range = bufferView вне диапазона
kernel-io-bufferview-without-bytelength = bufferView без byteLength
kernel-io-bad-buffer-index = неверный индекс буфера
kernel-io-buffer-without-data = буфер без данных
kernel-io-compressed-gltf-e-is-not-supported = сжатый glTF ({ $e }) не поддерживается
kernel-io-only-gltf-is-supported = поддерживается только glTF 2.0
kernel-io-external-file-uri-needs-a-path = внешнему файлу { $uri } нужен путь
kernel-io-bad-base-in-data-uri = неверный base64 в data URI
kernel-io-not-a-gltf-file-e = не файл glTF: { $e }
kernel-io-glb-without-json = GLB без JSON
kernel-io-glb-json-e = JSON в GLB: { $e }
kernel-io-truncated-glb-chunk = обрезанный фрагмент GLB

## meshes.rs
kernel-io-no-faces-in-the-file = в файле нет граней

## native.rs
kernel-io-unsupported-version-version = неподдерживаемая версия { $version }
kernel-io-not-a-varstotas-material-format-format = не материал Varstotas (format = { $format })
kernel-io-missing-root-definition = нет корневого определения
kernel-io-not-a-varstotas-file-format-format = не файл Varstotas (format = { $format })

## obj.rs
kernel-io-obj-line-ln-line = строка OBJ { $ln }: { $line }

## ply.rs
kernel-io-no-faces-a-point-cloud-can = нет граней (облако точек импортировать нельзя)
kernel-io-list-too-long = список слишком длинный
kernel-io-unknown-format-f = неизвестный формат { $f }
kernel-io-property-before-element = property до element
kernel-io-bad-element-count = неверное число элементов
kernel-io-no-end-header = нет end_header
kernel-io-not-a-ply-file = не файл PLY
kernel-io-truncated-data = обрезанные данные
kernel-io-unknown-type-s = неизвестный тип { $s }

## points.rs
kernel-io-no-points-in-display = в { $display } нет точек

## stl.rs
kernel-io-stl-vertex-count-not-a-multiple = число вершин STL не кратно 3
kernel-io-bad-stl-vertex-line = неверная вершина STL: { $line }
kernel-io-not-a-valid-stl = недопустимый STL

## threemf.rs
kernel-io-mf-no-d-model-part = 3MF: нет части 3D-модели
kernel-io-mf-no-object-id = 3MF: нет объекта { $id }
kernel-io-mf-components-nested-too-deep = 3MF: компоненты вложены слишком глубоко
kernel-io-mf-no-part-path = 3MF: нет части { $path }

## xml.rs
kernel-io-xml-no-root-element = XML: нет корневого элемента
kernel-io-xml-unbalanced-tags = XML: несбалансированные теги
kernel-io-xml-e = XML: { $e }

## zip.rs
kernel-io-name-zip-method-m-not-supported = { $name }: метод zip { $m } не поддерживается
kernel-io-truncated-zip = обрезанный zip
kernel-io-bad-zip-entry-name = неверная запись zip { $name }
kernel-io-bad-zip-directory = неверный каталог zip

## Префиксы читателей COLLADA / PLY
# $msg — собственное сообщение читателя
kernel-io-collada = COLLADA: { $msg }
kernel-io-ply = PLY: { $msg }

## IoError
# $why — системная ошибка / сообщение читателя
kernel-io-error-io = io: { $why }
kernel-io-error-format = формат: { $why }

## File templates
template-files-heading = Из файлов шаблонов
template-files-empty = В этой папке нет шаблонов .vrst
template-open-folder = Открыть папку шаблонов…
template-folder-error = Не удалось прочитать шаблоны: { $error }
template-created = Создано из шаблона: { $path }
template-load-failed = Не удалось загрузить шаблон { $path }: { $error }
template-folder-label = Папка шаблонов
template-folder-tip = Поместите сюда модели .vrst для создания новых документов на их основе.

app-export-replace-title = Заменить файлы экспорта?
app-export-replace-files =
    Экспорт заменит следующие файлы материалов или текстур:
    { $files }
    Продолжить?
app-export-folder = Выберите папку для { $file } и его файлов (материалы, текстуры)
app-export-replace-all =
    Экспорт заменит следующие файлы:
    { $files }
    Продолжить?
app-import-missing-title = Не найдены файлы рядом с моделью
app-import-missing =
    { $file } ссылается на файлы рядом с собой, которые Varstotas не смог прочитать:
    { $files }
    Выбрать папку, в которой лежит модель, чтобы загрузить их? (Приложение в песочнице, например Flatpak, видит только файл, выбранный в диалоге.)
app-import-not-in-folder = { $file } нет в папке { $folder }

## Object and tag locking
cmd-lock = Заблокировать
cmd-unlock = Разблокировать
cmd-unlock-all = Разблокировать всё
cmd-desc-lock = Запретить выделение и редактирование выбранных объектов
cmd-desc-unlock-all = Разблокировать все объекты и теги
edit-locked = Операция изменяет заблокированные объекты. Сначала разблокируйте их.
script-member-locked = Блокировка редактирования с учётом тега и родительских объектов (только чтение).

## Стартовый экран и обратная связь
start-menu = Стартовый экран
start-subtitle = Создайте модель, продолжите работу или посмотрите примеры.
start-create = Новая модель
start-new = Создать
start-open = Открыть документ…
start-guide = Краткое руководство
start-recent = Последние документы
start-no-recent = Здесь появятся недавно открытые документы.
start-basics-title = Первые шаги
start-basics = Пробел: выделение · L: линия · R: прямоугольник. Вводите точные размеры в поле измерений. Колесо мыши — масштаб, средняя кнопка — вращение вида.
start-show = Показывать при запуске
start-continue = Продолжить
report-menu = Сообщить о проблеме…
report-intro = Проверьте сообщение и откройте черновик в браузере. Для отправки нужно войти в GitHub и отправить его самостоятельно.
report-access = Для закрытого репозитория нужен доступ. Файлы модели и снимки экрана автоматически не прикладываются.
report-title = Краткое описание
report-description = Что произошло?
report-description-hint = Шаги воспроизведения, ожидаемый результат и фактическое поведение.
report-errors = Включить последние 10 предупреждений и ошибок
report-review = Перед отправкой проверьте текст ниже. В журнале могут быть имена файлов и другие сведения.
report-copy = Копировать сообщение
report-github = Открыть черновик на GitHub
report-paste = Полный текст скопирован. Вставьте его в описание issue на GitHub.

cli-usage =
    Использование: Varstotas [--open] <document.vrst>
    Varstotas -- <document.vrst>
    Varstotas --help | --version
cli-one-document = Укажите один документ
cli-unknown-option = Неизвестный параметр: { $option }
cli-open-path = После --open нужно указать путь к документу

prefs-save-backups = Сохранять резервную копию предыдущей версии
prefs-save-backups-note = При сохранении остаётся предыдущая версия. В папке приложения: { DATA_DIR() }/backups/<name> — <его папка> (…).bak.vrst, так что одноимённые документы из разных папок не перезаписывают копии друг друга. Рядом с документом: <name>.vrst.bak (в папку приложения, если в ту папку писать нельзя). Отключение не удаляет существующие копии.
prefs-backups-data = В папке приложения
prefs-backups-beside = Рядом с документом (<name>.vrst.bak)

model-info-metadata = Метаданные документа
model-info-metadata-text = Текст / описание
model-info-license = Лицензия
model-info-license-hint = Название лицензии, ссылка или полный текст (необязательно)
model-info-metadata-apply = Применить
model-info-metadata-reset = Сбросить изменения
model-info-format-version = Версия формата сохранения: { $version }
model-info-metadata-edit = Изменить метаданные документа

model-info-tab-info = Инфо
model-info-tab-meta = Мета
model-info-tab-cleanup = Очистка

download-progress-unknown = Скачано { $received } / общий размер неизвестен

# Practical step-by-step walkthrough.
tutorial-title = Туториал
tutorial-step = Шаг { $current } / { $total }
tutorial-try = Попробуйте сами
tutorial-tools = Включить инструмент
tutorial-details = Подробнее об этом шаге
tutorial-practice-note = Оставьте окно рядом с моделью и выполняйте задания в рабочей области.
tutorial-back = Назад
tutorial-restart = С начала
tutorial-finish = Завершить
tutorial-next = Далее
tutorial-0-title = Знакомство с Varstotas
tutorial-0-body = Модель строится из рёбер и граней, которые можно объединять в группы и повторно используемые компоненты. Здесь есть точное черчение, редактируемые 3D-формы, материалы, сцены и листы чертежей. Пройдём путь от нового документа до модели для обмена.
tutorial-0-task = Выберите File → New. На примере простой столешницы освоим инструменты и подготовим файл к передаче. Переходите дальше в удобном темпе.
tutorial-1-title = Осмотритесь вокруг модели
tutorial-1-body = Перетаскивание средней кнопкой мыши вращает вид, с Shift — перемещает камеру. Колесо приближает точку под курсором. Управление трекпадом и скорость меняются в настройках навигации. Виды и проекция доступны и в углу рабочей области.
tutorial-1-task = Попробуйте вращение и панорамирование. Переключите Top и Iso в Camera → Standard Views, затем впишите модель в окно командой Zoom Extents.
tutorial-2-title = Нарисуйте точное основание
tutorial-2-body = Замкнутый плоский контур из рёбер образует грань. Прямоугольник и окружность создают её сразу. Поле Measurements принимает точные размеры во время работы инструмента; подсказки и Подсказки объясняют текущий этап.
tutorial-2-task = Включите Rectangle. Укажите первый угол, сдвиньте указатель, введите в Measurements 1400mm,800mm и нажмите Enter. Получится контур столешницы.
tutorial-3-title = Добавьте объём
tutorial-3-body = Push/Pull превращает грань в объём. Draw → Shapes создаёт бокс, цилиндр, конус, пирамиду, сферу или тор за одну операцию. Эти формы — компоненты с изменяемыми параметрами генератора в Свойства.
tutorial-3-task = Вытяните прямоугольник инструментом Push/Pull и задайте толщину 40. Рядом попробуйте Box: первый угол, противоположный угол основания, затем высота.
tutorial-4-title = Используйте привязки
tutorial-4-body = Курсор отмечает концы, середины, оси и узлы сетки. Цветные линии помогают выдержать направление. Стрелки фиксируют ось, Shift — текущую привязку. В меню магнита можно включать отдельные типы привязок.
tutorial-4-task = Включите сетку и Line. Наведитесь на угол, затем ведите вдоль оси к следующей линии сетки. Повторите с Box: привязки работают на основании и высоте. Escape отменяет незавершённую операцию.
tutorial-5-title = Редактируйте и используйте повторно
tutorial-5-body = Move, Rotate и Scale изменяют выделение, а Undo помогает экспериментировать. Группа отделяет геометрию от соседних рёбер. Компоненты используют общее определение: редактирование одной копии обновляет остальные.
tutorial-5-task = Выделите столешницу и создайте группу через Edit. Передвиньте её и отмените перемещение. Двойным щелчком войдите внутрь группы; перед следующим шагом завершите редактирование контекста.
tutorial-6-title = Наведите порядок
tutorial-6-body = Outliner показывает вложенные группы и компоненты. Теги управляют видимостью и блокировкой редактирования. Свойства показывает выделение и его параметры. Скрытие и блокировка работают независимо.
tutorial-6-task = Назовите столешницу в Свойства. Создайте тег в Tags, назначьте его и переключите видимость. Попробуйте замок, затем разблокируйте объект перед покраской.
tutorial-7-title = Материалы и режимы отображения
tutorial-7-body = Paint назначает цвет и текстуру, Texture Position меняет их размещение. Online Materials загружает материалы в библиотеку. Shaded with Textures удобен при моделировании; Monochrome различает стороны граней, Realistic использует физические свойства и освещение.
tutorial-7-task = Выберите материал и покрасьте грань. Сравните Shaded with Textures, Monochrome и Realistic в меню стиля. Включите X-Ray, чтобы увидеть скрытые рёбра.
tutorial-8-title = Покажите результат
tutorial-8-body = Сцены запоминают виды и параметры отображения. Размеры подписывают модель, секущие плоскости показывают внутреннее устройство, а листы располагают виды на бумаге. Render создаёт итоговое изображение, а анимация переходит между сценами.
tutorial-8-task = Впишите модель в Iso и добавьте сцену. Создайте вторую сцену с видом Top. Переключитесь между ними, затем попробуйте Dimension или Section.
tutorial-9-title = Сохраните и поделитесь
tutorial-9-body = Файл .vrst хранит редактируемую геометрию, материалы, сцены и метаданные. В Сведения о документе → Meta задаются описание, лицензия и превью. Импорт и экспорт поддерживают OBJ, STL, GLB и другие форматы; скрипты и плагины расширяют инструменты.
tutorial-9-task = Сохраните модель. В Сведения о документе → Meta добавьте описание и лицензию, затем сохраните ещё раз. В Help → Examples доступны примеры; к любому шагу можно вернуться для практики.

app-reversed-faces = Перевёрнуто граней: { $count }. Режим Monochrome показывает лицевые и обратные стороны.

download-progress-previews = Загрузка превью: { $count }

prefs-trackpad-enabled = Навигация тачпадом / плавной прокруткой
prefs-gesture-speed = Чувствительность
prefs-invert-swipe = Инвертировать движение двух пальцев
prefs-pinch-enabled = Масштабирование щипком
prefs-invert-pinch = Инвертировать щипок
prefs-twist = Поворот двумя пальцами
prefs-twist-off = Выключен
prefs-twist-orbit = Вращать вокруг вертикали
prefs-twist-roll = Наклонять вид
prefs-invert-twist = Инвертировать поворот
prefs-trackpad-platform = Поддержка щипка и поворота зависит от ОС и драйвера. Эти настройки также действуют на устройства с плавной прокруткой.

view-triangles = Треугольники
view-triangles-hint = Показать все рёбра триангуляции поверх текущего стиля, включая мягкие рёбра и диагонали полигонов.

# 3D Printing: basic surface checks and conservative repair.
menu-3d-printing = 3D Printing
print-title = Подготовка к 3D-печати
print-check-command = Проверить модель…
print-repair-command = Ремонт сетки…
print-check-description = Проверить выбранные объекты или видимую модель на основные дефекты сетки для печати.
print-repair-description = Исправить швы, дубли, ориентацию и небольшие плоские отверстия.
print-scope = Область:
print-visible = Видимая модель
print-selection = Выделение
print-tab-check = Проверка
print-tab-repair = Ремонт
print-working = Обрабатывается копия модели…
print-worker-failed = Не удалось завершить проверку. Запустите её снова.
print-result-stale = Модель или выделение изменились. Ремонт не применён; повторите проверку.
print-repair-undo = Ремонт для 3D-печати
print-stale = Отчёт устарел. Запустите проверку снова.
print-repair-weld = Соединить совпадающие вершины
print-repair-degenerate = Удалить вырожденные грани
print-repair-duplicates = Удалить дублирующиеся грани
print-repair-orient = Согласовать ориентацию граней
print-repair-shells = Развернуть замкнутые оболочки с учётом полостей
print-repair-holes = Закрыть небольшие плоские отверстия
print-hole-size = Максимальный размер отверстия:
print-repair-note = Ремонт можно отменить. Заблокированная, скрытая и процедурная геометрия сохраняется. Сложные и вложенные контуры отверстий остаются для ручного ремонта.
print-repair-whole-objects = Для ремонта выберите группы/компоненты целиком или переключитесь на «Видимая модель».
print-repair-run = Исправить и проверить
print-repaired = Соединено вершин: { $welded }; удалено граней: { $removed }; развёрнуто граней: { $reversed }; закрыто отверстий: { $holes }; пропущено объектов: { $skipped }.
print-empty = В этой области нет видимых граней. Выберите объект или «Видимая модель».
print-counts = Граней: { $faces } · треугольников: { $triangles } · оболочек: { $shells } (замкнутых: { $closed })
print-size = Размер: { $x } × { $y } × { $z } мм
print-area = Площадь поверхности: { $area } мм²
print-volume = Замкнутый объём: { $volume } мм³
print-found-problems = Обнаружены проблемы поверхности
print-basic-pass = Базовая проверка поверхности пройдена
print-limitations = Анализ толщины выполняется по пробам. Полный анализ, нависания и ограничения принтера доступны на соседних вкладках. Проверьте слои в слайсере.
print-tolerance = Допуск совпадения вершин:
print-highlight = Подсвечивать проблемы в модели
print-check-run = Проверить
print-copy = Копировать отчёт
print-open-edges = Открытые граничные рёбра
print-nonmanifold = Рёбра с более чем двумя гранями
print-winding = Несогласованная ориентация рёбер
print-degenerate = Вырожденные треугольники / пустые грани
print-duplicates = Дублирующиеся треугольники
print-inverted = Вывернутые замкнутые оболочки
print-wire = Отдельные рёбра (не печатаются)
print-export-visible = Экспорт видимой модели

# Printing preparation tools
print-analysis = Анализ
print-bed = Принтер и стол
print-split = Разрезание и соединители
print-export = Экспорт для печати
print-export-stale = Модель или настройки изменились. Повторите экспорт перед открытием слайсера.
print-cut-undo = Разрезание для печати
print-place-undo = Размещение на печатном столе
print-placement-done = Размещение обновлено. Обновите проверку.
print-cut-select = Для разрезания выберите одну группу или компонент целиком.
print-cut-done = Создано частей: { $parts }, включая отдельные штифты: { $dowels }. Разместите их на столе.
print-export-extension = Выберите .3mf или .stl.
print-export-done = Экспортировано частей: { $parts } в { $path }
print-slicer-opened = Экспортированный файл открыт в приложении по умолчанию для него.
print-min-wall = Минимальная толщина стенки
print-two-nozzles = Два диаметра сопла
print-sample-budget = Максимум проб толщины
print-pair-budget = Максимум пар для проверки пересечений
print-overhang-angle = Угол нависания от вертикали вниз
print-map = Карта поверхности
print-map-none = Нет
print-map-wall = Толщина стенок
print-map-overhang = Нависания
print-intersection-count = Пересекающихся пар треугольников: { $n }; проверено пар: { $pairs }
print-intersection-incomplete = Достигнут лимит пар: проверка пересечений не завершена.
print-wall-coverage = Проб толщины: { $samples } / { $possible }; треугольников: { $triangles } / { $total }
print-wall-result = Минимум по пробам: { $minimum } мм; тонких: { $thin }; неопределённых: { $unknown }
print-overhang-result = Площадь нависаний: { $area } мм² (треугольников: { $n })
print-wall-unreliable = Открытые, пересекающиеся или неверно ориентированные поверхности снижают достоверность толщины.
print-wall-estimate = Толщина оценивается лучами внутрь, до четырёх проб на треугольник. Мелкие детали между пробами могут быть пропущены. Серый — выход не найден, красный — ниже порога. Контакт со столом исключён из нависаний. Проверьте слои в слайсере.
print-profile-name = Профиль принтера
print-bed-width = Ширина стола X
print-bed-depth = Глубина стола Y
print-bed-height = Высота печати Z
print-bed-margin = Отступ от края стола
print-bed-gap = Зазор между деталями
print-nozzle = Диаметр сопла
print-arrange-rotate = Разрешить поворот на 90° при размещении
print-bed-show = Показывать объём печати в сцене
print-drop = Опустить на стол
print-center = По центру стола
print-arrange = Разложить детали
print-lay-face = Положить выбранной гранью
print-placement-note = Размещение сохраняет масштаб и работает с целыми деталями. Укладка использует габариты: сложные формы иногда нужно разместить вручную. Предупреждения о перекрытии относятся к габаритам.
print-bed-result = Деталей: { $parts }; за пределами: { $outside }; вне плоскости стола: { $floating }; перекрытий габаритов: { $overlaps }
print-fits = Помещается
print-outside = За пределами объёма печати
print-cut-normal = Нормаль плоскости разреза
print-cut-offset = Смещение плоскости от начала координат
print-connectors = Добавить гнёзда и отдельные штифты
print-dowel-count = Количество штифтов
print-dowel-diameter = Диаметр штифта
print-dowel-length = Длина штифта
print-dowel-clearance = Радиальный и осевой зазор
print-dowel-wall = Минимальная стенка вокруг гнезда
print-dowel-note = Разрезание требует замкнутых деталей с согласованной ориентацией. Гнёзда распределяются вдоль разреза с проверкой окружающего материала. Половины и отдельные штифты остаются в положении сборки: затем разложите их. Подберите зазор для своего принтера.
print-cut-run = Разрезать и закрыть сечения
print-cut-to-bed = Разрезать по размерам принтера
print-cut-to-bed-note = Автоматические разрезы учитывают полезный объём X/Y/Z, до 256 деталей. Части закрываются без соединителей и сохраняют положение сборки.
print-export-scope-note = Экспорт использует выбранную область и сохраняет мировые координаты в миллиметрах. 3MF сохраняет отдельные объекты; STL содержит треугольники.
print-export-materials-note = 3MF включает базовые цвета, имена, описание, лицензию и источники. Текстуры PBR не экспортируются. STL содержит только геометрию.
print-export-3mf = Экспорт 3MF…
print-export-stl = Экспорт STL…
print-slicer-open = Открыть в слайсере
print-slicer-note = Экспорт открывается в приложении, назначенном в системе для .3mf / .stl: сделайте слайсер приложением по умолчанию для этих файлов. После изменений модели повторите экспорт.
print-export-missing = Экспортированный файл отсутствует. Повторите экспорт.

# Printing operation errors
print-error-1 = Некорректное преобразование родителя
print-error-2 = В этой области нет видимых граней для печати
print-error-3 = Некорректные размеры принтера или отступ
print-error-4 = Заблокированные и процедурные объекты нельзя изменять
print-error-5 = Деталь содержит заблокированную геометрию
print-error-6 = Перед разрезанием покажите всю геометрию детали
print-error-7 = Деталь содержит заблокированные рёбра
print-error-8 = Деталь содержит заблокированные объекты
print-error-9 = Выберите детали целиком
print-error-10 = Для размещения и разрезания выберите группы или компоненты целиком
print-error-11 = Сгруппируйте свободную геометрию перед размещением, если другие корневые грани скрыты
print-error-12 = Нет видимых деталей
print-error-13 = Деталь не содержит поверхности
print-error-14 = Необратимое преобразование родителя
print-error-15 = Некорректные размеры принтера
print-error-16 = Нет деталей
print-error-17 = Детали не помещаются на одном столе в исходном масштабе: выберите меньше деталей или разрежьте их
print-error-18 = Вырожденная грань
print-error-19 = Выбранная грань утоплена: при таком размещении часть геометрии окажется под столом
print-error-20 = Неоднозначная граница сечения: исправьте деталь перед разрезанием
print-error-21 = Открытая граница сечения
print-error-22 = Некорректная граница сечения
print-error-23 = Для отверстия в сечении не найден внешний контур
print-error-24 = После закрытия сечения поверхность не замкнута: сначала исправьте исходную сетку
print-error-25 = Плоскость не разделяет эту деталь
print-error-26 = Некорректная плоскость разреза
print-error-27 = Для разрезания выберите ровно одну деталь целиком
print-error-28 = Перед разрезанием исправьте открытые, дублирующиеся и неверно ориентированные поверхности
print-error-29 = Разрезание требует полной проверки пересечений без обнаруженных пересекающихся поверхностей
print-error-30 = Выберите одну деталь целиком
print-error-31 = Потребуется более 256 частей: увеличьте профиль принтера или разрезайте поэтапно
print-error-32 = Некорректные размеры, количество штифтов или зазор
print-error-33 = Не найдено сечение для размещения соединителей
print-error-34 = Соединители слишком близко: уменьшите количество или диаметр
print-error-35 = Соединитель выходит за пределы детали или не хватает толщины стенки/глубины. Уменьшите размеры или количество, переместите плоскость либо выберите более толстую область
print-error-36 = Некорректное гнездо
print-error-37 = Не удалось вычесть гнездо
print-error-38 = Некорректный штифт
print-bed-frame = Показать весь объём печати
print-part-name = Деталь для печати
print-socket-name = Вычитание гнезда
print-dowel-name = Сборочный штифт

print-go-check = Перейти к проверке
print-analysis-run = Запустить анализ
print-bed-refresh = Обновить проверку стола

# Window → Physics
physics-title = Физика
physics-drop = Падение
physics-drop-hint = Выделенные группы и компоненты падают и успокаиваются на всём остальном, что видно.
physics-shape = Форма
physics-shape-hull = Оболочка
physics-shape-hull-hint = Выпуклая оболочка каждого объекта: быстро и устойчиво, впадины заполняются.
physics-shape-pieces = Части
physics-shape-pieces-hint = Выпуклые части каждого объекта: в чашу можно что-то положить, подготовка дольше.
physics-friction = Трение
physics-bounce = Упругость
physics-gravity = Тяжесть
physics-time = Предел времени
physics-floor = Пол на Z = 0
physics-no-bodies = Выделите группы или компоненты, которые должны упасть.
physics-bodies = Объектов для падения: { $n }
physics-run = Запустить
physics-running = Симуляция… { $time } с
physics-done = Успокоились за { $time } с
physics-stopped = Остановлено на { $time } с
physics-fell = Объектов без опоры: { $n } (они остаются на месте)
physics-apply = Применить
physics-again = Запустить снова
physics-stop = Стоп
physics-cancel = Отмена
physics-interrupted = Модель изменилась: симуляция остановлена.
physics-failed = Симуляция не удалась: { $error }
physics-undo = Падение
physics-applied = Перемещено объектов: { $n }
physics-cloth = Ткань
physics-cloth-hint = Выделенная грань (или группа из одной грани) падает как ткань и облегает всё остальное, что видно. Она становится группой гладких треугольников.
physics-cloth-no-face = Выделите одну грань или группу из одной грани.
physics-cloth-too-small = Грань меньше шага.
physics-cloth-too-fine = Точек: { $n } — увеличьте шаг.
physics-cloth-particles = Точек ткани: { $n }
physics-cloth-spacing = Шаг
physics-cloth-spacing-hint = Расстояние между точками ткани: чем меньше, тем мельче складки и дольше расчёт.
physics-cloth-stiffness = Жёсткость
physics-cloth-stiffness-hint = Насколько ткань сопротивляется растяжению.
physics-cloth-bending = Изгиб
physics-cloth-bending-hint = Насколько ткань сопротивляется сгибу: мало — шёлк, много — кожа.
physics-cloth-thickness = Толщина
physics-cloth-thickness-hint = Зазор между тканью и тем, на чём она лежит (не меньше десятой доли шага).
physics-cloth-self = Не проходит сквозь себя
physics-cloth-self-hint = Складки сталкиваются друг с другом (медленнее).
physics-cloth-undo = Ткань
physics-cloth-applied = Ткань стала новой группой.
