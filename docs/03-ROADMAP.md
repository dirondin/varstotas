# Роадмап

## M0 — Каркас (оркестратор) ✅
- workspace, крейты, контракты API, документы.

## M1 — Фундамент (параллельно, 3 потока) ✅
- **Kernel:** Mesh с липкой геометрией (add_line, авто-грани, разбиение рёбер/граней), erase, push_pull, transform_vertices, tessellate, History, save/load, OBJ/STL export, units parser. Тесты.
- **Render:** Camera, Renderer (offscreen MSAA, грани с освещением, front/back цвета, рёбра, толстые/пунктирные оверлеи, сетка, оси, небо/земля, X-ray, wireframe).
- **App shell:** окно, layout в классическом стиле, навигация камерой, тулбар, статус-бар, VCB-виджет, меню файлов, панели (заглушки с реальными данными модели), горячие клавиши.

**Демо M1:** открыть программу, покрутить сцену с тестовым кубом, сохранить/открыть файл.

## M2 — Рисование (2 потока) ✅
- **Tools & Inference:** Select (клик/двойной/тройной/рамки), Line, Rectangle, Circle, Polygon, Arc, Eraser, Push/Pull, Move, Rotate, Offset, Tape Measure + направляющие, Paint Bucket. Inference engine P0. VCB для всех.
- **Kernel:** offset_face, arcs/curves (рёбра, объединённые в кривую), groups (make/explode/edit context), материалы.

**Демо M2:** нарисовать дом из `00-VISION.md`.

## M3 — Полировка MVP 🟨 (QA-раунд 1 закрыт: docs/qa/M3-report.md; осталось: релизные сборки mac universal / win, 2-й QA-раунд, производительность)
- Groups edit-контекст, Entity Info, Materials, стили, Zoom Extents, стандартные виды, undo всего, автосохранение.
- QA-проход по UX, исправление багов топологии, производительность (10k+ граней @ 60 FPS).
- Сборка релизов: macOS `.app` (universal), Windows `.exe`.

## M4 — Вторая волна
- Components + Outliner, Tags, Scenes, Follow Me, Scale, Rotated Rectangle, 3-point arc, Protractor, Axes, Dimensions, Intersect Faces, текстуры, импорт OBJ/STL, экспорт PNG.

## M5 — Дальше
- Solid tools, Section planes, тени, Text/3D Text, DXF, Soften edges, Walk.
