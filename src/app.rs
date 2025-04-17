use icondata;
use leptos::{prelude::*, task::spawn_local};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_navigate,
    StaticSegment,
};
use serde::{Deserialize, Serialize};
use thaw::ssr::SSRMountStyleProvider;
use thaw::*;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <SSRMountStyleProvider>
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />
                    <AutoReload options=options.clone() />
                    <HydrationScripts options />
                    <MetaTags />
                </head>
                <body>
                    <App />
                </body>
            </html>
        </SSRMountStyleProvider>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let theme = RwSignal::new(Theme::light());

    view! {
        <ConfigProvider theme>
            // injects a stylesheet into the document <head>
            // id=leptos means cargo-leptos will hot-reload this stylesheet
            <Stylesheet id="leptos" href="/pkg/chicken-door.css" />

            // sets the document title
            <Title text="Welcome to Leptos" />

            // content for this welcome page
            <Router>
                <main>
                    <Routes fallback=PageNotFound>
                        <Route path=StaticSegment("/control") view=ControlPanel />
                        <Route path=StaticSegment("/settings") view=SettingsPanel />
                    </Routes>
                </main>
            </Router>
        </ConfigProvider>
    }
}

#[component]
fn PageNotFound() -> impl IntoView {
    view! {
        <Layout>
            <NavBar />
            "Page not found."
        </Layout>
    }
}

#[component]
fn ControlPanel() -> impl IntoView {
    let open_clicked = move |_| spawn_local(async { open().await.unwrap() });
    let close_clicked = move |_| spawn_local(async { close().await.unwrap() });

    view! {
        <Layout>
            <NavBar />
            <Flex class="container">
                <Card>
                    <CardHeader>
                        <b>"Control Panel"</b>
                    </CardHeader>
                    <Button on_click=open_clicked>"Open Door"</Button>
                    <Button on_click=close_clicked>"Close Door"</Button>
                </Card>
            </Flex>
        </Layout>
    }
}

#[component]
fn SettingsPanel() -> impl IntoView {
    let (pending, set_pending) = signal(false);
    // let apply = move |_| {
    //     if !pending.get() {
    //         spawn_local(async { apply_settings().await.unwrap() })
    //     }
    // };
    let write_settings = ServerAction::<WriteSettings>::new();
    let settings = Resource::new(
        move || {
            (
                write_settings.version().get(),
                write_settings.version().get(),
            )
        },
        move |_| get_settings(),
    );

    view! {
        <Layout>
            <NavBar />
            <Flex class="container">
                <Card>
                    <CardHeader>
                        <b>"Settings"</b>
                    </CardHeader>
                    <Transition
                        fallback=move || {
                            view! { <p>"Loading initial data..."</p> }
                        }
                        set_pending
                    >
                        {move || Suspend::new(async move {
                            let settings = settings.await.unwrap();
                            let open_time = RwSignal::new(settings.times.open);
                            let close_time = RwSignal::new(settings.times.close);
                            let close_light_level = RwSignal::new(settings.light_levels.close);
                            let open_light_level = RwSignal::new(settings.light_levels.open);
                            {
                                view! {
                                    <Flex class="row">
                                        <div class="label">"Open time"</div>
                                        <TimePicker value=open_time />
                                    </Flex>
                                    <Flex class="row">
                                        "Close time" <TimePicker value=close_time />
                                    </Flex>
                                    <Flex class="row">
                                        "Open light level" <Flex>
                                            <Slider step=5.0 show_stops=false value=open_light_level>
                                                <SliderLabel value=open_light_level>
                                                    {open_light_level}
                                                </SliderLabel>
                                            </Slider>
                                            <Button>
                                                // on_click=set_light_open
                                                "Use Current Reading"
                                            </Button>
                                        </Flex>
                                    </Flex>
                                    <Flex class="row">
                                        "Close light level" <Flex>
                                            <Slider step=5.0 show_stops=false value=close_light_level>
                                                <SliderLabel value=close_light_level>
                                                    {close_light_level}
                                                </SliderLabel>
                                            </Slider>
                                            // on_click=set_light_close
                                            <Button>"Use Current Reading"</Button>
                                        </Flex>
                                    </Flex>
                                    <CardFooter>
                                        <Button
                                            icon=icondata::BsCheckLg
                                            on_click=move |_| {
                                                write_settings
                                                    .dispatch(
                                                        Settings {
                                                            light_levels: LightLevels {
                                                                close: close_light_level.get(),
                                                                open: open_light_level.get(),
                                                            },
                                                            times: Times {
                                                                open: open_time.get(),
                                                                close: close_time.get(),
                                                            },
                                                        }
                                                            .into(),
                                                    );
                                            }
                                        >
                                            "Apply"
                                        </Button>
                                    </CardFooter>
                                }
                            }
                        })}
                    </Transition>
                </Card>
            </Flex>
        </Layout>
    }
}

