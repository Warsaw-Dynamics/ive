use crate::{
    cli::{Capture, Input},
    examples,
    service::{Ready, Request, Response, Service},
    viewport::{Generation, Zoom},
};
use futures::StreamExt;
use gpui::{
    App, Bounds, Context, FocusHandle, KeyBinding, Render, Role, SharedString, Window,
    WindowBounds, WindowOptions, actions, div, img, prelude::*, px, rgb, size,
};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering},
    },
};

actions!(
    diagram_viewer,
    [
        Tab,
        PreviousTab,
        Fit,
        ZoomIn,
        ZoomOut,
        ActualSize,
        ToggleSource
    ]
);
pub struct Settings {
    pub language: String,
    pub background: bool,
    pub capture: Capture,
}
enum State {
    Rendering,
    Ready(Ready),
    Failed(String),
}
enum CaptureState {
    Disabled,
    Armed(PathBuf),
    Pending,
}
struct Viewer {
    input: Input,
    language: String,
    source: SharedString,
    state: State,
    zoom: Zoom,
    show_source: bool,
    generation: Generation,
    service: Service,
    focus: FocusHandle,
    capture: CaptureState,
    exit: Arc<AtomicI32>,
}

pub fn run(input: Input, settings: Settings) -> Result<(), String> {
    let exit = Arc::new(AtomicI32::new(0));
    let result = exit.clone();
    let startup_result = exit.clone();
    gpui_platform::application().run(move |cx: &mut App| {
        bind_keys(cx);
        let bounds = Bounds::centered(None, size(px(1360.), px(880.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            focus: !settings.background,
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("Native Diagrams".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let background = settings.background;
        match cx.open_window(options, |window, cx| {
            cx.new(|cx| Viewer::new(input, settings, exit, window, cx))
        }) {
            Ok(_) => {
                if !background {
                    cx.activate(true);
                }
            }
            Err(error) => {
                eprintln!("diagram-viewer: cannot open native window: {error}");
                startup_result.store(1, Ordering::Release);
                cx.quit();
            }
        }
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
    });
    if result.load(Ordering::Acquire) == 0 {
        return Ok(());
    }
    Err("native diagram render or capture failed; see the diagnostic above".into())
}

fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", Tab, None),
        KeyBinding::new("shift-tab", PreviousTab, None),
        KeyBinding::new("ctrl-0", Fit, None),
        KeyBinding::new("ctrl-1", ActualSize, None),
        KeyBinding::new("ctrl-=", ZoomIn, None),
        KeyBinding::new("ctrl--", ZoomOut, None),
        KeyBinding::new("ctrl-b", ToggleSource, None),
    ]);
}

impl Viewer {
    fn new(
        input: Input,
        settings: Settings,
        exit: Arc<AtomicI32>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (service, mut responses) = Service::start(cx.svg_renderer(), cx.text_system().clone());
        let mut viewer = Self {
            input,
            language: settings.language,
            source: "Loading source…".into(),
            state: State::Rendering,
            zoom: Zoom::Fit,
            show_source: true,
            generation: Generation::default(),
            service,
            focus: cx.focus_handle(),
            capture: match settings.capture {
                Capture::Disabled => CaptureState::Disabled,
                Capture::Png(path) => CaptureState::Armed(path),
            },
            exit,
        };
        window.focus(&viewer.focus, cx);
        cx.spawn_in(window, async move |this, cx| {
            while let Some(response) = responses.next().await {
                if this
                    .update_in(cx, |this, window, cx| this.accept(response, window, cx))
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        cx.observe_window_bounds(window, |this, window, cx| this.request(window, cx))
            .detach();
        cx.on_app_quit(|this, cx| {
            let stop = this.service.stop();
            cx.background_executor().spawn(async move {
                stop();
            })
        })
        .detach();
        viewer.request(window, cx);
        viewer
    }

    fn request(&mut self, window: &Window, cx: &mut Context<Self>) {
        let viewport = window.viewport_size();
        let scale = window.scale_factor();
        let source_width = if self.show_source { 340. } else { 0. };
        let viewport = (
            (f32::from(viewport.width) - 204. - source_width).max(120.) * scale,
            (f32::from(viewport.height) - 186.).max(120.) * scale,
        );
        let generation = self.generation.advance();
        self.state = State::Rendering;
        self.service.submit(Request {
            generation,
            input: self.input.clone(),
            language: self.language.clone(),
            zoom: self.zoom,
            viewport,
            scale,
        });
        cx.notify();
    }

    fn accept(&mut self, response: Response, window: &mut Window, cx: &mut Context<Self>) {
        if self.generation.accepts(response.generation) {
            self.source = response.source.into();
            self.state = match response.result {
                Ok(ready) => {
                    eprintln!(
                        "diagram-viewer: {} · {} · native measurements {} · deterministic fallbacks {}",
                        ready.metadata.family,
                        ready.metadata.backend,
                        ready.metadata.host_calls,
                        ready.metadata.fallback_calls
                    );
                    State::Ready(ready)
                }
                Err(error) => {
                    eprintln!("diagram-viewer: {error}");
                    State::Failed(error)
                }
            };
            cx.notify();
            self.capture(window, cx);
        }
    }

    fn capture(&mut self, window: &Window, _cx: &mut Context<Self>) {
        let CaptureState::Armed(_) = &self.capture else {
            return;
        };
        let CaptureState::Armed(path) = std::mem::replace(&mut self.capture, CaptureState::Pending)
        else {
            return;
        };
        let exit = self.exit.clone();
        if matches!(self.state, State::Failed(_)) {
            exit.store(1, Ordering::Release);
        }
        window.on_next_frame(move |window, _| {
            window.on_next_frame(move |window, cx| save_frame(window, cx, path, exit))
        });
    }

    fn set_zoom(&mut self, zoom: Zoom, window: &Window, cx: &mut Context<Self>) {
        self.zoom = zoom;
        self.request(window, cx);
    }
    fn zoom_by(&mut self, delta: i16, window: &Window, cx: &mut Context<Self>) {
        let current = match self.zoom {
            Zoom::Fit => 100,
            Zoom::Percent(percent) => percent,
        };
        self.set_zoom(
            Zoom::Percent((current as i16 + delta).clamp(25, 400) as u16),
            window,
            cx,
        );
    }
    fn toggle_source(&mut self, window: &Window, cx: &mut Context<Self>) {
        self.show_source = !self.show_source;
        self.request(window, cx);
    }

    fn header(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .h(px(64.))
            .flex_none()
            .flex()
            .items_center()
            .gap_3()
            .px_5()
            .border_b_1()
            .border_color(rgb(0x303747))
            .child(
                div()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_lg()
                    .child("Native diagrams"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x94a3b8))
                    .child("COMPATIBILITY VIEWER"),
            )
            .child(div().flex_1())
            .child(
                button("language-mermaid", "Mermaid", self.language == "mermaid").on_click(
                    cx.listener(|this, _, window, cx| {
                        this.language = "mermaid".into();
                        if matches!(this.input, Input::Example(_)) {
                            this.input = Input::Example("flowchart".into());
                        }
                        this.request(window, cx);
                    }),
                ),
            )
            .child(
                button("language-plantuml", "PlantUML", self.language == "plantuml").on_click(
                    cx.listener(|this, _, window, cx| {
                        this.language = "plantuml".into();
                        if matches!(this.input, Input::Example(_)) {
                            this.input = Input::Example("plantuml-sequence".into());
                        }
                        this.request(window, cx);
                    }),
                ),
            )
    }

    fn toolbar(&self, cx: &Context<Self>) -> impl IntoElement {
        let title = match &self.input {
            Input::File(path) => path.display().to_string(),
            Input::Example(name) => format!("Example / {name}"),
        };
        let zoom = match self.zoom {
            Zoom::Fit => "Fit".into(),
            Zoom::Percent(percent) => format!("{percent}%"),
        };
        div()
            .h(px(52.))
            .flex_none()
            .flex()
            .items_center()
            .gap_2()
            .px_4()
            .bg(rgb(0x1c2330))
            .border_b_1()
            .border_color(rgb(0x303747))
            .child(div().flex_1().text_sm().overflow_hidden().child(title))
            .child(
                button("fit", "Fit", matches!(self.zoom, Zoom::Fit)).on_click(
                    cx.listener(|this, _, window, cx| this.set_zoom(Zoom::Fit, window, cx)),
                ),
            )
            .child(button("actual-size", "100%", false).on_click(
                cx.listener(|this, _, window, cx| this.set_zoom(Zoom::Percent(100), window, cx)),
            ))
            .child(
                button("zoom-out", "−", false)
                    .aria_label("Zoom out")
                    .on_click(cx.listener(|this, _, window, cx| this.zoom_by(-25, window, cx))),
            )
            .child(div().w(px(48.)).text_sm().text_center().child(zoom))
            .child(
                button("zoom-in", "+", false)
                    .aria_label("Zoom in")
                    .on_click(cx.listener(|this, _, window, cx| this.zoom_by(25, window, cx))),
            )
            .child(
                button("toggle-source", "Source", self.show_source)
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_source(window, cx))),
            )
    }

    fn examples(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .w(px(180.))
            .flex_none()
            .flex()
            .flex_col()
            .border_r_1()
            .border_color(rgb(0x303747))
            .bg(rgb(0x141b27))
            .child(
                div()
                    .px_4()
                    .py_3()
                    .text_xs()
                    .text_color(rgb(0x94a3b8))
                    .child(if self.language == "plantuml" {
                        "PLANTUML · 5 EXAMPLES"
                    } else {
                        "MERMAID · 37 VARIANTS"
                    }),
            )
            .child(
                div()
                    .id("example-list")
                    .role(Role::List)
                    .aria_label("Built-in diagram examples")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_2()
                    .children(
                        examples::for_language(&self.language)
                            .iter()
                            .map(|example| {
                                let name = example.name;
                                button(
                            SharedString::from(format!("example-{name}")),
                            name,
                            matches!(&self.input, Input::Example(current) if current == name),
                        )
                        .w_full()
                        .mb_1()
                        .on_click(cx.listener(
                            move |this, _, window, cx| {
                                this.input = Input::Example(name.into());
                                this.zoom = Zoom::Fit;
                                this.request(window, cx);
                            },
                        ))
                            }),
                    ),
            )
    }

    fn source_panel(&self) -> impl IntoElement {
        div()
            .w(px(340.))
            .flex_none()
            .flex()
            .flex_col()
            .bg(rgb(0x19212e))
            .border_r_1()
            .border_color(rgb(0x303747))
            .child(
                div()
                    .px_4()
                    .py_3()
                    .text_xs()
                    .text_color(rgb(0x94a3b8))
                    .child("SOURCE · READ ONLY"),
            )
            .child(
                div()
                    .id("source")
                    .role(Role::Document)
                    .aria_label("Diagram source")
                    .accessibility_id("diagram-source")
                    .flex_1()
                    .overflow_scroll()
                    .px_4()
                    .pb_4()
                    .font_family("monospace")
                    .text_size(px(12.))
                    .children(
                        self.source
                            .lines()
                            .take(4096)
                            .enumerate()
                            .map(|(index, line)| {
                                div()
                                    .flex()
                                    .gap_3()
                                    .h(px(20.))
                                    .child(
                                        div()
                                            .w(px(28.))
                                            .flex_none()
                                            .text_color(rgb(0x64748b))
                                            .child(format!("{}", index + 1)),
                                    )
                                    .child(SharedString::from(line.to_owned()))
                            }),
                    ),
            )
    }

    fn canvas(&self) -> gpui::AnyElement {
        match &self.state {
            State::Ready(ready) => div()
                .id("diagram")
                .role(Role::Image)
                .aria_label(format!("{} diagram", ready.metadata.family))
                .flex_1()
                .overflow_scroll()
                .bg(rgb(0xf8fafc))
                .p_3()
                .child(
                    div()
                        .w(px(ready.logical_size.0))
                        .h(px(ready.logical_size.1))
                        .flex_none()
                        .min_w_full()
                        .min_h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            img(ready.image.clone())
                                .w(px(ready.logical_size.0))
                                .h(px(ready.logical_size.1))
                                .flex_none(),
                        ),
                )
                .into_any_element(),
            State::Rendering => div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(0xf8fafc))
                .text_color(rgb(0x475569))
                .child("Rendering native diagram…")
                .into_any_element(),
            State::Failed(error) => div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(0xf8fafc))
                .p_8()
                .child(
                    div()
                        .max_w(px(620.))
                        .p_6()
                        .rounded_lg()
                        .border_1()
                        .border_color(rgb(0xfca5a5))
                        .bg(rgb(0xfff1f2))
                        .text_color(rgb(0x9f1239))
                        .child(
                            div()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_lg()
                                .mb_3()
                                .child("Diagram could not be rendered"),
                        )
                        .child(error.clone()),
                )
                .into_any_element(),
        }
    }

    fn status(&self) -> impl IntoElement {
        let status = match &self.state {
            State::Rendering => "Rendering in isolated worker…".into(),
            State::Failed(error) => format!("Error · {error}"),
            State::Ready(ready) => format!(
                "{} · {} / {} · native metrics {} · fallback {}",
                ready.metadata.family,
                ready.metadata.backend,
                ready.metadata.upstream,
                ready.metadata.host_calls,
                ready.metadata.fallback_calls
            ),
        };
        div()
            .id("status")
            .role(Role::Status)
            .aria_label(status.clone())
            .h(px(36.))
            .flex_none()
            .flex()
            .items_center()
            .px_4()
            .text_xs()
            .text_color(rgb(0xa8b7ca))
            .border_t_1()
            .border_color(rgb(0x303747))
            .child(status)
    }
}

