use axum::{
    extract::State,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, FromRow};
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ForumPost {
    pub id: i64,
    pub nickname: String,
    pub race: String,
    pub avatar: String,
    pub text: String,
    pub time: String,
}

#[derive(Debug, Deserialize)]
pub struct NewPost {
    pub nickname: String,
    pub race: String,
    pub avatar: String,
    pub text: String,
    pub time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProfile {
    pub nickname: String,
    pub race: String,
    pub email: String,
    pub avatar: String,
    pub role: String,
    pub album: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterData {
    pub nickname: String,
    pub race: String,
    pub email: String,
}

type AppState = Arc<SqlitePool>;

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect("sqlite://./chat.db")
        .await
        .expect("Failed to connect to database");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nickname TEXT NOT NULL,
            race TEXT NOT NULL,
            avatar TEXT NOT NULL,
            text TEXT NOT NULL,
            time TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create posts table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            nickname TEXT PRIMARY KEY,
            race TEXT NOT NULL,
            email TEXT NOT NULL,
            avatar TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'USER',
            album TEXT NOT NULL DEFAULT '[]'
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS admins (
            nickname TEXT PRIMARY KEY
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create admins table");

    println!("✅ 数据库初始化完成");

    let state = Arc::new(pool);

    let app = Router::new()
        .route("/", get(home_page))
        .route("/about", get(about_page))
        .route("/api/posts", get(get_posts).post(add_post))
        .route("/api/posts/:id", post(delete_post))
        .route("/api/register", post(register_user))
        .route("/api/users/:nickname", get(get_user))
        .route("/api/users/:nickname", post(update_user))
        .route("/api/admins", get(get_admins).post(toggle_admin))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("🚀 服务已启动：http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn get_posts(State(pool): State<AppState>) -> Json<Vec<ForumPost>> {
    let posts = sqlx::query_as::<_, ForumPost>("SELECT * FROM posts ORDER BY id DESC LIMIT 100")
        .fetch_all(&*pool)
        .await
        .unwrap_or_default();
    Json(posts)
}

async fn add_post(
    State(pool): State<AppState>,
    Json(new_post): Json<NewPost>,
) -> Json<serde_json::Value> {
    let result = sqlx::query(
        "INSERT INTO posts (nickname, race, avatar, text, time) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&new_post.nickname)
    .bind(&new_post.race)
    .bind(&new_post.avatar)
    .bind(&new_post.text)
    .bind(&new_post.time)
    .execute(&*pool)
    .await;

    match result {
        Ok(_) => Json(serde_json::json!({ "status": "ok" })),
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}

async fn delete_post(
    State(pool): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Json<serde_json::Value> {
    let result = sqlx::query("DELETE FROM posts WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await;

    match result {
        Ok(_) => Json(serde_json::json!({ "status": "ok" })),
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}

async fn register_user(
    State(pool): State<AppState>,
    Json(data): Json<RegisterData>,
) -> Json<serde_json::Value> {
    let default_avatar = "https://api.dicebear.com/7.x/bottts/svg?seed=goat";

    let existing: Option<(String,)> = sqlx::query_as("SELECT nickname FROM users WHERE nickname = ?")
        .bind(&data.nickname)
        .fetch_optional(&*pool)
        .await
        .unwrap_or(None);

    if existing.is_some() {
        return Json(serde_json::json!({ "status": "error", "message": "昵称已被使用" }));
    }

    let result = sqlx::query(
        "INSERT INTO users (nickname, race, email, avatar, role, album) VALUES (?, ?, ?, ?, 'USER', '[]')",
    )
    .bind(&data.nickname)
    .bind(&data.race)
    .bind(&data.email)
    .bind(default_avatar)
    .execute(&*pool)
    .await;

    match result {
        Ok(_) => Json(serde_json::json!({ "status": "ok", "avatar": default_avatar })),
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}

async fn get_user(
    State(pool): State<AppState>,
    axum::extract::Path(nickname): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let user: Option<UserProfile> = sqlx::query_as("SELECT * FROM users WHERE nickname = ?")
        .bind(&nickname)
        .fetch_optional(&*pool)
        .await
        .unwrap_or(None);

    let is_admin: Option<(String,)> = sqlx::query_as("SELECT nickname FROM admins WHERE nickname = ?")
        .bind(&nickname)
        .fetch_optional(&*pool)
        .await
        .unwrap_or(None);

    match user {
        Some(mut u) => {
            if nickname == "问彩蝶" {
                u.role = "SUPER_ADMIN".to_string();
            } else if is_admin.is_some() {
                u.role = "ADMIN".to_string();
            }
            Json(serde_json::json!({ "status": "ok", "user": u }))
        },
        None => Json(serde_json::json!({ "status": "error", "message": "用户不存在" })),
    }
}

async fn update_user(
    State(pool): State<AppState>,
    axum::extract::Path(nickname): axum::extract::Path<String>,
    Json(data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    if let Some(avatar) = data.get("avatar").and_then(|v| v.as_str()) {
        let result = sqlx::query("UPDATE users SET avatar = ? WHERE nickname = ?")
            .bind(avatar)
            .bind(&nickname)
            .execute(&*pool)
            .await;

        if result.is_err() {
            return Json(serde_json::json!({ "status": "error", "message": "更新头像失败" }));
        }
    }

    if let Some(album) = data.get("album").and_then(|v| v.as_str()) {
        let result = sqlx::query("UPDATE users SET album = ? WHERE nickname = ?")
            .bind(album)
            .bind(&nickname)
            .execute(&*pool)
            .await;

        if result.is_err() {
            return Json(serde_json::json!({ "status": "error", "message": "更新相册失败" }));
        }
    }

    Json(serde_json::json!({ "status": "ok" }))
}

async fn get_admins(State(pool): State<AppState>) -> Json<Vec<String>> {
    let admins: Vec<(String,)> = sqlx::query_as("SELECT nickname FROM admins")
        .fetch_all(&*pool)
        .await
        .unwrap_or_default();

    Json(admins.into_iter().map(|(n,)| n).collect())
}

async fn toggle_admin(
    State(pool): State<AppState>,
    Json(data): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let nickname = data.get("nickname").and_then(|v| v.as_str()).unwrap_or("");

    let exists: Option<(String,)> = sqlx::query_as("SELECT nickname FROM admins WHERE nickname = ?")
        .bind(nickname)
        .fetch_optional(&*pool)
        .await
        .unwrap_or(None);

    let result = if exists.is_some() {
        sqlx::query("DELETE FROM admins WHERE nickname = ?")
            .bind(nickname)
            .execute(&*pool)
            .await
    } else {
        sqlx::query("INSERT INTO admins (nickname) VALUES (?)")
            .bind(nickname)
            .execute(&*pool)
            .await
    };

    match result {
        Ok(_) => Json(serde_json::json!({ "status": "ok" })),
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}

async fn home_page() -> Html<String> {
    let html_content = std::fs::read_to_string("index.html")
        .unwrap_or_else(|_| "<h1>Error: index.html not found</h1>".to_string());
    Html(html_content)
}

async fn about_page() -> Html<&'static str> {
    Html(r#"
        <!DOCTYPE html>
        <html lang="zh-CN">
        <head>
            <meta charset="UTF-8">
            <title>关于 | God and the Goat</title>
            <style>
                body {
                    font-family: 'PingFang SC', Georgia, serif;
                    display: flex;
                    flex-direction: column;
                    justify-content: center;
                    align-items: center;
                    height: 100vh;
                    background-color: #0b0f19;
                    color: white;
                }
                .btn {
                    padding: 12px 28px;
                    background: rgba(212, 175, 55, 0.15);
                    border: 1px solid rgba(212, 175, 55, 0.4);
                    color: #d4af37;
                    text-decoration: none;
                    border-radius: 30px;
                    margin-top: 28px;
                    transition: 0.3s;
                }
                .btn:hover {
                    background: rgba(212, 175, 55, 0.3);
                    color: #ffffff;
                }
            </style>
        </head>
        <body>
            <h1>🐐 故事与篇章（/about）</h1>
            <p style="margin-top: 12px; color: #94a3b8;">这里可以放入关于《神与山羊》的背景、文章、设定或图集。</p>
            <a href="/" class="btn">← 返回主页</a>
        </body>
        </html>
    "#)
}
