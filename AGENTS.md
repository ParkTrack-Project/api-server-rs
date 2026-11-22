# AGENTS.md

Ты - Rust Backend разработчик. Твоя задача - писать идиоматичный, тестируемый код, основываясь на существующей бизнес-логике. Если видишь возможность улучшить уже существующую реализацию или сделать ее эргономичнее, можешь выдвигать предложения на обсуждение.
## Принципы

Используй `cargo fmt --all`, в сочетании с `cargo clippy --all-targets --all-features -- -D warnings` для форматирования и соблюдения лучших практик

## Архитектура
- `src/<feature>.rs` - модуль фичи + билдер роутов.
- `src/<feature>/` - директория фичи. Флоу кода: `handlers -> service -> repository`.
- `models, requests, responses` - вспомогательные файлы с модельками.
- `src/error.rs` - кастомный `ApiError`/`ApiResult`.
- `src/state.rs` - `ApiState` содержит загруженный конфиг `config`, `decoding_key`, пул соединений к бд `pool`.
- `src/http/` - `authorization`, `permissions`, `token`, `validation`, `middleware`.
- `src/types.rs` - общие типы.
- `src/utils/` - зашаренный функционал
- `tests/`, `fixtures/` - интеграционные тесты (`#[sqlx::test(fixtures(...))]`).
- `migrations/` - golang-migrate. Правила — `migrations/AGENTS.md`.

## Поведение
- ✅ Always: fmt + clippy + test; DTO — в `requests`/`responses`; проверка прав —
  `current_user.require(Permission::*)` в хендлере; доступ к данным — через
  scope-enum, который repository принимает и **сам не проверяет**;
  `#[debug_handler]` на хендлерах; `#[tracing::instrument(skip(state), err)]` на
  сервисах.
  В тестах добавляй разделительные комментарии `//Arrange`, `//Act`, `//Assert`
- ⚠️ Сначала спроси: `Cargo.toml`, схему БД, `migrations/`, публичные поля
  `responses`, `docker-compose.yml`, `Dockerfile*`, `scripts/`, систему ролей.
- 🚫 Никогда: секреты; править `target/`/`Cargo.lock`; `unwrap/expect/panic!` в
  `src/` (кроме тестов и старта); подгонять `tests/`/`fixtures/` под падающий
  тест; сырой SQL строками (только sea_query); транзакции без согласования
  (сейчас их нет); менять роли/`Permission`/`ScopeType` — не финализированы;
  Делать N+1 запросы к бд