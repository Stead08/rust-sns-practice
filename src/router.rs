use crate::models::prelude::{Sessions, Users};
use crate::models::sessions::ActiveModel as SessionModel;
use crate::models::sessions::Entity as SessionEntity;
use crate::models::users::{ActiveModel as UserModel, Model};
use crate::models::{sessions, users};
use axum::extract::State;
use axum::headers::HeaderValue;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, ORIGIN};
use axum::http::{Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{middleware, Json, Router};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::PrivateCookieJar;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    sea_query, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, NotSet, Order,
    QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use time::Duration;

use crate::{models, AppState};
use tower_http::cors::CorsLayer;

pub fn create_router(state: AppState) -> Router {
    let api_router = api_router(state);

    //APIルーターを/apiにネスト
    Router::new().nest("/api", api_router)
}

pub fn api_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![ACCEPT, AUTHORIZATION, ORIGIN, CONTENT_TYPE])
        .allow_origin(state.domain.parse::<HeaderValue>().unwrap());

    let auth_router = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", get(logout))
        .route("/me", get(get_me))
        .route("/check", get(auth_check));

    let content_router = Router::new()
        .route("/user_tweets", post(create_tweet))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            validate_session,
        ));

    let timeline_router =
        Router::new()
            .route("/", get(get_timeline))
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                validate_session,
            ));

    let relation_router = Router::new()
        .route("/follow", post(create_follow))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            validate_session,
        ));
    Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth_router)
        .nest("/content", content_router)
        .nest("/timeline", timeline_router)
        .nest("/relation", relation_router)
        .with_state(state)
        .layer(cors)
}

pub async fn health_check() -> Response {
    (StatusCode::OK, "OK!").into_response()
}

pub async fn validate_session<B>(
    jar: PrivateCookieJar,
    State(state): State<AppState>,
    // Request<B> と Next<B> は axum の関数からのミドルウェアに必要な型
    request: Request<B>,
    next: Next<B>,
) -> (PrivateCookieJar, Response) {
    //cookieの取得を試みる、できなかったら403を返す
    let Some(cookie) = jar.get("foo").map(|cookie| cookie.value().to_owned()) else {
        println!("{:?} Could not find a cookie in jar", jar);
        return (jar, (StatusCode::FORBIDDEN, "ログインしてください".to_string()).into_response());
    };

    //作成されたセッションを見つける
    let find_session = Sessions::find()
        .filter(sessions::Column::SessionId.eq(cookie))
        .one(&state.postgres)
        .await;
    //セッションが見つからなかったら、403を返す
    match find_session {
        Ok(_) => (jar, next.run(request).await),
        Err(_) => (
            jar,
            (StatusCode::FORBIDDEN, "ログインしてください".to_string()).into_response(),
        ),
    }
}

pub async fn auth_check(State(state): State<AppState>, jar: PrivateCookieJar) -> impl IntoResponse {
    let Some(cookie) = jar.get("foo").map(|cookie| cookie.value().to_owned()) else {
        println!("{:?} Could not find a cookie in jar", jar);
        return (StatusCode::FORBIDDEN, "ログインしてください".to_string()).into_response();
    };

    let find_session = Sessions::find()
        .filter(sessions::Column::SessionId.eq(cookie))
        .one(&state.postgres)
        .await;

    match find_session {
        Ok(session) => match session {
            Some(_) => (StatusCode::OK, "ログインしています".to_string()).into_response(),
            None => (StatusCode::FORBIDDEN, "ログインしていません".to_string()).into_response(),
        },
        Err(_) => (StatusCode::FORBIDDEN, "ログインしていません".to_string()).into_response(),
    }
}

#[derive(Deserialize)]
pub struct RegisterDetails {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginDetails {
    username: String,
    password: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(new_user): Json<RegisterDetails>,
) -> impl IntoResponse {
    //空パスワードを回避する。ログイン時はハッシュ化されたパスワードを検証。
    let hashed_password = bcrypt::hash(new_user.password, 10).unwrap();

    let user = UserModel {
        id: Default::default(),
        username: Set(new_user.username),
        email: Set(new_user.email),
        password: Set(hashed_password),
        createdat: NotSet,
    };

    let res = user.insert(&state.postgres);

    //作成成功したら Created status code, 失敗したら Internal Server Error status code を返す。
    match res.await {
        Ok(_) => (StatusCode::CREATED, "作成されました".to_string()).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("作成できませんでした: {}", e),
        )
            .into_response(),
    }
}

