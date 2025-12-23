# Примеры использования RPM API

## HTTP API для расширений браузера

### Аутентификация

```bash
curl -X POST http://127.0.0.1:8765/api/auth \
  -H "Content-Type: application/json" \
  -d '{"master_password": "your_master_password"}'
```

Ответ:
```json
{
  "token": "abc123...",
  "expires_at": "2024-01-01T12:00:00Z"
}
```

### Создание пароля

```bash
curl -X POST http://127.0.0.1:8765/api/passwords \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "title": "GitHub",
    "username": "user@example.com",
    "password": "secure_password_123",
    "url": "https://github.com",
    "notes": "Personal account",
    "tags": ["development", "git"]
  }'
```

### Получение списка паролей

```bash
curl http://127.0.0.1:8765/api/passwords \
  -H "Authorization: Bearer <token>"
```

## Использование в коде Rust

### Инициализация криптографии

```rust
use rpm::crypto::CryptoManager;

let crypto = CryptoManager::new()?;

// Хеширование мастер-пароля
let hash = crypto.hash_password("my_master_password")?;

// Проверка пароля
let is_valid = crypto.verify_password("my_master_password", &hash)?;

// Шифрование пароля
let key = derive_key("master_password", None)?;
let (ciphertext, nonce) = crypto.encrypt_password("secret_password", &key)?;

// Расшифровка пароля
let decrypted = crypto.decrypt_password(&ciphertext, &nonce, &key)?;
```

### Работа с базой данных

```rust
use rpm::db::Database;
use rpm::models::PasswordEntryDto;
use uuid::Uuid;
use chrono::Utc;

let db = Database::new("rpm.db").await?;
db.init().await?;

// Создание записи
let entry = PasswordEntryDto {
    id: Uuid::new_v4(),
    title: "GitHub".to_string(),
    username: Some("user@example.com".to_string()),
    password: base64::encode(ciphertext),
    nonce: base64::encode(nonce),
    url: Some("https://github.com".to_string()),
    notes: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    tags: vec!["development".to_string()],
};

db.create_entry(entry).await?;
```

## TUI Команды

- `q` - Выход из приложения
- `↑` / `↓` - Навигация по списку
- `Enter` - Открыть запись (будет реализовано)
- `n` - Новая запись (будет реализовано)
- `d` - Удалить запись (будет реализовано)
- `/` - Поиск (будет реализовано)

## Конфигурация

Файл конфигурации находится в `~/.config/rpm/config.toml`:

```toml
database_path = "/home/user/.local/share/rpm/rpm.db"
server_port = 8765
server_host = "127.0.0.1"
encryption_algorithm = "aes256-gcm"
```

