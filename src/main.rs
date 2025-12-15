use axum::{
    extract::{Query, State, Form},
    response::{Html, Redirect, IntoResponse},
    routing::{get, post},
    http::{HeaderMap, StatusCode},
    Router,
    Json,
};
use axum::response::Response;
use serde::Deserialize;
use tower_http::services::ServeDir;
use std::sync::Arc;
use askama::Template;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;
use percent_encoding::{percent_decode_str, NON_ALPHANUMERIC};
use axum::response::sse::{Event, Sse};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use futures::stream::Stream;
use std::convert::Infallible;

mod model;
mod state;

use state::{AppState, Lobby, Player, LobbyStatus, LobbyEvent};
use model::{MissionsData, Mission};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState::new().expect("Failed to load missions");

    let app = Router::new()
        .route("/", get(root))
        .route("/{seed}", get(splash))
        .route("/{seed}/coop", get(coop))
        .route("/{seed}/solo", get(solo))
        .route("/{seed}/randomizer", get(randomizer))
        .route("/all_missions", get(all_missions))

        .route("/{seed}/mission/{name}", get(mission_view))
        .route("/traitor/create", get(traitor_setup).post(traitor_create))
        .route("/traitor/{room_id}/join", get(traitor_join_view).post(traitor_join_action))
        .route("/traitor/{room_id}", get(traitor_lobby_view))
        .route("/traitor/{room_id}/sse", get(traitor_lobby_sse))
        .route("/traitor/{room_id}/start", post(traitor_start))
        .route("/traitor/{room_id}/role", get(traitor_role_view))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct ViewParams {
    view: Option<String>,
    seed: Option<String>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    view: String,
}

async fn root() -> axum::response::Redirect {
    let seed = rand::thread_rng().gen::<u32>().to_string();
    axum::response::Redirect::to(&format!("/{}", seed))
}

#[derive(Template)]
#[template(path = "layout_wrapper.html")]
struct LayoutWrapperTemplate {
    content: String,
}

fn render_response(headers: HeaderMap, content: String) -> Html<String> {
    if headers.contains_key("hx-request") {
        Html(content)
    } else {
        let wrapper = LayoutWrapperTemplate { content };
        Html(wrapper.render().unwrap())
    }
}

#[derive(Template)]
#[template(path = "partials/splash.html")]
struct SplashTemplate {
    seed: String,
}

async fn splash(
    axum::extract::Path(seed): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let template = SplashTemplate { seed };
    render_response(headers, template.render().unwrap())
}

#[derive(Template)]
#[template(path = "partials/coop.html")]
struct CoopTemplate {
    seed: String,
    next_seed: String,
    mission: Mission,
    // We might need to pass nested data for complex missions
    nested_mission: Option<Mission>,
    random_item_img: Option<String>,
    random_loadout: Option<Loadout>,
    random_number: Option<u32>,
    nested_gacha_items: Option<Vec<String>>,
    view_name: String,
}

#[derive(Clone)] // Added Clone here
struct Loadout {
    char_img: String,
    char_name: String,
    kart_img: String,
    kart_name: String,
    wheel_img: String,
    wheel_name: String,
    glider_img: String,
    glider_name: String,
}

