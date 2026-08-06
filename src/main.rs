use axum::{
    extract::State,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;

// 1. 论坛发帖数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumPost {
    pub nickname: String,
    pub race: String,
    pub avatar: Option<String>,
    pub text: String,
    pub time: String,
}

// 共享的内存状态（用于全服保存论坛消息）
type AppState = Arc<Mutex<Vec<ForumPost>>>;

#[tokio::main]
async fn main() {
    let posts = Arc::new(Mutex::new(Vec::<ForumPost>::new()));

    let app = Router::new()
        .route("/", get(home_page))
        .route("/about", get(about_page))
        .route("/api/posts", get(get_posts).post(add_post))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(posts);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("🚀 服务已启动：http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

// 2. 论坛 API 处理函数：获取所有发言
async fn get_posts(State(state): State<AppState>) -> Json<Vec<ForumPost>> {
    let posts = state.lock().unwrap();
    Json(posts.clone())
}

// 3. 论坛 API 处理函数：发布新发言
async fn add_post(
    State(state): State<AppState>,
    Json(new_post): Json<ForumPost>,
) -> Json<serde_json::Value> {
    let mut posts = state.lock().unwrap();
    posts.push(new_post);
    // 保持最多 100 条全服最新发言
    if posts.len() > 100 {
        posts.remove(0);
    }
    Json(serde_json::json!({ "status": "ok" }))
}

// 4. 主页（God and the Goat 主题）
async fn home_page() -> Html<&'static str> {
    Html(r#"
        <!DOCTYPE html>
        <html lang="zh-CN">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>God and the Goat | 神与山羊</title>
            <style>
                * {
                    box-sizing: border-box;
                    margin: 0;
                    padding: 0;
                }
                body {
                    height: 100vh;
                    width: 100vw;
                    background: radial-gradient(circle, rgba(0,0,0,0.3) 0%, rgba(0,0,0,0.85) 100%), url('/static/bg.jpg');
                    background-size: cover;
                    background-position: center;
                    background-repeat: no-repeat;
                    font-family: -apple-system, BlinkMacSystemFont, "PingFang SC", "Georgia", serif;
                    
                    display: flex;
                    flex-direction: column;
                    justify-content: center;
                    align-items: center;
                    text-align: center;
                }

                h1.art-title {
                    font-size: 4.5rem;
                    font-weight: 800;
                    letter-spacing: 5px;
                    margin-bottom: 12px;
                    background: linear-gradient(180deg, #ffffff 0%, #d4af37 100%);
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    filter: drop-shadow(0 0 20px rgba(212, 175, 55, 0.4)) drop-shadow(0 8px 16px rgba(0, 0, 0, 0.9));
                }

                p.subtitle {
                    font-size: 1.4rem;
                    color: rgba(255, 255, 255, 0.85);
                    margin-bottom: 45px;
                    font-weight: 300;
                    letter-spacing: 6px;
                    text-shadow: 0 2px 10px rgba(0, 0, 0, 0.8);
                }

                .btn {
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    padding: 14px 42px;
                    font-size: 1.05rem;
                    font-weight: 500;
                    color: #0f172a;
                    text-decoration: none;
                    border-radius: 50px;
                    background: linear-gradient(135deg, #fbf5b7 0%, #d4af37 100%);
                    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.5);
                    transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
                }

                .btn:hover {
                    transform: translateY(-4px) scale(1.03);
                    box-shadow: 0 12px 30px rgba(212, 175, 55, 0.5);
                    background: linear-gradient(135deg, #ffffff 0%, #fbf5b7 100%);
                }

                .btn span {
                    margin-left: 8px;
                    transition: transform 0.3s ease;
                }

                .btn:hover span {
                    transform: translateX(5px);
                }
            </style>
        </head>
        <body>
            <h1 class="art-title">God and the Goat</h1>
            <p class="subtitle">神 与 山 羊</p>
            <a href="/about" class="btn">探索序章 <span>→</span></a>
        </body>
        </html>
    "#)
}

// 5. 第二个页面
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