impl Render for Viewer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("viewer")
            .role(Role::Application)
            .aria_label("Native diagram viewer")
            .track_focus(&self.focus)
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x111827))
            .text_color(rgb(0xe2e8f0))
            .font_family("sans-serif")
            .on_action(cx.listener(|_, _: &Tab, window, cx| window.focus_next(cx)))
            .on_action(cx.listener(|_, _: &PreviousTab, window, cx| window.focus_prev(cx)))
            .on_action(
                cx.listener(|this, _: &Fit, window, cx| this.set_zoom(Zoom::Fit, window, cx)),
            )
            .on_action(cx.listener(|this, _: &ActualSize, window, cx| {
                this.set_zoom(Zoom::Percent(100), window, cx)
            }))
            .on_action(cx.listener(|this, _: &ZoomIn, window, cx| this.zoom_by(25, window, cx)))
            .on_action(cx.listener(|this, _: &ZoomOut, window, cx| this.zoom_by(-25, window, cx)))
            .on_action(
                cx.listener(|this, _: &ToggleSource, window, cx| this.toggle_source(window, cx)),
            )
            .child(self.header(cx))
            .child(self.toolbar(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(self.examples(cx))
                    .when(self.show_source, |element| {
                        element.child(self.source_panel())
                    })
                    .child(self.canvas()),
            )
            .child(self.status())
    }
}