async fn coop(
    State(state): State<AppState>,
    axum::extract::Path(seed): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    
    // Create RNG from seed
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&seed, &mut hasher);
    let seed_u64 = std::hash::Hasher::finish(&hasher);
    let mut rng = StdRng::seed_from_u64(seed_u64);
    let next_seed = rand::thread_rng().gen::<u32>().to_string();

    let missions = &state.missions.coop_granprix;
    let mission = missions.choose(&mut rng).unwrap().clone();
    
    // Resolve dynamic content
    let random_item_img = if mission.needs_random_item {
        let item = model::ITEMS.choose(&mut rng).unwrap();
        Some(format!("/assets/items/{}.png", item))
    } else {
        None
    };

    let random_loadout = if mission.needs_random_loadout {
        let c = model::CHARACTERS.choose(&mut rng).unwrap();
        let k = model::KARTS.choose(&mut rng).unwrap();
        let w = model::WHEELS.choose(&mut rng).unwrap();
        let g = model::GLIDERS.choose(&mut rng).unwrap();
        
        Some(Loadout {
            char_img: format!("/assets/characters/{}.webp", c.to_lowercase().replace(" ", "_")),
            char_name: c.to_string(),
            kart_img: format!("/assets/karts/{}.webp", k.to_lowercase().replace(" ", "_")),
            kart_name: k.to_string(),
            wheel_img: format!("/assets/wheels/{}.webp", w.to_lowercase().replace(" ", "_")),
            wheel_name: w.to_string(),
            glider_img: format!("/assets/gliders/{}.webp", g.to_lowercase().replace(" ", "_")),
            glider_name: g.to_string(),
        })
    } else {
        None
    };

    let random_number = mission.needs_random_number.map(|max| {
         rng.gen_range(1..=max)
    });

    let (nested_mission, nested_gacha_items) = if mission.needs_coop_singles {
         let sub = state.missions.coop_single.choose(&mut rng).cloned();
         let gacha = sub.as_ref().and_then(|m| {
             if m.needs_gacha_item_checklist {
                 Some(model::GACHA_ITEMS.iter().map(|s| format!("/assets/items/{}.png", s)).collect())
             } else { None }
         });
         (sub, gacha)
    } else { (None, None) };

    let template = CoopTemplate {
        seed,
        next_seed,
        mission,
        nested_mission,
        random_item_img,
        random_loadout,
        random_number,
        nested_gacha_items,
        view_name: "coop".to_string(),
    };

    render_response(headers, template.render().unwrap())
}

#[derive(Template)]
#[template(path = "partials/solo.html")]
struct SoloTemplate {
    seed: String,
    next_seed: String,
    mission: Mission,
    nested_mission: Option<Mission>,
    random_item_img: Option<String>,
    random_loadout: Option<Loadout>,
    random_number: Option<u32>,
    nested_gacha_items: Option<Vec<String>>,
    view_name: String,
}

async fn solo(
    State(state): State<AppState>,
    axum::extract::Path(seed): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&seed, &mut hasher);
    let seed_u64 = std::hash::Hasher::finish(&hasher);
    let mut rng = StdRng::seed_from_u64(seed_u64);
    let next_seed = rand::thread_rng().gen::<u32>().to_string();
    
    let mission = state.missions.missions.choose(&mut rng).unwrap().clone();
    
    let random_item_img = if mission.needs_random_item {
        let item = model::ITEMS.choose(&mut rng).unwrap();
        Some(format!("/assets/items/{}.png", item))
    } else { None };

    let random_loadout = if mission.needs_random_loadout {
        let c = model::CHARACTERS.choose(&mut rng).unwrap();
        let k = model::KARTS.choose(&mut rng).unwrap();
        let w = model::WHEELS.choose(&mut rng).unwrap();
        let g = model::GLIDERS.choose(&mut rng).unwrap();
        
        Some(Loadout {
            char_img: format!("/assets/characters/{}.webp", c.to_lowercase().replace(" ", "_")),
            char_name: c.to_string(),
            kart_img: format!("/assets/karts/{}.webp", k.to_lowercase().replace(" ", "_")),
            kart_name: k.to_string(),
            wheel_img: format!("/assets/wheels/{}.webp", w.to_lowercase().replace(" ", "_")),
            wheel_name: w.to_string(),
            glider_img: format!("/assets/gliders/{}.webp", g.to_lowercase().replace(" ", "_")),
            glider_name: g.to_string(),
        })
    } else { None };

    let random_number = mission.needs_random_number.map(|max| rng.gen_range(1..=max));

    let (nested_mission, nested_gacha_items) = if mission.needs_coop_singles {
         let sub = state.missions.coop_single.choose(&mut rng).cloned();
         let gacha = sub.as_ref().and_then(|m| {
             if m.needs_gacha_item_checklist {
                 Some(model::GACHA_ITEMS.iter().map(|s| format!("/assets/items/{}.png", s)).collect())
             } else { None }
         });
         (sub, gacha)
    } else { (None, None) };

    let template = SoloTemplate {
        seed,
        next_seed,
        mission,
        nested_mission,
        random_item_img,
        random_loadout,
        random_number,
        nested_gacha_items,
        view_name: "solo".to_string(),
    };
    render_response(headers, template.render().unwrap())
}

#[derive(Template)]
#[template(path = "partials/randomizer.html")]
struct RandomizerTemplate {
    seed: String,
    next_seed: String,
    loadout: Loadout,
    view_name: String,
}

