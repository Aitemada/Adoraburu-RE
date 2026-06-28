use axum::{
    extract::{Multipart, Path, Query, State, DefaultBodyLimit},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, put, delete},
    Router,
};
use metrics::describe_counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use rand::seq::{IteratorRandom, SliceRandom};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::{fs, sync::RwLock};

const BASE_PATH: &str = "/var/www/AdoRaburu-Re";

#[derive(Serialize, Deserialize, Clone, Default)]
struct SongInfo {
    flavour_text: String,
    lyrics: String,
}

#[derive(Serialize)]
struct SongResponse {
    name: String,
}

#[derive(Deserialize)]
struct DeleteQuery {
    confirm: Option<String>,
}

type Db = HashMap<String, SongInfo>;

struct AppState {
    db: Arc<RwLock<Db>>,
}

#[tokio::main]
async fn main() {
    let builder = PrometheusBuilder::new();
    let handle = builder
        .install_recorder()
        .expect("failed to install recorder");
    describe_counter!("songs_added_total", "Total number of songs added via the API");

    let db = sync_db().await;
    let shared_state = Arc::new(AppState {
        db: Arc::new(RwLock::new(db)),
    });

    let app = Router::new()
        .route("/api/add-song/:name", post(add_song))
        .route("/api/delete-song/:name", delete(delete_song))
        .route("/api/random-song", get(random_song))
        .route("/metrics", get(move || std::future::ready(handle.render())))
        .route("/api/health", get(|| async { "OK" }))

        .route("/api/songs", get(get_db))
        .route("/api/db", put(update_db))
        .route("/api/songlist/update", post(update_songlist_endpoint))
        
        .route("/api/suggestions/update-all", post(update_all_suggestions))
        .route("/api/suggestions/update/:name", post(update_single_suggestions))

        .route("/api/lyrics/:name", post(push_lyrics))
        .route("/api/flavour/:name", put(push_flavour))

        .route("/api/upload/video/:name", post(push_vid))
        .layer(DefaultBodyLimit::disable())
        .route("/api/upload/image/:name/:img_type", post(push_image))
        .layer(DefaultBodyLimit::max(500 * 1024 * 1024)) 
        .with_state(shared_state.clone())

        .route("/api/css/:name", get(get_css).put(update_css))
        .with_state(shared_state.clone());

    println!("Server running on 0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn sync_db() -> Db {
    let db_path = format!("{}/exstg-songs.json", BASE_PATH);
    let mut db: Db = if let Ok(data) = fs::read_to_string(&db_path).await {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    };

    let mut dir = fs::read_dir(format!("{}/pages", BASE_PATH)).await.unwrap();
    while let Ok(Some(entry)) = dir.next_entry().await {
        let name = entry.file_name().into_string().unwrap();
        if name == "TEMPLATEPAGE" || name == "Songlist" || name.starts_with("404") || name.contains('.') {
            continue;
        }

        if let Ok(meta) = entry.metadata().await {
            if !meta.is_dir() { continue; }
        } else {
            continue;
        }

        if !db.contains_key(&name) {
            db.insert(
                name,
                SongInfo {
                    flavour_text: "Flavour text placeholder".into(),
                    lyrics: "Lyrics placeholder".into(),
                },
            );
        }
    }
    save_db(&db).await;
    db
}

async fn save_db(db: &Db) {
    let db_path = format!("{}/exstg-songs.json", BASE_PATH);
    let json = serde_json::to_string_pretty(db).unwrap();
    fs::write(db_path, json).await.unwrap();
}

async fn get_db(State(state): State<Arc<AppState>>) -> Json<Db> {
    let db = state.db.read().await;
    Json(db.clone())
}

async fn update_db(State(state): State<Arc<AppState>>, Json(new_db): Json<Db>) -> impl IntoResponse {
    let mut db = state.db.write().await;
    *db = new_db;
    save_db(&db).await;
    (StatusCode::OK, "Database overwritten successfully")
}

async fn add_song(
    State(state): State<Arc<AppState>>,
    Path(songname): Path<String>,
) -> impl IntoResponse {
    let template_path = format!("{}/pages/TEMPLATEPAGE", BASE_PATH);
    let new_song_path = format!("{}/pages/{}", BASE_PATH, songname);

    if std::path::Path::new(&new_song_path).exists() {
        return (StatusCode::CONFLICT, "Error: Song page already exists").into_response();
    }

    let mut options = fs_extra::dir::CopyOptions::new();
    options.copy_inside = true;
    if let Err(_) = tokio::task::spawn_blocking(move || {
        fs_extra::dir::copy(&template_path, &new_song_path, &options)
    }).await.unwrap() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Error: Could not copy template").into_response();
    }

    let extensions = ["html", "css", "js"];
    for ext in extensions {
        let upper_file = format!("{}/pages/{}/TEMPLATE.{}", BASE_PATH, songname, ext);
        let lower_file = format!("{}/pages/{}/template.{}", BASE_PATH, songname, ext);
        let new_file = format!("{}/pages/{}/{}.{}", BASE_PATH, songname, songname, ext);
        
        if std::path::Path::new(&upper_file).exists() {
            fs::rename(&upper_file, &new_file).await.ok();
        } else if std::path::Path::new(&lower_file).exists() {
            fs::rename(&lower_file, &new_file).await.ok();
        }
    }

    let html_path = format!("{}/pages/{}/{}.html", BASE_PATH, songname, songname);
    if let Ok(content) = fs::read_to_string(&html_path).await {
        let mut new_content = content.replace("PH-SONGNAME", &songname);
        new_content = new_content.replace("template.css", &(songname.clone() + ".css"));
        new_content = new_content.replace("template.js", &(songname.clone() + ".js"));
        new_content = new_content.replace("Adounravel0.jpg", &(songname.clone() + "Bg.jpg"));
        new_content = new_content.replace("adounravel.mp4", &(songname.clone() + ".mp4"));
        fs::write(&html_path, new_content).await.ok();
    }

    {
        let mut db = state.db.write().await;
        db.insert(
            songname.clone(),
            SongInfo { flavour_text: "A great new song".into(), lyrics: "[PH LYRICS]".into() },
        );
        save_db(&db).await;
    }

    let db_read = state.db.read().await;
    update_songlist_html(&db_read).await;
    update_page_suggestions_html(&songname, &db_read).await;

    metrics::counter!("songs_added_total").increment(1);
    (StatusCode::OK, format!("Song {} page was added!", songname)).into_response()
}