pub async fn login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Json(login): Json<LoginDetails>,
) -> Result<(PrivateCookieJar, StatusCode), StatusCode> {
    let user = Users::find()
        .filter(users::Column::Username.eq(&login.username))
        .one(&state.postgres)
        .await;

    match user {
        Ok(Some(user)) => {
            // bcryptがハッシュ値を認証できなかったら、BAD_REQUESTエラーを返す。
            match bcrypt::verify(&login.password, &user.password) {
                Ok(bool) => {
                    if !bool {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                }
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            }

            // ランダムセッションIDを生成し、ハッシュマップエントリーに追加
            let session_id = rand::random::<u64>().to_string();

            let session = SessionModel {
                id: Default::default(),
                session_id: Set(session_id.clone()),
                user_id: Set(user.id),
            };

            let result = SessionEntity::insert(session)
                .on_conflict(
                    // on conflict do update
                    sea_query::OnConflict::column(sessions::Column::UserId)
                        .update_column(sessions::Column::SessionId)
                        .to_owned(),
                )
                .exec(&state.postgres);

            match result.await {
                Ok(_) => {
                    let cookie = Cookie::build("foo", session_id)
                        .secure(false)
                        .same_site(SameSite::Lax)
                        .http_only(true)
                        .path("/")
                        .max_age(Duration::WEEK)
                        .finish();
                    // ステータスコード200とクッキーを返す。
                    Ok((jar.add(cookie), StatusCode::OK))
                }
                Err(e) => {
                    eprintln!("An error occurred: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Err(StatusCode::BAD_REQUEST),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn logout(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<PrivateCookieJar, StatusCode> {
    let Some(cookie) = jar.get("foo").map(|cookie| cookie.value().to_owned()) else {
        return Ok(jar);
    };

    //削除対象を取得する
    let target = models::sessions::Entity::find()
        .filter(sessions::Column::SessionId.eq(cookie))
        .one(&state.postgres)
        .await;

    match target {
        Ok(target) => {
            //ActiveModelを取得する
            let delete_row: models::sessions::ActiveModel = target.unwrap().into_active_model();
            let delete_result = delete_row.delete(&state.postgres).await;
            match delete_result {
                Ok(_) => Ok(jar.remove(Cookie::named("foo"))),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

#[derive(Deserialize)]
pub struct NewTweet {
    content: String,
}
pub async fn create_tweet(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Json(new_tweet): Json<NewTweet>,
) -> impl IntoResponse {
    let Some(cookie) = jar.get("foo").map(|cookie| cookie.value().to_owned()) else {
        return (StatusCode::FORBIDDEN, "ログインしてください".to_string()).into_response();
    };

    let target = models::sessions::Entity::find()
        .filter(sessions::Column::SessionId.eq(cookie))
        .one(&state.postgres)
        .await;

    match target {
        Ok(target) => {
            let user_id = target.unwrap().user_id;
            let tweet = models::user_tweets::ActiveModel {
                id: Default::default(),
                user_id: Set(user_id),
                content: Set(Some(new_tweet.content)),
                created_at: Default::default(),
            };
            let res = tweet.insert(&state.postgres).await;
            match res {
                Ok(_) => (StatusCode::CREATED, "作成されました".to_string()).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("作成できませんでした: {}", e),
                )
                    .into_response(),
            }
        }
        Err(_) => (StatusCode::BAD_REQUEST).into_response(),
    }
}

pub async fn get_me(State(state): State<AppState>, jar: PrivateCookieJar) -> impl IntoResponse {
    let Some(cookie) = jar.get("foo").map(|cookie| cookie.value().to_owned()) else {
        return (StatusCode::FORBIDDEN, "ログインしてください".to_string()).into_response();
    };

    let target = models::sessions::Entity::find()
        .filter(models::sessions::Column::SessionId.eq(cookie))
        .one(&state.postgres)
        .await;

    match target {
        Ok(target) => {
            let user_id = target.unwrap().user_id;
            let user = models::users::Entity::find()
                .filter(models::users::Column::Id.eq(user_id))
                .one(&state.postgres)
                .await;
            match user {
                Ok(user) => {
                    let user = user.unwrap();
                    let user = Model {
                        id: user.id,
                        username: user.username,
                        email: user.email,
                        password: "".to_string(),
                        createdat: user.createdat,
                    };
                    Json(user).into_response()
                }
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            }
        }
        Err(_) => (StatusCode::BAD_REQUEST).into_response(),
    }
}
#[derive(Serialize)]
struct Tweet {
    id: i32,
    username: String,
    content: String,
    created_at: Option<DateTimeWithTimeZone>,
}
pub async fn get_timeline(State(state): State<AppState>) -> impl IntoResponse {
    let tweets = models::user_tweets::Entity::find()
        .order_by(models::user_tweets::Column::CreatedAt, Order::Desc)
        .limit(100)
        .all(&state.postgres)
        .await;

    match tweets {
        Ok(tweets) => {
            let mut timeline = Vec::new();
            for tweet in tweets {
                let user = models::users::Entity::find()
                    .filter(models::users::Column::Id.eq(tweet.user_id))
                    .one(&state.postgres)
                    .await;
                match user {
                    Ok(user) => {
                        let user = user.unwrap();
                        let tweet = Tweet {
                            id: tweet.id,
                            username: user.username,
                            content: tweet.content.unwrap(),
                            created_at: tweet.created_at,
                        };
                        timeline.push(tweet);
                    }
                    Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
                }
            }
            Json(timeline).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}
#[derive(Deserialize)]
pub struct Follow {
    username: String,
}

pub async fn create_follow(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Json(follow): Json<Follow>,
) -> impl IntoResponse {
    let target = models::users::Entity::find()
        .filter(models::users::Column::Username.eq(follow.username))
        .one(&state.postgres)
        .await;

    // Cookieを取得する
    let Some(cookie) = jar.get("foo").map(|cookie| cookie.value().to_owned()) else {
        return (StatusCode::FORBIDDEN, "ログインしてください".to_string()).into_response();
    };
    // フォローする人のIDを取得する
    let follower = models::sessions::Entity::find()
        .filter(models::sessions::Column::SessionId.eq(cookie))
        .one(&state.postgres)
        .await
        .expect("failed to get session");
    match target {
        Ok(target) => {
            let target = target.unwrap();
            let follower = follower.unwrap();
            let follow = models::follow_relations::ActiveModel {
                id: Default::default(),
                followee_id: Set(target.id as i64),
                follower_id: Set(follower.user_id as i64),
            };
            let res = follow.insert(&state.postgres).await;
            match res {
                Ok(_) => (StatusCode::CREATED, "フォローしました".to_string()).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("フォローできませんでした: {}", e),
                )
                    .into_response(),
            }
        }
        Err(_) => (StatusCode::BAD_REQUEST).into_response(),
    }
}
