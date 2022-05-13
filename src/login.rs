// Endpoints to handle login/logout
use crate::models;
use crate::models::APIError;
use crate::converters;
use actix_web::cookie::time::Duration as CookieDuration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::header::HeaderValue;
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use log::error;
use std::collections::HashMap;
use std::time::Duration;
use std::sync::Arc;
use uuid::Uuid;
use sha2::Sha512;
use hmac::{Hmac, Mac};

type HmacSha512 = Hmac<Sha512>;

/// Returns the oauth2 link to use for login
#[get("/oauth2")]
async fn get_oauth2(req: HttpRequest) -> HttpResponse {
    let data: &models::AppState = req.app_data::<web::Data<models::AppState>>().unwrap();
    let state = Uuid::new_v4().to_hyphenated().to_string();
    let auth_default = &HeaderValue::from_str("").unwrap();
    let url = format!(
        "https://discord.com/oauth2/authorize?client_id={discord_client_id}&redirect_uri={redirect_url_domain}/frostpaw/login&scope=identify&response_type=code",
        discord_client_id = data.config.secrets.client_id,
        redirect_url_domain = req.headers().get("Frostpaw-Server").unwrap_or(auth_default).to_str().unwrap(),
    );
    HttpResponse::Ok().json(models::APIResponse {
        done: true,
        reason: Some(state),
        context: Some(url),
    })
}

/// Get client info
#[get("/frostpaw/clients/{client_id}")]
async fn get_frostpaw_client(req: HttpRequest, client_id: web::Path<String>) -> HttpResponse {
    let data: &models::AppState = req.app_data::<web::Data<models::AppState>>().unwrap();
    let client = data.database.get_frostpaw_client(&client_id.into_inner()).await;
    if let Some(cli) = client {
        HttpResponse::Ok().json(cli)
    } else {
        HttpResponse::NotFound().json(models::APIResponse::err_small(&models::GenericError::NotFound))
    }
}

/// Regenerate access token from refresh token
#[post("/frostpaw/clients/{client_id}/refresh")]
async fn refresh_access_token(req: HttpRequest, client_id: web::Path<String>, info: web::Json<models::FrostpawTokenReset>) -> HttpResponse {
    let data: &models::AppState = req.app_data::<web::Data<models::AppState>>().unwrap();
    let client = data.database.get_frostpaw_client(&client_id).await;

    if client.is_none() {        
        return HttpResponse::NotFound().json(models::APIResponse {
            done: false,
            reason: Some("Client not found".to_string()),
            context: None,
        });
    }

    let client = client.unwrap();
    
    if client.secret != info.secret {
        return HttpResponse::Unauthorized().json(models::APIResponse {
            done: false,
            reason: Some("Invalid client secret".to_string()),
            context: None,
        });
    }

    let refresh_data = data.database.get_frostpaw_refresh_token(info.refresh_token.clone()).await;

    if refresh_data.client.id != client_id.into_inner() {
        return HttpResponse::BadRequest().json(models::APIResponse {
            done: false,
            reason: Some("Invalid client ID for this refresh token".to_string()),
            context: None,
        });
    }

    // Invalidate all other tokens
    for (key, value) in data.database.client_data.iter() {
        if value.user_id == refresh_data.user_id && value.client_id == refresh_data.client.id {
            data.database.client_data.invalidate(&key).await;
        }
    }

    let access_token = "Frostpaw.".to_string() + &converters::create_token(64);

    data.database.client_data.insert(access_token.clone(), Arc::new(models::FrostpawLogin {
        client_id: client.id,
        user_id: refresh_data.user_id,
        token: data.database.get_user_token(refresh_data.user_id).await,
    })).await;

    HttpResponse::Ok().json(models::APIResponse {
        done: true,
        reason: None,
        context: Some(access_token),
    })
}

