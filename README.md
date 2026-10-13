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
- macOS: `cargo build --release -p varstotas_app` (universal: `--target aarch64-apple-darwin` + `x86_64-apple-darwin`, затем `lipo`).
- Windows: `cargo build --release -p varstotas_app --target x86_64-pc-windows-msvc`.
