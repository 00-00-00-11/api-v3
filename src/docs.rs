use serde::Serialize;
use crate::models;
use bevy_reflect::{Reflect, Struct};

fn _get_value_from(
    value: &dyn Reflect,
) -> String {
    let mut field_name_ext: String = value.type_name().to_string();    

    // type_name replacer
    field_name_ext = field_name_ext.replace("core::option::Option", "Optional ");
    field_name_ext = field_name_ext.replace("alloc::string::", "");

    // Optional string case
    if let Some(value) = value.downcast_ref::<Option<String>>() {
        match value {
            Some(value) => {
                field_name_ext = "String? ".to_string() + "| default = " + value;
            },
            None => {
                // Ignored
            },
        }
    }

    // Optional i64 case
    if let Some(value) = value.downcast_ref::<Option<i64>>() {
        match value {
            Some(value) => {
                field_name_ext = "i64? ".to_string() + "| default = " + &value.to_string();
            },
            None => {
                // Ignored
            },
        }
    }    

    "[".to_owned() + &field_name_ext + " (type info may be incomplete, see example)]"
}

fn _get_params<T: Struct>(
    params: &T,
) -> String {
    let mut params_string = String::new();
    for (i, value) in params.iter_fields().enumerate() {
        let field_name: String = params.name_at(i).unwrap().to_string();
        let field_value = _get_value_from(value);
        params_string += &format!(
            "- **{field_name}** {field_value}\n",
            field_name = field_name,
            field_value = field_value,
        )
    }
    params_string
}

fn doc<T: Serialize, T2: Serialize, T3: Struct + Serialize, T4: Struct + Serialize>(
    route: models::Route<T, T2, T3, T4>,
) -> String {
    // Serialize request body
    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
    
    route.request_body.serialize(&mut ser).unwrap();

    // Serialize response body
    let buf2 = Vec::new();
    let formatter2 = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser2 = serde_json::Serializer::with_formatter(buf2, formatter2);

    route.response_body.serialize(&mut ser2).unwrap();

    // Serialize query parameters
    let buf4 = Vec::new();
    let formatter4 = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser4 = serde_json::Serializer::with_formatter(buf4, formatter4);

    let mut query_params_str = _get_params(route.query_params);

    route.query_params.serialize(&mut ser4).unwrap();

    let query_params_json = &String::from_utf8(ser4.into_inner()).unwrap();

    query_params_str += &("\n\n**Example**\n\n```json\n".to_string() + &query_params_json.clone() + "\n```");

    // Serialize path parameters
    let buf3 = Vec::new();
    let formatter3 = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser3 = serde_json::Serializer::with_formatter(buf3, formatter3);

    let mut path_params_str = _get_params(route.path_params);

    route.path_params.serialize(&mut ser3).unwrap();

    let path_params_json = &String::from_utf8(ser3.into_inner()).unwrap();

    path_params_str += &("\n\n**Example**\n\n```json\n".to_string() + &path_params_json.clone() + "\n```");

    let mut base_doc = format!(
        "### {title}\n#### {method} {path}\n\n{description}\n\n**API v2 analogue:** {equiv_v2_route}",
        title = route.title,
        method = route.method,
        path = route.path,
        description = route.description,
        equiv_v2_route = route.equiv_v2_route,
    );

    if path_params_json.len() > 2 {
        base_doc += &("\n\n**Path parameters**\n\n".to_string() + &path_params_str);
    }
    if query_params_json.len() > 2 {
        base_doc += &("\n\n**Query parameters**\n\n".to_string() + &query_params_str);
    }

    return base_doc + &format!(
        "\n\n**Request Body**\n\n```json\n{request_body}\n```\n\n**Response Body**\n\n```json\n{response_body}\n```\n\n\n",
        request_body = String::from_utf8(ser.into_inner()).unwrap(),
        response_body = String::from_utf8(ser2.into_inner()).unwrap(),
    );
}

/// Begin a new doc category
fn doc_category(name: &str) -> String {
    format!(
        "## {name}\n\n",
        name = name,
    )
}

