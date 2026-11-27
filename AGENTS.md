# AGENTS.md

Ты - Rust Backend разработчик. Твоя задача - писать идиоматичный, тестируемый код, основываясь на существующей бизнес-логике. Если видишь возможность улучшить уже существующую реализацию или сделать ее эргономичнее, можешь выдвигать предложения на обсуждение.
## Принципы

Используй `cargo fmt --all`, в сочетании с `cargo clippy --all-targets --all-features -- -D warnings` для форматирования и соблюдения лучших практик

## Архитектура
- `src/features/<feature>.rs` - модуль фичи + билдер роутов.
- `src/features/<feature>/` - директория фичи. Флоу кода: `handlers -> service -> repository`.
- `models, requests, responses` - вспомогательные файлы с модельками.
- `src/features/<feature>/partners.rs` - опциональный субмодуль с хендлерами `/partners/<partner_id>/<feature>...`, если у фичи есть партнерская зона видимости
- `src/error.rs` - кастомный `ApiError`/`ApiResult`.
- `src/state.rs` - `ApiState` содержит загруженный конфиг `config`, `decoding_key`, пул соединений к бд `pool`.
- `src/http/` - `authorization`, `permissions`, `token`, `validation`, `middleware`.
- `src/types.rs` - общие типы.
- `src/utils/` - зашаренный функционал
- `tests/`, `fixtures/` - интеграционные тесты (`#[sqlx::test(fixtures(...))]`).
  Все `#[sqlx::test]` используют `migrator = "crate::support::MIGRATOR"`: проверка тестового подключения должна выполняться до любых операций SQLx.
  Запуск: `bash scripts/test.sh`; подключение — только `TEST_DATABASE_URL` из окружения или `.env.test`, без fallback на обычный `.env`.
- `migrations/` - golang-migrate. Правила — `migrations/AGENTS.md`.

## Best practices

### `<feature>/handlers.rs`:
- Нейминг функций и комментарии

*Делай*:

```rust
// METHOD /<feature>/....
#[debug_handler]
pub async fn <method>_<feature>(...)

// Референс

// GET /cameras/{camera_id}
#[debug_handler]
pub async fn get_camera(...)
```

*Вместо*

```rust
#[debug_handler]
pub async fn get(...)

// (Нету описания хендлера и фичи в названии)
```

### `<feature>/service.rs`:

- Нейминг у CRUD

*Делай*:

```rust
create_<feature>()
list_<feature>()
update_<feature>()
delete_<feature>()
```

*Вместо*:

```rust
create()
list()
update()
delete()
```

### `<feature>/repository.rs`:
- В update запросах

*Делай*:

```rust
.values(
  [
    query.<поле>.map(|v| (<Название таблицы>::<Атрибут>, v.into()))
    ... // все остальные поля
  ]
  .into_iter()
  .flatten()
)
```

*Вместо*:

```rust
if let Some(v) = query.<field1> {
    update.value(<Table>::<Column>, v);
}
if let Some(v) = query.<field2> {
    update.value(<Table>::<Column>, v);
}
if let Some(v) = query.<field3> {
    update.value(<Table>::<Column>, v);
}
```

- При подстановке `.columns()`

*Делай*:

```rust
// в <feature>/models.rs

pub const <TABLE>_COLUMNS: [<Table>; N] = [
  <Table>::<Column>,
  ... // для каждой колонки
];

// Референс

pub const CAMERA_COLUMNS: [Cameras; 14] = [
    Cameras::CameraId,
    Cameras::Title,
    Cameras::Source,
    Cameras::ImageWidth,
    Cameras::ImageHeight,
    Cameras::Calib,
    Cameras::Latitude,
    Cameras::Longitude,
    Cameras::PartnerId,
    Cameras::CreatedByUserId,
    Cameras::IsActive,
    Cameras::LastSnapshotAt,
    Cameras::CreatedAt,
    Cameras::UpdatedAt,
];
```

*Вместо*:

```rust
// <feature>/repository.rs

pub fn columns()
```

*Если*: схема колонн переиспользуется в нескольких запросах

- Фильтры

*Делай*

```rust
q.apply_if(query.<field1>, |stmt, v| {
    stmt.<condition method>(...);
})
.apply_if(query.<field2>, |stmt, v| {
    stmt.<condition method>(...);
})
```

*Вместо*:

```rust
if let Some(v) = query.<field1> {
  stmt.<condition method>(...);
}
if let Some(v) = query.<field2> {
  stmt.<condition method>(...);
}
```

Потому что семантика фильтра у `.apply_if()` более очевидна, чем у `if let Some(v) = query.<field> {...}`

- Нейминг у CRUD методов create, list, update, delete для основной модели фичи

*Делай*:

```rust
create_<feature>()
list_<feature>()
update_<feature>()
delete_<feature>()
```

*Вместо*:

```rust
create()
list()
update()
delete()
```

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