/// Creates a oauth2 login
#[post("/oauth2")]
async fn do_oauth2(req: HttpRequest, info: web::Json<models::OauthDoQuery>) -> HttpResponse {
    // Get code
    let code = info.code.clone();
    let data: &models::AppState = req.app_data::<web::Data<models::AppState>>().unwrap();

    let auth_default = &HeaderValue::from_str("").unwrap();

    let redirect_url_domain = req
        .headers()
        .get("Frostpaw-Server")
        .unwrap_or(auth_default)
        .to_str()
        .unwrap();

    let valid_domains = vec!["https://fateslist.xyz", "https://www.fateslist.xyz", "https://sunbeam.fateslist.xyz"];
    
    
    if !valid_domains.contains(&redirect_url_domain) {
        return HttpResponse::BadRequest().json(models::APIResponse {
            done: false,
            reason: Some("Frostpaw-Server is not in valid format, perhaps your client isn't setting the header properly?".to_string()),
            context: Some("Frostpaw-Server".to_string())
        });
    }

    let redirect_uri = format!("{}/frostpaw/login", redirect_url_domain);

    let login = login_user(data, code, redirect_uri).await;

    match login {
        Err(err) => {
            error!("{:?}", err.error());
            HttpResponse::BadRequest().json(models::APIResponse::err_small(&err))
        }
        Ok(mut user) => {
            // Check for a frostpaw login
            if info.frostpaw {
                // If a frostpaw custom client login is used 
                // Check claw with blood
                if info.frostpaw_blood.is_none() || info.frostpaw_claw.is_none() || info.frostpaw_claw_unseathe_time.is_none() {
                    return HttpResponse::BadRequest().json(models::APIResponse::err_small(&models::GenericError::InvalidFields));
                }

                // These clones arent easy to avoid
                let blood = info.frostpaw_blood.clone().unwrap();
                let claw = info.frostpaw_claw.clone().unwrap();
                let claw_unseathe_time = info.frostpaw_claw_unseathe_time.unwrap();

                let time_elapsed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() - claw_unseathe_time;

                if !(5..=75).contains(&time_elapsed) {
                    return HttpResponse::BadRequest().json(models::APIResponse::err_small(&models::OauthError::NonceTooOld));
                }

                let client = data.database.get_frostpaw_client(&blood).await;
                if client.is_none() {
                    return HttpResponse::BadRequest().json(models::APIResponse {
                        done: false,
                        reason: Some("Frostpaw login requires a valid client ID".to_string()),
                        context: None,
                    });
                }
                let client = client.unwrap();

                // Now check HMAC
                let mac = HmacSha512::new_from_slice(client.secret.as_bytes());
    
                if mac.is_err() {
                    error!("Failed to create HMAC");
                    return HttpResponse::BadRequest().json(models::APIResponse {
                        done: false,
                        reason: Some("Failed to create HMAC".to_string()),
                        context: None,
                    });
                }
        
                let mut mac = mac.unwrap();        

                mac.update(claw_unseathe_time.to_string().as_bytes());

                let expected_hmac = hex::encode(mac.finalize().into_bytes());

                if expected_hmac != claw {
                    return HttpResponse::BadRequest().json(models::APIResponse {
                        done: false,
                        reason: Some(format!("Expected HMAC of {expected} but got {got}", expected = expected_hmac, got = claw)),
                        context: None,
                    });
                }

                // OK, now that we are reasonably confident about client, we can create the frostpaw login
                let access_token = "Frostpaw.".to_string() + &converters::create_token(64);
                
                data.database.client_data.insert(access_token.clone(), Arc::new(models::FrostpawLogin {
                    client_id: client.id,
                    user_id: user.user.id.parse().unwrap(),
                    token: user.token,
                })).await;

                // Put new access token and refresh token in user struct
                user.token = access_token.clone();
                user.refresh_token = Some(data.database.add_refresh_token(&blood, user.user.id.parse().unwrap()).await);

                return HttpResponse::Ok().json(user);
            }

            let cookie_val = base64::encode(serde_json::to_string(&user).unwrap());
            let sunbeam_cookie = Cookie::build("sunbeam-session:warriorcats", cookie_val)
                .path("/")
                .domain("fateslist.xyz")
                .secure(true)
                .http_only(true)
                .max_age(CookieDuration::new(60 * 60 * 8, 0))
                .same_site(SameSite::Strict)
                .finish();
            return HttpResponse::Ok().cookie(sunbeam_cookie).json(user);
        }
    }
}

/// 'Deletes' (logs out) a oauth2 login
#[delete("/oauth2")]
async fn del_oauth2(_req: HttpRequest) -> HttpResponse {
    let sunbeam_cookie = Cookie::build("sunbeam-session:warriorcats", "")
        .path("/")
        .domain("fateslist.xyz")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict)
        .finish();

    let mut resp = HttpResponse::Ok().json(models::APIResponse::ok());
    let cookie_del = resp.add_removal_cookie(&sunbeam_cookie);

    match cookie_del {
        Err(err) => {
            error!("{}", err);
        }
        _ => {}
    }

    resp
}

/// Actual Oauth2 impl
async fn login_user(
    data: &models::AppState,
    code: String,
    redirect_url: String,
) -> Result<models::OauthUserLogin, models::OauthError> {
    let mut params = HashMap::new();
    params.insert("client_id", data.config.secrets.client_id.clone());
    params.insert("client_secret", data.config.secrets.client_secret.clone());
    params.insert("grant_type", "authorization_code".to_string());
    params.insert("code", code);
    params.insert("redirect_uri", redirect_url);

    let access_token_exchange = data
        .requests
        .post("https://discord.com/api/v10/oauth2/token")
        .timeout(Duration::from_secs(10))
        .form(&params)
        .send()
        .await
        .map_err(models::OauthError::BadExchange)?;

    if !access_token_exchange.status().is_success() {
        let json = access_token_exchange
            .text()
            .await
            .map_err(models::OauthError::BadExchange)?;
        return Err(models::OauthError::BadExchangeJson(json));
    }

    let json = access_token_exchange
        .json::<models::OauthAccessTokenResponse>()
        .await
        .map_err(models::OauthError::BadExchange)?;

    let user_exchange = data
        .requests
        .get("https://discord.com/api/v10/users/@me")
        .bearer_auth(json.access_token)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(models::OauthError::NoUser)?;

    if !user_exchange.status().is_success() {
        let json = user_exchange
            .text()
            .await
            .map_err(models::OauthError::BadExchange)?;
        return Err(models::OauthError::BadExchangeJson(json));
    }

    let json = user_exchange
        .json::<models::OauthUser>()
        .await
        .map_err(models::OauthError::NoUser)?;

    let data = data
        .database
        .create_user_oauth(json)
        .await
        .map_err(models::OauthError::SQLError)?;

    Ok(data)
}
