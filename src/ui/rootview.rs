use crate::device::io;
use crate::device::types::{DeviceMethod, FullDeviceStatus};
use crate::ui::components::sidebar::AppSidebar;
use crate::ui::ui_types::{ActiveView, GlobalDeviceState};
use crate::ui::{
    colors,
    views::{
        about::AboutView, config::ConfigView, home::HomeView, logs::LogsView,
        passkeys::PasskeysView, security::SecurityView,
    },
};
use gpui::WeakEntity;
use gpui::*;
use gpui_component::{
    ActiveTheme, IconName, TitleBar,
    button::{Button, ButtonVariants},
    h_flex,
    scroll::ScrollableElement,
    v_flex,
};

pub struct ApplicationRoot {
    active_view: ActiveView,
    collapsed: bool,
    state: GlobalDeviceState,
    device_loading: bool,
    sidebar_width: Pixels,
}

impl ApplicationRoot {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            active_view: ActiveView::Home,
            collapsed: false,
            state: GlobalDeviceState::new(),
            device_loading: false,
            sidebar_width: px(255.),
        };
        this.refresh_device_status(cx);
        this
    }

    fn refresh_device_status(&mut self, cx: &mut Context<Self>) {
        if self.device_loading {
            return;
        }

        self.device_loading = true;
        self.state.error = None;
        cx.notify();

        // Perform synchronous IO for now as requested/implied by structure suitable for this environment
        // ideal would be spawning a task but let's stick to simple first if IO is not blocking main thread too badly or if we accept it.
        // Actually, IO should be async or in background.
        // But since I cannot easily spawn async here without an executor passed or using gpui's spawn, let's try direct call.
        // If the user said "run the refresh command", and it's rust, IO blocks.
        // Let's use cx.spawn to run it in background.

        // Synchronous implementation to avoid GPUI async type issues
        match io::read_device_details() {
            Ok(status) => {
                self.state.device_status = Some(status);
                self.state.error = None;

                // If successful, try to get FIDO info
                match io::get_fido_info() {
                    Ok(fido) => {
                        self.state.fido_info = Some(fido);
                    }
                    Err(e) => {
                        eprintln!("FIDO Info fetch failed: {}", e);
                        self.state.fido_info = None;
                    }
                }
            }
            Err(e) => {
                self.state.device_status = None;
                self.state.error = Some(format!("{}", e));
                self.state.fido_info = None;
            }
        }
        self.device_loading = false;
        cx.notify();
    }
}

impl Render for ApplicationRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let target_width = if self.collapsed { px(48.) } else { px(255.) };
        if (self.sidebar_width - target_width).abs() > px(0.1) {
            self.sidebar_width = self.sidebar_width + (target_width - self.sidebar_width) * 0.2;
            window.request_animation_frame();
        } else {
            self.sidebar_width = target_width;
        }

        div().size_full().overflow_hidden().child(
            h_flex()
                .size_full()
                .child(
                    AppSidebar::new(
                        self.active_view,
                        self.sidebar_width,
                        self.collapsed,
                        self.state.clone(),
                    )
                    .on_select(|this: &mut Self, view, _, _| {
                        this.active_view = view;
                    })
                    .on_refresh(|this: &mut Self, _, cx| {
                        this.refresh_device_status(cx);
                    })
                    .render(cx),
                )
                .child(
                    v_flex()
                        .size_full()
                        .child(
                            TitleBar::new().bg(rgba(colors::zinc::ZINC950)).child(
                                h_flex()
                                    .w_full()
                                    .justify_between()
                                    .bg(rgba(colors::zinc::ZINC950))
                                    .items_center()
                                    .cursor(gpui::CursorStyle::OpenHand)
                                    .child(
                                        Button::new("sidebar_toggle")
                                            .ghost()
                                            .icon(IconName::PanelLeft)
                                            .on_click(cx.listener(|this, _, _, _| {
                                                this.collapsed = !this.collapsed;
                                            }))
                                            .tooltip("Toggle Sidebar"),
                                    ),
                            ),
                        )
                        .child(
                            v_flex()
                                .min_h(px(0.))
                                .min_w(px(0.))
                                .overflow_y_scrollbar()
                                .flex_grow()
                                .bg(cx.theme().background)
                                .child(match self.active_view {
                                    ActiveView::Home => {
                                        HomeView::build(&self.state, cx.theme()).into_any_element()
                                    }
                                    ActiveView::Passkeys => {
                                        PasskeysView::build(cx.theme()).into_any_element()
                                    }
                                    ActiveView::Configuration => {
                                        cx.new(|cx| ConfigView::new(window, cx)).into_any_element()
                                    }
                                    ActiveView::Security => {
                                        SecurityView::build(cx).into_any_element()
                                    }
                                    ActiveView::Logs => {
                                        cx.new(|cx| LogsView::new(window, cx)).into_any_element()
                                    }
                                    ActiveView::About => {
                                        AboutView::build(cx.theme()).into_any_element()
                                    }
                                }),
                        ),
                ),
        )
    }
}