async fn randomizer(
    axum::extract::Path(seed): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&seed, &mut hasher);
    let seed_u64 = std::hash::Hasher::finish(&hasher);
    let mut rng = StdRng::seed_from_u64(seed_u64);
    let next_seed = rand::thread_rng().gen::<u32>().to_string();
    
    let c = model::CHARACTERS.choose(&mut rng).unwrap();
    let k = model::KARTS.choose(&mut rng).unwrap();
    let w = model::WHEELS.choose(&mut rng).unwrap();
    let g = model::GLIDERS.choose(&mut rng).unwrap();
    
    let loadout = Loadout {
        char_img: format!("/assets/characters/{}.webp", c.to_lowercase().replace(" ", "_")),
        char_name: c.to_string(),
        kart_img: format!("/assets/karts/{}.webp", k.to_lowercase().replace(" ", "_")),
        kart_name: k.to_string(),
        wheel_img: format!("/assets/wheels/{}.webp", w.to_lowercase().replace(" ", "_")),
        wheel_name: w.to_string(),
        glider_img: format!("/assets/gliders/{}.webp", g.to_lowercase().replace(" ", "_")),
        glider_name: g.to_string(),
    };

    let template = RandomizerTemplate {
        seed,
        next_seed,
        loadout,
        view_name: "randomizer".to_string(),
    };
    render_response(headers, template.render().unwrap())
}

#[derive(Template)]
#[template(path = "partials/all_missions.html")]
struct AllMissionsTemplate<'a> {
    missions: &'a MissionsData,
}

async fn all_missions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let template = AllMissionsTemplate {
        missions: &state.missions,
    };
    render_response(headers, template.render().unwrap())
}

async fn mission_view(
    State(state): State<AppState>,
    axum::extract::Path((seed, name)): axum::extract::Path<(String, String)>,
    headers: HeaderMap,
) -> Html<String> {
    // Check main.rs imports for percent_encoding, if not present we might need to rely on direct match or add it
    // Assuming simple name match for now or re-adding helper function
    
    let mission = find_mission(&state.missions, &name);

    if let Some(mission) = mission {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&seed, &mut hasher);
        let seed_u64 = std::hash::Hasher::finish(&hasher);
        let mut rng = StdRng::seed_from_u64(seed_u64);
        let next_seed = rand::thread_rng().gen::<u32>().to_string();

        let random_item_img = if mission.needs_random_item {
            let item = model::ITEMS.choose(&mut rng).unwrap();
            Some(format!("/assets/items/{}.png", item))
        } else { None };

        let random_loadout = if mission.needs_random_loadout {
            let c = model::CHARACTERS.choose(&mut rng).unwrap();
            let k = model::KARTS.choose(&mut rng).unwrap();
            let w = model::WHEELS.choose(&mut rng).unwrap();
            let g = model::GLIDERS.choose(&mut rng).unwrap();
            
            Some(Loadout {
                char_img: format!("/assets/characters/{}.webp", c.to_lowercase().replace(" ", "_")),
                char_name: c.to_string(),
                kart_img: format!("/assets/karts/{}.webp", k.to_lowercase().replace(" ", "_")),
                kart_name: k.to_string(),
                wheel_img: format!("/assets/wheels/{}.webp", w.to_lowercase().replace(" ", "_")),
                wheel_name: w.to_string(),
                glider_img: format!("/assets/gliders/{}.webp", g.to_lowercase().replace(" ", "_")),
                glider_name: g.to_string(),
            })
        } else { None };

        let random_number = mission.needs_random_number.map(|max| rng.gen_range(1..=max));
        
        // Populate nested_mission for SoloTemplate just in case
        let (nested_mission, nested_gacha_items) = if mission.needs_coop_singles {
             let sub = state.missions.coop_single.choose(&mut rng).cloned();
             let gacha = sub.as_ref().and_then(|m| {
                 if m.needs_gacha_item_checklist {
                     Some(model::GACHA_ITEMS.iter().map(|s| format!("/assets/items/{}.png", s)).collect())
                 } else { None }
             });
             (sub, gacha)
        } else { (None, None) };

        let template = SoloTemplate {
            seed,
            next_seed,
            mission,
            nested_mission,
            random_item_img,
            random_loadout,
            random_number,
            nested_gacha_items,
            view_name: format!("mission/{}", percent_encoding::utf8_percent_encode(&name, percent_encoding::NON_ALPHANUMERIC).to_string()),
        };
        render_response(headers, template.render().unwrap())

    } else {
        Html("<h1>Mission Not Found</h1>".to_string())
    }
}