pub fn document_routes() -> String {
    let mut docs: String = "# API v3\n**API URL**: ``https://next.fateslist.xyz`` *or* ``https://api.fateslist.xyz`` (for now, can change in future)\n".to_string();

    // Add basic auth stuff
    docs += r#"
## Authorization

- **Bot:** These endpoints require a bot token. 
You can get this from Bot Settings. Make sure to keep this safe and in 
a .gitignore/.env. A prefix of `Bot` before the bot token such as 
`Bot abcdef` is supported and can be used to avoid ambiguity but is not 
required. The default auth scheme if no prefix is given depends on the
endpoint: Endpoints which have only one auth scheme will use that auth 
scheme while endpoints with multiple will always use `Bot` for 
backward compatibility

- **Server:** These endpoints require a server
token which you can get using ``/get API Token`` in your server. 
Same warnings and information from the other authentication types 
apply here. A prefix of ``Server`` before the server token is 
supported and can be used to avoid ambiguity but is not required.

- **User:** These endpoints require a user token. You can get this 
from your profile under the User Token section. If you are using this 
for voting, make sure to allow users to opt out! A prefix of `User` 
before the user token such as `User abcdef` is supported and can be 
used to avoid ambiguity but is not required outside of endpoints that 
have both a user and a bot authentication option such as Get Votes. 
In such endpoints, the default will always be a bot auth unless 
you prefix the token with `User`
"#;

    // API Response route
    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
    
    models::APIResponse {
        done: true,
        reason: Some("Reason for success of failure, can be null".to_string()),
        context: Some("Any extra context".to_string()),
    }.serialize(&mut ser).unwrap();

    docs += &("\n## Base Response\n\nA default API Response will be of the below format:\n\n```json\n".to_string() + &String::from_utf8(ser.into_inner()).unwrap() + "\n```\n\n");

    // TODO: For each route, add doc system

    docs += &doc_category("Core");

    // - Index route
    let index_bots = vec![models::IndexBot::default()];

    let tags = vec![models::Tag::default()];

    let features = vec![models::Feature::default()];

    docs += &doc(models::Route {
        title: "Index",
        method: "GET",
        path: "/index",
        path_params: &models::Empty {},
        query_params: &models::IndexQuery {
            target_type: Some("bot".to_string()),
        },
        description: "Returns the index for bots and servers",
        request_body: &models::Empty {},
        response_body: &models::Index {
            top_voted: index_bots.clone(),
            certified: index_bots.clone(),
            new: index_bots, // Clone later if needed
            tags: tags.clone(),
            features: features.clone(),
        },
        equiv_v2_route: "(no longer working) [Get Index](https://legacy.fateslist.xyz/docs/redoc#operation/get_index)",
    });


    // - Vanity route
    docs += &doc( models::Route {
        title: "Resolve Vanity",
        method: "GET",
        path: "/code/{code}",
        path_params: &models::VanityPath {
            code: "my-vanity".to_string(),
        },
        query_params: &models::Empty {},
        description: "Resolves the vanity for a bot/server in the list",
        request_body: &models::Empty {},
        response_body: &models::Vanity {
            target_id: "0000000000".to_string(),
            target_type: "bot | server".to_string(),
        },
        equiv_v2_route: "(no longer working) [Get Vanity](https://legacy.fateslist.xyz/docs/redoc#operation/get_vanity)",
    });

    // - Policies route
    docs += &doc( models::Route {
        title: "Get Policies",
        method: "GET",
        path: "/policies",
        path_params: &models::Empty {},
        query_params: &models::Empty {},
        description: "Get policies (rules, privacy policy, terms of service)",
        request_body: &models::Empty {},
        response_body: &models::Policies::default(),
        equiv_v2_route: "(no longer working) [All Policies](https://legacy.fateslist.xyz/api/docs/redoc#operation/all_policies)",
    });

    // - Fetch Bot route
    docs += &doc( models::Route {
        title: "Get Bot",
        method: "GET",
        path: "/bots/{id}",
        path_params: &models::FetchBotPath::default(),
        query_params: &models::FetchBotQuery::default(),
description: r#"
Fetches bot information given a bot ID. If not found, 404 will be returned. 

This endpoint handles both bot IDs and client IDs

Differences from API v2:

- Unlike API v2, this does not support compact or no_cache. Owner order is also guaranteed
- *``long_description/css`` is sanitized with ammonia by default, use `long_description_raw` if you want the unsanitized version*
- All responses are cached for a short period of time. There is *no* way to opt out unlike API v2
- Some fields have been renamed or removed (such as ``promos`` which may be readded at a later date)

This API returns some empty fields such as ``webhook``, ``webhook_secret``, `api_token`` and more. 
This is to allow reuse of the Bot struct in Get Bot Settings which does contain this sensitive data. 

**Set the Frostpaw header if you are a custom client**
"#,
    request_body: &models::Empty{},
    response_body: &models::Bot::default(), // TODO
    equiv_v2_route: "[Fetch Bot](https://legacy.fateslist.xyz/docs/redoc#operation/fetch_bot)",
    });

    // - Search List route
    docs += &doc(models::Route {
        title: "Search List",
        method: "GET",
        path: "/search?q={query}",
        path_params: &models::Empty {},
        query_params: &models::SearchQuery {
            q: Some("mew".to_string()),
        },
        description: r#"Searches the list based on a query named ``q``"#,
        request_body: &models::Empty {},
        response_body: &models::Search {
            bots: vec![models::IndexBot::default()],
            servers: vec![models::IndexBot::default()],
            packs: vec![models::BotPack::default()],
            profiles: vec![models::SearchProfile::default()],
            tags: models::SearchTags {
                bots: vec![models::Tag::default()],
                servers: vec![models::Tag::default()]
            },
        },
        equiv_v2_route: "(no longer working) [Fetch Bot](https://legacy.fateslist.xyz/docs/redoc#operation/search_list)",
    });

    docs += &doc(
        models::Route {
            title: "Random Bot",
            method: "GET",
            path: "/random-bot",
            path_params: &models::Empty {},
            query_params: &models::Empty {},
            request_body: &models::Empty {},
            response_body: &models::IndexBot::default(),
description: r#"
Fetches a random bot on the list

Example:
```py
import requests

def random_bot():
    res = requests.get(api_url"/random-bot")
    json = res.json()
    if res.status != 200:
        # Handle an error in the api
        ...
    return json
```
"#,
            equiv_v2_route: "(no longer working) [Fetch Random Bot](https://legacy.fateslist.xyz/api/docs/redoc#operation/fetch_random_bot)",
    });

    docs += &doc(
        models::Route {
            title: "Random Server",
            method: "GET",
            path: "/random-server",
            path_params: &models::Empty {},
            query_params: &models::Empty {},
            request_body: &models::Empty {},
            response_body: &models::IndexBot::default(),
description: r#"
Fetches a random server on the list

Example:
```py
import requests

def random_server():
    res = requests.get(api_url"/random-server")
    json = res.json()
    if res.status != 200:
        # Handle an error in the api
        ...
    return json
```
"#,
            equiv_v2_route: "(no longer working) [Fetch Random Server](https://legacy.fateslist.xyz/api/docs/redoc#operation/fetch_random_server)",
    });

    docs += &doc( models::Route {
        title: "Get Server",
        method: "GET",
        path: "/servers/{id}",
        path_params: &models::FetchBotPath::default(),
        query_params: &models::FetchBotQuery::default(),
description: r#"
Fetches server information given a server/guild ID. If not found, 404 will be returned. 

Differences from API v2:

- Unlike API v2, this does not support compact or no_cache.
- *``long_description/css`` is sanitized with ammonia by default, use `long_description_raw` if you want the unsanitized version*
- All responses are cached for a short period of time. There is *no* way to opt out unlike API v2
- Some fields have been renamed or removed

**Set the Frostpaw header if you are a custom client**
"#,
    request_body: &models::Empty{},
    response_body: &models::Server::default(),
    equiv_v2_route: "(no longer working) [Fetch Server](https://legacy.fateslist.xyz/docs/redoc#operation/fetch_server)",
    });

    // - Get User Votes
    docs += &doc( models::Route {
        title: "Get User Votes",
        method: "GET",
        path: "/users/{user_id}/bots/{bot_id}/votes",
        path_params: &models::GetUserBotPath {
            user_id: 0,
            bot_id: 0,
        },
        query_params: &models::Empty {},
description: r#"
Endpoint to check amount of votes a user has.

- votes | The amount of votes the bot has.
- voted | Whether or not the user has *ever* voted for the bot.
- vote_epoch | The redis TTL of the users vote lock. This is not time_to_vote which is the
elapsed time the user has waited since their last vote.
- timestamps | A list of timestamps that the user has voted for the bot on that has been recorded.
- time_to_vote | The time the user has waited since they last voted.
- vote_right_now | Whether a user can vote right now. Currently equivalent to `vote_epoch < 0`.

Differences from API v2:

- Unlike API v2, this does not require authorization to use. This is to speed up responses and 
because the last thing people want to scrape are Fates List user votes anyways. **You should not rely on
this however, it is prone to change *anytime* in the future**.
- ``vts`` has been renamed to ``timestamps``
"#,
        request_body: &models::Empty {},
        response_body: &models::UserVoted {
            votes: 10,
            voted: true,
            vote_epoch: 101,
            timestamps: vec![chrono::DateTime::<chrono::Utc>::from_utc(chrono::NaiveDateTime::from_timestamp(0, 0), chrono::Utc)],
            time_to_vote: 0,
            vote_right_now: false,
        },
        equiv_v2_route: "(no longer working) [Get User Votes](https://legacy.fateslist.xyz/api/docs/redoc#operation/get_user_votes)",
    });

    docs += &doc(
        models::Route {
            title: "Post Stats",
            method: "GET",
            path: "/bots/{bot_id}/stats",
            path_params: &models::Empty {},
            query_params: &models::Empty {},
            request_body: &models::BotStats {
                guild_count: 3939,
                shard_count: Some(48484),
                shards: Some(vec![149, 22020]),
                user_count: Some(39393),
            },
            response_body: &models::IndexBot::default(),
description: r#"
Post stats to the list

Example:
```py
import requests

# On dpy, guild_count is usually the below
guild_count = len(client.guilds)

# If you are using sharding
shard_count = len(client.shards)
shards = client.shards.keys()

# Optional: User count (this is not accurate for larger bots)
user_count = len(client.users) 

def post_stats(bot_id: int, guild_count: int):
    res = requests.post(f"{api_url}/bots/{bot_id}/stats", json={"guild_count": guild_count})
    json = res.json()
    if res.status != 200:
        # Handle an error in the api
        ...
    return json
```
"#,
            equiv_v2_route: "(no longer working) [Post Stats](https://legacy.fateslist.xyz/api/docs/redoc#operation/set_stats)",
    });

    docs += &doc( models::Route {
        title: "Mini Index",
        method: "GET",
        path: "/mini-index",
        path_params: &models::Empty {},
        query_params: &models::Empty {},
        request_body: &models::Empty {},
        response_body: &models::Index {
            new: Vec::new(),
            top_voted: Vec::new(),
            certified: Vec::new(),
            tags: tags.clone(),
            features: features.clone(),
        },
description: r#"
Returns a mini-index which is basically a Index but with only ``tags``
and ``features`` having any data. Other fields are empty arrays/vectors.

This is used internally by sunbeam for the add bot system where a full bot
index is too costly and making a new struct is unnecessary.
"#,
        equiv_v2_route: "None",
    });

    docs += &doc( models::Route {
        title: "Gets Bot Settings",
        method: "GET",
        path: "/users/{user_id}/bots/{bot_id}/settings",
        path_params: &models::GetUserBotPath {
            user_id: 0,
            bot_id: 0,
        },
        query_params: &models::Empty {},
        request_body: &models::Empty {},
        response_body: &models::Index {
            new: Vec::new(),
            top_voted: Vec::new(),
            certified: Vec::new(),
            tags,
            features,
        },
description: r#"
Returns the bot settings.

The ``bot`` key here is equivalent to a Get Bot response with the following
differences:

- Sensitive fields (see examples) like ``webhook``, ``api_token``, 
``webhook_secret`` and others are filled out here
- This API only allows bot owners to use it, otherwise it will 400!

Staff members *should* instead use Lynx.

Due to massive changes, this API cannot be mapped onto any v2 API
"#,
        equiv_v2_route: "None",
    });

    docs += &doc_category("Auth");

    // Oauth Link API
    docs += &doc( models::Route {
        title: "Get OAuth2 Link",
        method: "GET",
        path: "/oauth2",
        description: "Returns the oauth2 link used to login with",
        path_params: &models::Empty {},
        query_params: &models::Empty {},
        request_body: &models::Empty {},
        response_body: &models::APIResponse {
            done: true,
            reason: None,
            context: Some("https://discord.com/.........".to_string()),
        },
        equiv_v2_route: "(no longer working) [Get OAuth2 Link](https://legacy.fateslist.xyz/docs/redoc#operation/get_oauth2_link)",
    });

    docs += &doc( models::Route {
        title: "Create OAuth2 Login",
        method: "POST",
        path: "/oauth2",
        description: "Creates a oauth2 login given a code",
        path_params: &models::Empty {},
        query_params: &models::Empty {},
        request_body: &models::OauthDoQuery {
            code: "code from discord oauth".to_string(),
            state: Some("Random UUID right now".to_string())
        },
        response_body: &models::OauthUserLogin::default(),
        equiv_v2_route: "(no longer working) [Login User](https://legacy.fateslist.xyz/api/docs/redoc#operation/login_user)",
    });

    docs += &doc( models::Route {
        title: "Delete OAuth2 Login",
        method: "DELETE",
        path: "/oauth2",
description: r#"
'Deletes' (logs out) a oauth2 login. Always call this when logging out 
even if you do not use cookies as it may perform other logout tasks in future

This API is essentially a logout
"#,
        path_params: &models::Empty {},
        query_params: &models::Empty {},
        request_body: &models::Empty {},
        response_body: &models::APIResponse {
            done: true,
            reason: None,
            context: None,
        },
        equiv_v2_route: "(no longer working) [Logout Sunbeam](https://legacy.fateslist.xyz/docs/redoc#operation/logout_sunbeam)",
    });

    // Return docs
    docs
}
