# blog-project

Проект представляет собой пример  взаимодействия
программ(сервисов) предоставляющих разный интерфейс обмена информацией
посредством предоставления публичного API.

Crates

## Blog server Crate

Веб сервер с http публичным программным интерфейсом
GRPC сервис

### Приготовления

1. Создаем базу данных

createdb blog_example

createdb -T [source] [new database]

Тестовая база данных. Необязательно

createdb -T blog_example blog_example_test

2. Копируем файл .env.example в .env

### Запускаем сервер 

RUST_LOG="info,blog-project=debug" cargo  run

### Http запросы

- Регистрация

curl -v  -H "content-type: application/json" -d '{"username": "username", "email": "email@mail.ru", "password": "password"}' http://localhost:3000/api/auth/register

- Авторизация

curl -v  -H "content-type: application/json" -d '{"username": "username", "password": "password"}' http://localhost:3000/api/auth/login

- Создание поста

$token = токен полученный в ответе на запрос регистрации или авторизации

curl -v  -H "content-type: application/json" -H "Authorization: Bearer ${token}" -X POST -d '{"title": "title", "content": "content"}' http://localhost:3000/api/posts

- Список постов

curl -v  http://localhost:3000/api/posts?offset=0&limit=0

## Blog-client Crate

Библиотека обьединяющая в единый интерфейс методы
для взаимодействия с веб сервером или grpc сервисом

### Приготовления

Создаем символьную ссылку на файл blog-server/proto/blog.proto в папке blog-cli/proto

### Использование

use blog_client::{BlogClient, Transport};

let trasport = Transport::Http(args.server.unwrap_or("http://localhost:3000".into());
BlogClient::new(transport).await

### Blog-cli Crate

#### Программа с интерфейсом коммандой строки.

Возможности

- Регистраци
- Авторизация
- Создание поста
- Обновление поста
- Удаление поста
- Список постов
- Получение поста

*Использование*

Запускаем веб сервер и grpc сервис

1. cd blog-server

2. RUST_LOG="info,blog-project=debug" cargo  run

3. cd blog-cli

4. cargo run help (Список доступных команд)

## Blog wasm Crate

Минималистичное приложение фронтенд для взаимодействия с веб сервером.
Используются крейты для реализации связки программного интерфейса rust 
в js код путем трансформации в соответсвующие типы данных.

## Использование

1. Собираем пакет wasm.

cd blog-cli
wasm-pack build --target web

2. Запускаем веб сервер

python3 -m http.server 8001
