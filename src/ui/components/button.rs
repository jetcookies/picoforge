// TODO: convert this from an entity to a stateless button component.

use crate::ui::colors;
use gpui::prelude::*;
use gpui::*;
use gpui_component::{
    Icon,
    button::{Button, ButtonCustomVariant, ButtonVariants},
    h_flex,
};

pub struct Clicked;

pub struct PFButton {
    text: SharedString,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut Context<Self>) + 'static>>,
    hovered: bool,
    hover_t: f32,
    bg_color_start: Rgba,
    bg_color_hover: Rgba,
    bg_color_active: Rgba,
    width_full: bool,
    centered: bool,
}

impl PFButton {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            on_click: None,
            hovered: false,
            hover_t: 0.0,
            bg_color_start: rgb(0x1b1b1d),
            bg_color_hover: rgb(0x232325),
            bg_color_active: rgb(colors::zinc::ZINC700),
            width_full: false,
            centered: false,
        }
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut Context<Self>) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn section_header(mut self) -> Self {
        self.width_full = true;
        self.centered = false;
        self
    }

    pub fn w_full(mut self) -> Self {
        self.width_full = true;
        self
    }

    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }
}

impl Render for PFButton {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let target_hover = if self.hovered { 1.0 } else { 0.0 };
        if (self.hover_t - target_hover).abs() > 0.01 {
            self.hover_t += (target_hover - self.hover_t) * 0.2;
            window.request_animation_frame();
        } else {
            self.hover_t = target_hover;
        }

        let c1 = self.bg_color_start;
        let c2 = self.bg_color_hover;
        let t = self.hover_t;
        let bg_color = Rgba {
            r: c1.r + (c2.r - c1.r) * t,
            g: c1.g + (c2.g - c1.g) * t,
            b: c1.b + (c2.b - c1.b) * t,
            a: 1.0,
        };

        let text = self.text.clone();

        let mut btn = Button::new("pf-btn").custom(
            ButtonCustomVariant::new(cx)
                .color(bg_color.into())
                .hover(bg_color.into())
                .active(self.bg_color_active.into()),
        );

        if self.width_full {
            btn = btn.w_full();
        }

        let content = if self.centered {
            h_flex().justify_center().child(text)
        } else {
            h_flex().child(text)
        };

        let on_click = self.on_click.take();
        if let Some(handler) = on_click {
            btn = btn.on_click(cx.listener(move |_, event, window, cx| handler(event, window, cx)));
        }

        div()
            .child(btn.child(content))
            .id("pf-btn-wrapper")
            .on_hover(cx.listener(|this, hovered, _, cx| {
                if this.hovered != *hovered {
                    this.hovered = *hovered;
                    cx.notify();
                }
            }))
    }
}

pub struct PFIconButton {
    icon: Icon,
    text: SharedString,
    hovered: bool,
    hover_t: f32,
    bg_color_start: Rgba,
    bg_color_hover: Rgba,
    bg_color_active: Rgba,
}

impl PFIconButton {
    pub fn new(icon: impl Into<Icon>, text: impl Into<SharedString>) -> Self {
        Self {
            icon: icon.into(),
            text: text.into(),
            hovered: false,
            hover_t: 0.0,
            bg_color_start: rgb(0x1b1b1d),
            bg_color_hover: rgb(0x232325),
            bg_color_active: rgb(colors::zinc::ZINC700),
        }
    }
}

impl EventEmitter<Clicked> for PFIconButton {}

impl Render for PFIconButton {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let target_hover = if self.hovered { 1.0 } else { 0.0 };
        if (self.hover_t - target_hover).abs() > 0.01 {
            self.hover_t += (target_hover - self.hover_t) * 0.2;
            window.request_animation_frame();
        } else {
            self.hover_t = target_hover;
        }

        let c1 = self.bg_color_start;
        let c2 = self.bg_color_hover;
        let t = self.hover_t;
        let bg_color = Rgba {
            r: c1.r + (c2.r - c1.r) * t,
            g: c1.g + (c2.g - c1.g) * t,
            b: c1.b + (c2.b - c1.b) * t,
            a: 1.0,
        };

        let text = self.text.clone();
        let icon = self.icon.clone();

        let btn = Button::new("pf-icon-btn")
            .custom(
                ButtonCustomVariant::new(cx)
                    .color(bg_color.into())
                    .hover(bg_color.into())
                    .active(self.bg_color_active.into()),
            )
            .w_full()
            .on_click(cx.listener(|_, _, _, cx| {
                cx.emit(Clicked);
            }));

        div()
            .child(btn.child(h_flex().gap_2().justify_center().child(icon).child(text)))
            .id("pf-icon-btn-wrapper")
            .on_hover(cx.listener(|this, hovered, _, cx| {
                if this.hovered != *hovered {
                    this.hovered = *hovered;
                    cx.notify();
                }
            }))
    }
}