fn button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    selected: bool,
) -> gpui::Stateful<gpui::Div> {
    let id = id.into();
    let label = label.into();
    div()
        .id(id.clone())
        .accessibility_id(id)
        .role(Role::Button)
        .aria_label(label.clone())
        .focusable()
        .tab_stop(true)
        .cursor_pointer()
        .px_3()
        .py_2()
        .rounded_md()
        .text_sm()
        .bg(rgb(if selected { 0x24486a } else { 0x253044 }))
        .hover(|style| style.bg(rgb(0x33455e)))
        .child(label)
}

fn save_frame(window: &Window, cx: &mut App, path: PathBuf, exit: Arc<AtomicI32>) {
    let frame = window
        .render_to_image()
        .map_err(|error| format!("cannot capture native viewer frame: {error}"));
    let save = cx.background_executor().spawn(async move {
        frame.and_then(|image| {
            image
                .save_with_format(&path, image::ImageFormat::Png)
                .map_err(|error| format!("cannot save viewer capture {}: {error}", path.display()))
        })
    });
    cx.spawn(async move |cx| {
        if let Err(error) = save.await {
            eprintln!("diagram-viewer: {error}");
            exit.store(1, Ordering::Release);
        }
        cx.update(|cx| cx.quit());
    })
    .detach();
}
