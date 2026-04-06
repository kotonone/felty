use std::{path::PathBuf, process, env, sync::Arc};

use crate::util::open;
use super::protocol::{respond, to_custom_protocol_path, to_package_and_path};

use tao::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoopBuilder}, window::WindowBuilder};
use wry::{WebContext, WebViewAttributes, WebViewBuilder};

#[derive(Debug)]
pub enum FeltyEvents {
    MenuEvent(String),
}

pub type ProtocolHookFn = Arc<dyn Fn(http::Request<Vec<u8>>, wry::RequestAsyncResponder) -> Result<(), (http::Request<Vec<u8>>, wry::RequestAsyncResponder)> + Send + Sync>;

pub struct FeltyApp {
    config: crate::config::AppConfig,
    custom_protocol_hook: Option<ProtocolHookFn>,
    menu_event_hook: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
    before_run_hook: Option<Box<dyn FnOnce(&crate::config::AppConfig)>>,
    /// アプリケーションの起動時に最初に読み込む URL
    start_url: Option<String>,
    /// 内部プロトコルのみにナビゲーションを制限するかどうか
    is_internal_navigation_only: bool,
}

impl FeltyApp {
    pub fn new(config: crate::config::AppConfig) -> Self {
        Self {
            config,
            custom_protocol_hook: None,
            menu_event_hook: None,
            before_run_hook: None,
            start_url: None,
            is_internal_navigation_only: true,
        }
    }

    pub fn on_custom_protocol_request<F>(mut self, f: F) -> Self
    where
        F: Fn(http::Request<Vec<u8>>, wry::RequestAsyncResponder) -> Result<(), (http::Request<Vec<u8>>, wry::RequestAsyncResponder)> + Send + Sync + 'static,
    {
        self.custom_protocol_hook = Some(Arc::new(f));
        self
    }

    pub fn on_menu_event<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.menu_event_hook = Some(Arc::new(f));
        self
    }

    pub fn on_before_run<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&crate::config::AppConfig) + 'static,
    {
        self.before_run_hook = Some(Box::new(f));
        self
    }

    /// アプリケーションの起動時に最初に読み込む URL を指定します。
    /// None に設定した場合、ランタイムパッケージの index.html が読み込まれます。
    pub fn with_start_url<S: Into<String>>(mut self, url: Option<S>) -> Self {
        self.start_url = url.map(Into::into);
        self
    }

    /// 内部プロトコルのみにナビゲーションを制限するかどうかを指定します。
    pub fn with_internal_navigation_only(mut self, enabled: bool) -> Self {
        self.is_internal_navigation_only = enabled;
        self
    }

    pub fn run(self) {
        if let Some(hook) = self.before_run_hook {
            hook(&self.config);
        }

        let event_loop = EventLoopBuilder::<FeltyEvents>::with_user_event().build();

        if cfg!(target_os = "macos") {
            use muda::MenuEvent;

            #[cfg(target_os = "macos")]
            if let Some(menu) = &self.config.menu {
                menu.init_for_nsapp();
            }

            let proxy = event_loop.create_proxy();
            MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
                let _ = proxy.send_event(FeltyEvents::MenuEvent(event.id.0));
            }));
        }

        let mut context = WebContext::new(Some(PathBuf::from(&self.config.cache_directory)));
        let mut attributes = WebViewAttributes::default();
        attributes.context = Some(&mut context);

        let window = WindowBuilder::new()
            .with_title(&self.config.name)
            .with_inner_size(self.config.window_size)
            .with_resizable(self.config.resizable)
            .with_maximizable(self.config.maximizable)
            .with_window_icon(self.config.icon.clone())
            .build(&event_loop)
            .unwrap();

        let custom_protocol_hook = self.custom_protocol_hook.clone();

        let start_url = self.start_url.clone().unwrap_or_else(|| {
            to_custom_protocol_path(&self.config.runtime_package, "index.html")
        });
        let is_internal_navigation_only = self.is_internal_navigation_only;

        let webview = WebViewBuilder::with_attributes(attributes)
            .with_url(start_url)
            .with_autoplay(true)
            .with_accept_first_mouse(true)
            .with_incognito(true)
            .with_user_agent(&format!("{}/{}", self.config.name, self.config.version))
            .with_back_forward_navigation_gestures(false)
            .with_clipboard(true)
            .with_hotkeys_zoom(false)
            .with_new_window_req_handler(|_| false)
            .with_download_started_handler(|_, _| false)
            .with_navigation_handler(move |url| {
                if is_internal_navigation_only {
                    let global_config = crate::config::get_global();
                    to_package_and_path(&url).is_some_and(|(package, path)| {
                        log::debug!("Navigation requested: {}/{}", package, path);
                        package == global_config.runtime_package
                    })
                } else {
                    true
                }
            })
            .with_asynchronous_custom_protocol(self.config.internal_protocol.clone(), move |_ctx, request, responder| {
                if let Some(hook) = &custom_protocol_hook {
                    match hook(request, responder) {
                        Ok(()) => return,
                        Err((req, res)) => {
                            tokio::spawn(respond(req, res));
                        }
                    }
                } else {
                    tokio::spawn(respond(request, responder));
                }
            })
            .build(&window)
            .unwrap();

        #[cfg(debug_assertions)]
        webview.open_devtools();

        let menu_hook = self.menu_event_hook.clone();
        let website_url = self.config.website_url.clone();
        let release_note_url = self.config.release_note_url.clone();
        let report_url = self.config.report_url.clone();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    let pid = {
                        #[cfg(target_os = "windows")] {
                            use wry::WebViewExtWindows;

                            let mut pid = 0u32;
                            unsafe { let _ = webview.controller().CoreWebView2().map(|w| w.BrowserProcessId(&mut pid)); }

                            if pid > 0 {
                                Some(pid)
                            } else {
                                None
                            }
                        }
                        #[cfg(not(target_os = "windows"))] {
                            None::<u32>
                        }
                    };

                    if let Some(pid) = pid {
                        // NOTE: プロセスはデフォルトでデタッチ状態になる: https://github.com/rust-lang/rust/issues/31289
                        let _ = process::Command::new(env::current_exe().unwrap())
                            .args(["--wait-pid", &pid.to_string(), "--delete-caches-only"])
                            .spawn();
                    }

                    *control_flow = ControlFlow::Exit;
                },
                Event::UserEvent(user_event) => match user_event {
                    FeltyEvents::MenuEvent(id) => {
                        let mut continue_default = true;
                        if let Some(hook) = &menu_hook {
                            continue_default = hook(&id);
                        }

                        if continue_default {
                            match &*id {
                                "help.website" => { let _ = open(&website_url); }
                                "help.update-log" => { let _ = open(&release_note_url); }
                                "help.bug-report" => { let _ = open(&report_url); }
                                #[cfg(debug_assertions)]
                                "debug.devtools" => { webview.open_devtools(); }
                                #[cfg(debug_assertions)]
                                "debug.reload-webview" => { let _ = webview.evaluate_script("location.reload()"); }
                                &_ => ()
                            }
                        }
                    }
                },
                _ => ()
            }
        });
    }
}

pub fn run(config: crate::config::AppConfig) {
    FeltyApp::new(config).run();
}