async fn delete_song(
    State(state): State<Arc<AppState>>,
    Path(songname): Path<String>,
    Query(query): Query<DeleteQuery>,
) -> impl IntoResponse {
    if query.confirm.as_deref() != Some("Y") && query.confirm.as_deref() != Some("y") {
        let msg = format!("Do you really want to delete the '{}' page? Add '?confirm=Y' to your URL to proceed.", songname);
        return (StatusCode::BAD_REQUEST, msg).into_response();
    }

    let dir_path = format!("{}/pages/{}", BASE_PATH, songname);
    if std::path::Path::new(&dir_path).exists() {
        let _ = tokio::fs::remove_dir_all(&dir_path).await;
    }

    {
        let mut db = state.db.write().await;
        db.remove(&songname);
        save_db(&db).await;
    }

    let db_read = state.db.read().await;
    update_songlist_html(&db_read).await;

    (StatusCode::OK, format!("Song '{}' was deleted successfully!", songname)).into_response()
}

async fn update_songlist_endpoint(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.read().await;
    update_songlist_html(&db).await;
    (StatusCode::OK, "Songlist updated")
}

async fn update_songlist_html(db: &Db) {
    let path = format!("{}/pages/Songlist/songlist.html", BASE_PATH);
    if let Ok(content) = fs::read_to_string(&path).await {
        let mut new_lis = String::new();
        for song in db.keys() {
            new_lis.push_str(&format!(
                "        <li><a href=\"../../pages/{}/{}.html\">{}</a></li>\n",
                song, song, song
            ));
        }

        let re = Regex::new(r"(?s)<!-- SONGLIST_START -->.*?<!-- SONGLIST_END -->").unwrap();
        let replacement = format!("<!-- SONGLIST_START -->\n{}        <!-- SONGLIST_END -->", new_lis);
        let new_content = re.replace(&content, &replacement).to_string();

        fs::write(path, new_content).await.ok();
    }
}

async fn update_all_suggestions(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.read().await;
    for song in db.keys() {
        update_page_suggestions_html(song, &db).await;
    }
    (StatusCode::OK, "All pages' suggestions updated")
}

async fn update_single_suggestions(
    State(state): State<Arc<AppState>>,
    Path(songname): Path<String>,
) -> impl IntoResponse {
    let db = state.db.read().await;
    if !db.contains_key(&songname) {
        return (StatusCode::NOT_FOUND, "Song not found").into_response();
    }
    update_page_suggestions_html(&songname, &db).await;
    (StatusCode::OK, format!("Suggestions updated for {}", songname)).into_response()
}

