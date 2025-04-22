const LIMIT_PIN: u8 = 24;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use chicken_door::app::*;
    use chicken_door::settings::Settings;
    use std::time::Duration;
    use std::ops::Deref;
    use tokio::sync::mpsc;
    use toml;
    use notify::{Config, Event,EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::path::Path;
    use std::fs::read_to_string;

    tokio::spawn(async move {
        let settings_file = Path::new("./settings.toml");
        let mut settings: Settings;
        if settings_file.exists() {
            let settings_str = read_to_string("settings.toml").unwrap();
            settings = toml::from_str(settings_str.as_str()).unwrap();
        } else {
            settings = Settings::default();
        }
        
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                match res {
                    Ok(Event { kind: EventKind::Modify(_), ..}) => {
                        println!("Reloading settings");
                        let settings_str = read_to_string("settings.toml").unwrap();
                        settings = toml::from_str(settings_str.as_str()).unwrap();
                    },
                    Err(e) => println!("watch error: {:?}", e),
                    _ => println!("Unhandled watcher event"),
                }
            },
            Config::default(),
        ).unwrap();
        watcher.watch(settings_file, RecursiveMode::NonRecursive);
        
        loop {
            println!("Sleeping 2 seconds");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });

    let conf = get_configuration(Some("Cargo.toml")).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}

