use icondata;
use leptos::{prelude::*, task::spawn_local};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes, Redirect},
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
                        <Route path=StaticSegment("/") view=|| view! {<Redirect path="/control" /> }/>
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
    // let open_clicked = move |_| spawn_local(async { open().await.unwrap() });
    // let close_clicked = move |_| spawn_local(async { close().await.unwrap() });
    let close_clicked = ServerAction::<Close>::new();
    let open_clicked = ServerAction::<Open>::new();

    view! {
        <Layout>
            <NavBar />
            <Flex class="container">
                <Card>
                    <CardHeader>
                        <b>"Control Panel"</b>
                    </CardHeader>
                    <Button on_click=move |_| {
                        open_clicked.dispatch(Open {});
                    }>"Open Door"</Button>
                    <Button on_click=move |_| {
                        close_clicked.dispatch(Close {});
                    }>"Close Door"</Button>
                </Card>
            </Flex>
        </Layout>
    }
}

#[component]
fn SettingsPanel() -> impl IntoView {
    let (pending, set_pending) = signal(false);
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
                                                on_click=move |_| {
                                                    spawn_local(async move {
                                                        if let Ok(level) = light_level().await {
                                                            open_light_level.set(level);
                                                        }
                                                    });
                                                    // if let ok(level) = light_level().await {
                                                    //     open_light_level.set(level);
                                                    // }
                                                }
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
                                                if !pending.get() {
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

const LIMIT_PIN: u8 = 24;
const MOTOR_FLIP_FLOP_PIN: u8 = 5;
const MOTOR_ENABLE_PIN: u8 = 6;
const DOOR_CLOSE_SECS: u64 = 5;
const MFF_SAFETY_MSECS: u64 = 250;
const OPEN_TIMEOUT_SECS: u64 = 6;

#[server(
    name = Close,
    endpoint = "close_door",
)]
async fn close() -> Result<(), ServerFnError> {
    use rppal::gpio::{Gpio, Trigger};
    use std::time::Duration;
    use std::thread;

    let gpio = Gpio::new().expect("failed to open gpio interface");
    let mut mff_pin = gpio.get(MOTOR_FLIP_FLOP_PIN).expect("failed to get motor flip flop pin").into_output();
    let mut me_pin = gpio.get(MOTOR_ENABLE_PIN).expect("failed to get motor enable pin").into_output();
    mff_pin.set_reset_on_drop(false);
    me_pin.set_reset_on_drop(false);
    
    me_pin.set_low();
    println!("Sleeping for {MFF_SAFETY_MSECS} milliseconds");
    thread::sleep(Duration::from_millis(MFF_SAFETY_MSECS));
    mff_pin.set_high();
    me_pin.set_high();
    println!("Sleeping for {DOOR_CLOSE_SECS} seconds");
    thread::sleep(Duration::from_secs(DOOR_CLOSE_SECS));
    mff_pin.set_low();
    me_pin.set_low();
    println!("Finished close routine");
    Ok(())
}

#[server(
    name = Open,
    endpoint = "open_door",
)]
async fn open() -> Result<(), ServerFnError> {
    use rppal::gpio::{Gpio, Trigger};
    use std::time::Duration;
    use std::thread;

    let gpio = Gpio::new().expect("failed to open gpio interface");
    let mut limit_pin = gpio.get(LIMIT_PIN).expect("failed to get limit switch pin").into_input_pullup();
    let mut mff_pin = gpio.get(MOTOR_FLIP_FLOP_PIN).expect("failed to get motor flip flop pin").into_output();
    let mut me_pin = gpio.get(MOTOR_ENABLE_PIN).expect("failed to get motor enable pin").into_output();
    mff_pin.set_reset_on_drop(false);
    me_pin.set_reset_on_drop(false);
    limit_pin.set_interrupt(Trigger::Both, Some(Duration::from_millis(10)));
    
    me_pin.set_low();
    println!("Sleeping for {MFF_SAFETY_MSECS} milliseconds");
    thread::sleep(Duration::from_millis(MFF_SAFETY_MSECS));
    mff_pin.set_low();
    me_pin.set_high();
    println!("Waiting for switch interrupt (timout {OPEN_TIMEOUT_SECS} seconds)");
    match limit_pin.poll_interrupt(true, Some(Duration::from_secs(OPEN_TIMEOUT_SECS))) {
        Ok(None) => println!("Timeout reached, switch was not hit"),
        Ok(Some(_)) => println!("Limit switch hit, door opened"),
        _ => println!("Error waiting for interrupt"),
    }
    mff_pin.set_low();
    me_pin.set_low();
    println!("Finished open routine");
    Ok(())
}

#[server(
    name = ApplySettings,
    endpoint = "apply_settings",
)]
async fn apply_settings() -> Result<(), ServerFnError> {
    println!("applying settings");
    Ok(())
}

#[server(
    name = SetLightOpen,
    endpoint = "set_light_open",
)]
async fn set_light_open() -> Result<(), ServerFnError> {
    println!("using current light sensor value for open threshold");
    Ok(())
}

#[server(
    name = SetLightClose,
    endpoint = "set_light_close",
)]
async fn set_light_close() -> Result<(), ServerFnError> {
    println!("using current light sensor value for close threshold");
    Ok(())
}

#[server(
    name = GetSettings,
    endpoint = "get_settings",
)]
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

#[server(
    name = WriteSettings,
    endpoint = "write_settings",
)]
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

#[server(
    name = LightLevel,
    endpoint = "light_level",
)]
async fn light_level() -> Result<f64, ServerFnError> {
    use rppal::spi::{Bus, Mode, Segment, SlaveSelect, Spi};

    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0)?;
    let write_buffer: [u8; 3] = [6, 0, 0];
    let mut read_buffer = [0u8; 5];
    
    spi.transfer(&mut read_buffer, &write_buffer);
    
    let mut result: u32 = 0;
    for (i, byte) in read_buffer.iter().enumerate() {
        result |= (u32::from(*byte) << (u32::from(2 - i as u8) * 8)) as u32;
    }
    let result = (1.0 - ((result as f64)/4096.0)) * 100.0;
    Ok(result)
}