fn find_mission(data: &MissionsData, name: &str) -> Option<Mission> {
    // Simple lookup if encoding crate missing, otherwise use decode
    // We didn't re-add percent_encoding to imports in my recent 'Revert' step, 
    // but the previous Agent step 703 tried to add it. 
    // The USER removed it in 728.
    // I will try to use it assuming I can add the use statement or just do direct comparison if name is simple
    // Safer to decode.
    let name_decoded = percent_encoding::percent_decode_str(name).decode_utf8_lossy();
    
    data.missions.iter()
        .chain(data.coop_granprix.iter())
        .chain(data.coop_single.iter())
        .find(|m| m.name.eq_ignore_ascii_case(&name_decoded))
        .cloned()
}

// --- Traitor Mode Handlers & Structs ---

#[derive(Template)]
#[template(path = "traitor_setup.html")]
struct TraitorSetupTemplate;

async fn traitor_setup() -> Html<String> {
    let template = TraitorSetupTemplate;
    Html(template.render().unwrap())
}

async fn traitor_create(State(state): State<AppState>) -> Redirect {
    // Use a short random string for room ID
    let room_id: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(4)
        .map(char::from)
        .collect::<String>()
        .to_uppercase();

    let (tx, _rx) = broadcast::channel(100);

    let lobby = Lobby {
        id: room_id.clone(),
        players: Vec::new(),
        status: LobbyStatus::Waiting,
        seed: rand::thread_rng().gen::<u32>().to_string(),
        tx,
    };

    state.lobbies.write().unwrap().insert(room_id.clone(), lobby);
    
    Redirect::to(&format!("/traitor/{}/join", room_id))
}

#[derive(Deserialize)]
struct JoinForm {
    name: String,
}

#[derive(Template)]
#[template(path = "traitor_join.html")]
struct TraitorJoinTemplate {
    room_id: String,
    room_id_display: String,
}

async fn traitor_join_view(
    axum::extract::Path(room_id): axum::extract::Path<String>,
) ->  impl IntoResponse {
    // Simple validation could go here
    let template = TraitorJoinTemplate { 
        room_id: room_id.clone(),
        room_id_display: room_id,
    };
    Html(template.render().unwrap())
}

async fn traitor_join_action(
    State(state): State<AppState>,
    axum::extract::Path(room_id): axum::extract::Path<String>,
    Form(form): Form<JoinForm>,
) -> Response {
    // 1. Lock lobbies
    // 2. Add player if not started
    let mut lobbies = state.lobbies.write().unwrap();
    
    if let Some(lobby) = lobbies.get_mut(&room_id) {
        if lobby.status != LobbyStatus::Waiting {
             return (StatusCode::BAD_REQUEST, "Game already started").into_response();
        }
        
        // Check duplicate name? For now, allow duplicates or suffix them. 
        // Better to avoid confusion: simply allow.
        
        lobby.players.push(Player {
            name: form.name.clone(),
            is_traitor: false,
        });
        
        // Redirect to lobby + set cookie ideally, but for now we'll just redirect with query param or just simple flow
        // The user says "link to share... sign in... presses go".
        // How do we know WHICH player this browser is? 
        // We need a cookie or local storage. 
        // Simplest: Redirect to lobby with ?player=NAME. Weak security but fine for casual app.
        
        // Encode name for URL
        let encoded_name = percent_encoding::utf8_percent_encode(&form.name, percent_encoding::NON_ALPHANUMERIC).to_string();
        // Broadcast update
        let _ = lobby.tx.send(LobbyEvent::PlayerJoined(form.name.clone()));

        return Redirect::to(&format!("/traitor/{}?player={}", room_id, encoded_name)).into_response();
    }

    (StatusCode::NOT_FOUND, "Lobby not found").into_response()
}

#[derive(Template)]
#[template(path = "traitor_lobby.html")]
struct TraitorLobbyTemplate {
    room_id: String,
    player_name: String,
    players: Vec<Player>,
    is_started: bool,
}

#[derive(Deserialize)]
struct LobbyQuery {
    player: String,
}