#[component]
fn NavBar() -> impl IntoView {
    let navigate = RwSignal::new(use_navigate());
    let theme = Theme::use_rw_theme();
    let theme_name = Memo::new(move |_| {
        theme.with(|theme| {
            if theme.name == *"light" {
                "Dark".to_string()
            } else {
                "Light".to_string()
            }
        })
    });
    let change_theme = move |_| {
        if theme_name.get_untracked() == "Light" {
            theme.set(Theme::light());
        } else {
            theme.set(Theme::dark());
        }
    };

    view! {
        <LayoutHeader class="header">
            <Flex>
                <Button
                    icon=icondata::AiHomeTwotone
                    on_click=move |_| {
                        navigate.get()("/control", Default::default());
                    }
                >
                    <b>"Home"</b>
                </Button>
                <Button
                    icon=icondata::AiSettingTwotone
                    on_click=move |_| {
                        navigate.get()("/settings", Default::default());
                    }
                >
                    <b>"Settings"</b>
                </Button>
            </Flex>
            <Button
                icon=Memo::new(move |_| {
                    theme
                        .with(|theme| {
                            if theme.name == "light" {
                                icondata::BiMoonRegular
                            } else {
                                icondata::BiSunRegular
                            }
                        })
                })
                on_click=change_theme
            />

        </LayoutHeader>
    }
}

#[server]
async fn close() -> Result<(), ServerFnError> {
    println!("closing door");
    Ok(())
}

#[server]
async fn open() -> Result<(), ServerFnError> {
    println!("opening door");
    Ok(())
}

#[server]
async fn apply_settings() -> Result<(), ServerFnError> {
    println!("applying settings");
    Ok(())
}

#[server]
async fn set_light_open() -> Result<(), ServerFnError> {
    println!("using current light sensor value for open threshold");
    Ok(())
}

#[server]
async fn set_light_close() -> Result<(), ServerFnError> {
    println!("using current light sensor value for close threshold");
    Ok(())
}

#[server]
async fn get_settings() -> Result<Settings, ServerFnError> {
    use std::fs::read_to_string;
    use std::path::Path;
    use toml;
    let settings_file = Path::new("./settings.toml");
    let settings: Settings;
    if settings_file.exists() {
        let settings_str = read_to_string("settings.toml")?;
        settings = toml::from_str(settings_str.as_str())?;
    } else {
        settings = Settings::default();
    }
    return Ok(settings);
}

#[server]
async fn write_settings(settings: Settings) -> Result<(), ServerFnError> {
    use std::fs::write;
    use toml;
    let settings_str = toml::to_string_pretty(&settings)?;
    Ok(write("./settings.toml", settings_str)?)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    light_levels: LightLevels,
    times: Times,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            light_levels: LightLevels::default(),
            times: Times::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LightLevels {
    close: f64,
    open: f64,
}

impl Default for LightLevels {
    fn default() -> Self {
        Self {
            open: 100.0,
            close: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Times {
    open: chrono::NaiveTime,
    close: chrono::NaiveTime,
}

impl Default for Times {
    fn default() -> Self {
        Self {
            open: chrono::NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            close: chrono::NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        }
    }
}