async fn update_page_suggestions_html(target_song: &str, db: &Db) {
    let mut suggestions_html = String::from("<!-- SUGGESTIONS_START -->\n    <div id=\"othervids\">\n");

    {
        let mut rng = rand::thread_rng();
        let pool: Vec<&String> = db.keys().filter(|&k| k != target_song).collect();
        let chosen: Vec<&&String> = pool.choose_multiple(&mut rng, 3).collect();

        if chosen.is_empty() { return; }

        for &song in chosen {
            let flavour = db.get(song).map(|s| s.flavour_text.clone()).unwrap_or_default(); 
            let img_check_path = format!("{}/Images/{}.jpg", BASE_PATH, song);
            let final_img = if std::path::Path::new(&img_check_path).exists() {
                format!("{}.jpg", song)
            } else {
                "AdoLogo.jpg".to_string()
            };

            suggestions_html.push_str(&format!(
                "        <div class=\"specialBorder\"><a href=\"../../pages/{}/{}.html\"><div class=\"vidsug\">\n           <img style=\"width: 135px; height: auto;\" src=\"../../Images/{}\" alt=\"thumbnail\">\n           <h3>{}</h3>\n           <span>{}</span>\n        </div></a></div>\n",
                song, song, final_img, song, flavour
            ));
        }
        suggestions_html.push_str("    </div>\n    <!-- SUGGESTIONS_END -->");
    } 

    let path = format!("{}/pages/{}/{}.html", BASE_PATH, target_song, target_song);
    if let Ok(content) = fs::read_to_string(&path).await {
        let re = Regex::new(r"(?s)<!-- SUGGESTIONS_START -->.*?<!-- SUGGESTIONS_END -->").unwrap();
        let new_content = re.replace(&content, &suggestions_html).to_string();
        fs::write(path, new_content).await.ok();
    }
}
async fn push_lyrics(
    State(state): State<Arc<AppState>>,
    Path(songname): Path<String>,
    body: String, // Accept raw text!
) -> impl IntoResponse {
    {
        let mut db = state.db.write().await;
        if let Some(song_info) = db.get_mut(&songname) {
            song_info.lyrics = body.clone();
            save_db(&db).await;
        } else {
            return (StatusCode::NOT_FOUND, "Song not found in DB").into_response();
        }
    }

    let path = format!("{}/pages/{}/{}.html", BASE_PATH, songname, songname);
    if let Ok(content) = fs::read_to_string(&path).await {
        let re = Regex::new(r"(?s)<pre id=.lyrics.>(.*?)</pre>").unwrap();
        let replacement = format!("<pre id=\"lyrics\">\n{}\n</pre>", body);
        let new_content = re.replace(&content, &replacement).to_string();
        fs::write(path, new_content).await.unwrap();
    }
    (StatusCode::OK, "Lyrics updated successfully").into_response()
}
async fn push_flavour(
    State(state): State<Arc<AppState>>,
    Path(songname): Path<String>,
    body: String,
) -> impl IntoResponse {
    {
        let mut db = state.db.write().await;
        if let Some(song_info) = db.get_mut(&songname) {
            song_info.flavour_text = body.clone();
            save_db(&db).await;
        } else {
            return (StatusCode::NOT_FOUND, "Song not found in DB").into_response();
        }
    }
    
    // Automatically re-roll suggestions globally so the site updates immediately
    let db_read = state.db.read().await;
    for song in db_read.keys() {
        update_page_suggestions_html(song, &db_read).await;
    }

    (StatusCode::OK, format!("Flavour text updated for {}", songname)).into_response()
}

#[derive(Deserialize)]
struct MediaQuery {
    overwrite: Option<bool>,
}

async fn push_vid(
    Path(songname): Path<String>,
    Query(query): Query<MediaQuery>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let path = format!("{}/videos/{}.mp4", BASE_PATH, songname);
    handle_upload(path, query.overwrite.unwrap_or(false), &mut multipart).await
}

async fn push_image(
    Path((songname, img_type)): Path<(String, String)>,
    Query(query): Query<MediaQuery>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let filename = if img_type == "bg" {
        format!("{}Bg.jpg", songname)
    } else if img_type == "thumbic" {
        format!("{}.jpg", songname)
    } else {
        return (StatusCode::BAD_REQUEST, "Invalid image type. Use 'bg' or 'thumbic'").into_response();
    };

    let path = format!("{}/Images/{}", BASE_PATH, filename);
    handle_upload(path, query.overwrite.unwrap_or(false), &mut multipart).await
}

async fn handle_upload(path: String, overwrite: bool, multipart: &mut Multipart) -> axum::response::Response {
    if std::path::Path::new(&path).exists() && !overwrite {
        return (StatusCode::CONFLICT, "File exists. Use ?overwrite=true query parameter to replace.").into_response();
    }

    if let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if let Ok(data) = field.bytes().await {
            fs::write(path, data).await.unwrap();
            return (StatusCode::OK, "File successfully uploaded").into_response();
        }
    }
    (StatusCode::BAD_REQUEST, "Failed to read multipart data").into_response()
}

async fn get_css(Path(songname): Path<String>) -> impl IntoResponse {
    let path = format!("{}/pages/{}/{}.css", BASE_PATH, songname, songname);
    match fs::read_to_string(path).await {
        Ok(css) => (StatusCode::OK, css).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "CSS file not found").into_response(),
    }
}

async fn update_css(Path(songname): Path<String>, body: String) -> impl IntoResponse {
    let path = format!("{}/pages/{}/{}.css", BASE_PATH, songname, songname);
    if !std::path::Path::new(&path).exists() {
        return (StatusCode::NOT_FOUND, "CSS file not found").into_response();
    }
    fs::write(path, body).await.unwrap();
    (StatusCode::OK, "CSS successfully updated").into_response()
}

async fn random_song(State(state): State<Arc<AppState>>) -> Json<SongResponse> {
    let db = state.db.read().await;
    let keys: Vec<&String> = db.keys().collect();
    let chosen = keys.choose(&mut rand::thread_rng()).unwrap_or(&&String::from("index")).to_string();
    Json(SongResponse { name: chosen })
}