async fn traitor_lobby_view(
    State(state): State<AppState>,
    axum::extract::Path(room_id): axum::extract::Path<String>,
    Query(params): Query<LobbyQuery>,
) -> impl IntoResponse {
    let lobbies = state.lobbies.read().unwrap();
    if let Some(lobby) = lobbies.get(&room_id) {
        if lobby.status == LobbyStatus::Started {
             // If started, redirect to role view
             // Redirect::to ...
             // But we need to keep the player name query param
             let encoded_name = percent_encoding::utf8_percent_encode(&params.player, percent_encoding::NON_ALPHANUMERIC).to_string();
             return Redirect::to(&format!("/traitor/{}/role?player={}", room_id, encoded_name)).into_response();
        }
        
        let template = TraitorLobbyTemplate {
            room_id,
            player_name: params.player.clone(),
            players: lobby.players.clone(),
            is_started: false,
        };
        return Html(template.render().unwrap()).into_response();
    }
    (StatusCode::NOT_FOUND, "Lobby not found").into_response()
}

// SSE Endpoint
async fn traitor_lobby_sse(
    State(state): State<AppState>,
    axum::extract::Path(room_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let lobbies = state.lobbies.read().unwrap();
    
    if let Some(lobby) = lobbies.get(&room_id) {
        let rx = lobby.tx.subscribe();
        let stream = BroadcastStream::new(rx);
        
        let stream = stream.map(|msg| {
             match msg {
                 Ok(LobbyEvent::PlayerJoined(name)) => {
                     let html = format!("<li class='bg-gray-50 p-3 rounded shadow-sm border flex items-center'><span class='font-medium'>{}</span></li>", name);
                     Ok::<Event, Infallible>(Event::default().event("player_joined").data(html))
                 },
                 Ok(LobbyEvent::GameStarted) => {
                     Ok::<Event, Infallible>(Event::default().event("game_start").data("started"))
                 },
                 Err(_) => Ok::<Event, Infallible>(Event::default()) // Ignore lag errors
             }
        });

        Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default()).into_response()
    } else {
        let (_, rx) = broadcast::channel::<LobbyEvent>(1);
        let stream = BroadcastStream::new(rx).map(|_| Ok::<Event, Infallible>(Event::default()));
        Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default()).into_response()
    }
}

async fn traitor_start(
    State(state): State<AppState>,
    axum::extract::Path(room_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let mut lobbies = state.lobbies.write().unwrap();
    if let Some(lobby) = lobbies.get_mut(&room_id) {
        if lobby.players.len() < 1 { // Allow 1 for testing, though traitor implies >1
             return (StatusCode::BAD_REQUEST, "Not enough players").into_response();
        }
        if lobby.status == LobbyStatus::Started {
            return (StatusCode::OK, "Already started").into_response();
        }

        // Assign Traitor
        let traitor_idx = rand::thread_rng().gen_range(0..lobby.players.len());
        lobby.players[traitor_idx].is_traitor = true;
        lobby.status = LobbyStatus::Started;
        
        let _ = lobby.tx.send(LobbyEvent::GameStarted);
        
        // Return 200 OK
        return StatusCode::OK.into_response();
    }
    (StatusCode::NOT_FOUND, "Lobby not found").into_response()
}

#[derive(Template)]
#[template(path = "traitor_role.html")]
struct TraitorRoleTemplate {
    is_traitor: bool,
    role_name: String,
    player_name: String,
    mission: Mission,
}

async fn traitor_role_view(
    State(state): State<AppState>,
    axum::extract::Path(room_id): axum::extract::Path<String>,
    Query(params): Query<LobbyQuery>,
) -> impl IntoResponse {
    let lobbies = state.lobbies.read().unwrap();
    if let Some(lobby) = lobbies.get(&room_id) {
        // Find player
        // Note: players have names. We trust query param.
        if let Some(player) = lobby.players.iter().find(|p| p.name == params.player) {
            
            // Get mission using lobby seed
             let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&lobby.seed, &mut hasher);
            let seed_u64 = std::hash::Hasher::finish(&hasher);
            let mut rng = StdRng::seed_from_u64(seed_u64);
            
            // Traitor mode uses coop missions "for now we can just use the coop missions"
            let mission = state.missions.coop_granprix.choose(&mut rng).unwrap().clone();
            
            let template = TraitorRoleTemplate {
                is_traitor: player.is_traitor,
                role_name: if player.is_traitor { "Traitor".to_string() } else { "Innocent".to_string() },
                player_name: player.name.clone(),
                mission,
            };
            return Html(template.render().unwrap()).into_response();
        }
        return (StatusCode::FORBIDDEN, "Player not in lobby").into_response();
    }
    (StatusCode::NOT_FOUND, "Lobby not found").into_response()
}
