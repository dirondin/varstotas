# Архитектура

## Стек
| Слой | Выбор | Почему |
|---|---|---|
| Язык | Rust (stable, edition 2024) | скорость C++, безопасность, кросс-компиляция mac/win одной командой |
| Окно + UI | `eframe`/`egui` 0.36 (wgpu-бэкенд) | нативное окно, immediate-mode UI, крошечный бинарь, без Qt/Electron |
| GPU | `wgpu` (реэкспорт `egui_wgpu::wgpu` — **только он**, чтобы версии совпадали) | Metal на macOS, DX12/Vulkan на Windows |
| Математика | `glam` (f64 — `DVec3/DMat4` в ядре, f32 — на GPU) | |
| ID сущностей | `slotmap` | стабильные generational-ключи |
| Сериализация | `serde` + `serde_json` (`.vrst`) | читаемо, легко мигрировать |
| Файловые диалоги | `rfd` | нативные диалоги |

Единицы внутри ядра — **миллиметры, f64**. Допуск слияния точек `TOL = 1e-3` мм.
Оси: X — красная, Y — зелёная, **Z — синяя (вверх)**, правая система.

## Крейты (workspace)
```
crates/
  varstotas_kernel/   # геометрическое ядро + документ. Без GPU, без UI. 100% unit-тестируемо.
  varstotas_render/   # wgpu-рендерер + Camera. Не знает о ядре: принимает плоские буферы.
  varstotas_app/      # бинарь: окно, UI-панели, инструменты, inference, VCB, ввод.
```
Зависимости: `app → kernel`, `app → render`. `kernel` и `render` друг от друга **не зависят**.

## varstotas_kernel
- `Model` — документ: дерево `Definition` (корень + группы/компоненты), материалы, теги, сцены, настройки единиц.
- `Mesh` — «липкая» геометрия одного контекста: вершины, рёбра, грани (внешний контур + дыры), экземпляры дочерних определений.
  - `add_line` сливает вершины, режет рёбра в точках пересечения, режет грани, автоматически создаёт грани на замкнутых плоских контурах.
  - `push_pull`, `offset_face`, `follow_me`, `transform_vertices` (растяжение связной геометрии), `erase`, `reverse_face`, `intersect`.
- Группа = `Definition` с одним `Instance`; компонент = `Definition` с N экземплярами.
- `History` — undo/redo на снимках `Model` (клон). Просто и надёжно; оптимизация — позже.
- `tessellate` — треугуляция граней (с дырами) + список рёбер с ссылками на сущности для пикинга.
- `io` — `.vrst` (JSON), экспорт OBJ/STL, импорт OBJ/STL.
- `units` — парсер/форматтер длин VCB (`2500`, `2.5m`, `1'6"`, `300,200`).

## varstotas_render
- `Camera` (f64): perspective/ortho, orbit/pan/zoom, `screen_ray`, `world_to_screen`, `zoom_extents`, стандартные виды.
- `Renderer`: рисует **во внеэкранную текстуру** (MSAA 4x + depth), регистрирует её в egui как `TextureId`; app показывает её как изображение и рисует 2D-оверлеи (маркеры inference, рамку выделения, подписи) через `egui::Painter`.
- Входные данные:
  - `SceneGeometry` — треугольники (pos, normal, color) + рёбра (pos, color). Загружается **только при изменении модели**.
  - `Overlay3d` — линии в 3D на каждый кадр (inference-оси, направляющие, превью инструмента): толщина в px, пунктир, «поверх всего» или с depth-test.
  - `RenderStyle` — фон (небо/земля), рёбра вкл/выкл, профили, X-ray, wireframe, hidden-line, сетка, оси.
- Толстые линии — расширение в квады в вершинном шейдере (в wgpu нет line width).

## varstotas_app
- `App` (eframe): меню, левый тулбар, правый лоток панелей (Entity Info, Materials, Tags, Outliner, Scenes, Instructor), статус-бар с VCB.
- `tools/` — trait `Tool` (state machine): `on_mouse_move/down/up`, `on_key`, `on_vcb(input)`, `overlay()`, `status_text()`.
- `inference/` — движок привязок: собирает кандидатов (вершины/середины/рёбра/грани/оси/параллели/направляющие) в экранном пространстве, ранжирует, отдаёт `InferenceResult { point, kind, constraint_dir, ref_entity }`.
- `nav` — камера: средняя кнопка — orbit, Shift+средняя — pan, колесо — zoom к курсору.
- `scene_builder` — `Model + selection + context` → `SceneGeometry` (подсветка выделения, тусклость вне редактируемой группы).
- `picking` — CPU: грани — raycast, рёбра/вершины — расстояние в пикселях после проекции.

## Поток данных кадра
```
input → nav / active Tool ── (InferenceEngine) ──→ ops над Model (через History.commit)
                                                   │ model_version++
Model ──scene_builder (если version изменилась)──→ Renderer.set_geometry
Tool.overlay() ─→ Overlay3d + 2D-маркеры ──→ Renderer.render → egui image + painter
```

## Правила совместной разработки
1. Каждый крейт имеет владельца (см. `04-TEAM.md`). Чужой крейт не правим — пишем запрос в `docs/requests/`.
2. Публичный API крейта описан в его `lib.rs`; ломающие изменения — только с обновлением этого документа.
3. `cargo build`, `cargo test`, `cargo clippy` — зелёные перед сдачей задачи.
4. Ядро покрыто unit-тестами на каждую операцию (особенно топология граней).
