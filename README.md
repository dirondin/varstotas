# Varstotas

Лёгкий нативный 3D-редактор классического типа для macOS и Windows. Rust + egui + wgpu.

```bash
cargo run --release -p varstotas_app     # запустить
cargo test --workspace                 # тесты
```

## Документы
- [Видение и UX-принципы](docs/00-VISION.md)
- [Функции → план и статусы](docs/01-FEATURES.md)
- [Архитектура](docs/02-ARCHITECTURE.md)
- [Роадмап](docs/03-ROADMAP.md)
- [Команда и процесс](docs/04-TEAM.md)
- [**Текущий статус и прогресс**](docs/06-STATUS.md)
- [UX-спецификация инструментов](docs/05-UX-SPEC.md)
- [QA-отчёт M3](docs/qa/M3-report.md)

## Структура
```
crates/varstotas_kernel  — геометрическое ядро (липкая геометрия, push/pull, offset, follow me, IO)
crates/varstotas_render  — wgpu-рендер и камера
crates/varstotas_app     — приложение: UI, инструменты, inference, VCB
```

## Сборка релизов
Артефакты складываются в `dist/` (в git не попадает).

```bash
scripts/build-macos.sh      # macOS universal (arm64 + x86_64)
scripts/build-windows.sh    # Windows x86_64, кросс-сборка с macOS
```

- **macOS** — `scripts/build-macos.sh [--no-dmg]`: при необходимости ставит target `x86_64-apple-darwin`, собирает release под обе архитектуры, склеивает через `lipo`, собирает `dist/Varstotas.app` (Info.plist: `dev.vrst.app`, версия из Cargo, macOS ≥ 11, Retina), рисует иконку (`scripts/make_icon.py` → `iconutil` → `.icns`), подписывает ad-hoc (`codesign -s -`) и упаковывает `Varstotas-<версия>-macos-universal.zip` и `.dmg`. Подписи Developer ID и нотаризации нет — при первом запуске: правый клик → «Открыть».
- **Windows** — `scripts/build-windows.sh`: ставит target `x86_64-pc-windows-gnu` и `mingw-w64` (через brew), собирает `varstotas.exe` (GUI-подсистема, без консоли; иконка и version-ресурс встраиваются через `crates/varstotas_app/build.rs` / `winresource` + `windres`) и кладёт его в `dist/varstotas-windows-x86_64.zip`. Нужен Windows 10+ (UCRT); внешних DLL нет.
- Проверка сборки без UI-клика: `VARSTOTAS_SCREENSHOT=shot.bmp dist/Varstotas.app/Contents/MacOS/varstotas` — снимет окно и выйдет.
