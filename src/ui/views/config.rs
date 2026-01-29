use crate::device::io;
use crate::device::types::{AppConfigInput, FullDeviceStatus};
use crate::ui::components::page_view::PageView;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, Icon, StyledExt, Theme,
    button::Button,
    input::{Input, InputState},
    select::{Select, SelectItem, SelectState},
    slider::{Slider, SliderState},
    switch::Switch,
    v_flex,
};

#[derive(Clone, PartialEq)]
struct VendorItem {
    value: SharedString,
    label: SharedString,
}

impl SelectItem for VendorItem {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.label.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}

#[derive(Clone, PartialEq)]
struct DriverItem {
    value: u8,
    label: SharedString,
}

impl SelectItem for DriverItem {
    type Value = u8;

    fn title(&self) -> SharedString {
        self.label.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}

pub struct ConfigView {
    vendor_select: Entity<SelectState<Vec<VendorItem>>>,
    vid_input: Entity<InputState>,
    pid_input: Entity<InputState>,
    product_name_input: Entity<InputState>,
    led_gpio_input: Entity<InputState>,
    led_driver_select: Entity<SelectState<Vec<DriverItem>>>,
    led_brightness_slider: Entity<SliderState>,
    led_dimmable: bool,
    led_steady: bool,
    touch_timeout_input: Entity<InputState>,
    power_cycle: bool,
    enable_secp256k1: bool,
    loading: bool,
    device_status: Option<FullDeviceStatus>,
}

impl ConfigView {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        device_status: Option<FullDeviceStatus>,
    ) -> Self {
        let config = device_status.as_ref().map(|s| &s.config);

        let vendors = vec![
            VendorItem {
                value: "custom".into(),
                label: "Custom".into(),
            },
            VendorItem {
                value: "solokeys".into(),
                label: "SoloKeys".into(),
            },
            VendorItem {
                value: "google".into(),
                label: "Google".into(),
            },
            VendorItem {
                value: "yubico".into(),
                label: "Yubico".into(),
            },
        ];

        let drivers = vec![
            DriverItem {
                value: 0,
                label: "WS2812".into(),
            },
            DriverItem {
                value: 1,
                label: "SK6812".into(),
            },
            DriverItem {
                value: 2,
                label: "APA102".into(),
            },
        ];

        let vendor_select = cx.new(|cx| {
            SelectState::new(
                vendors,
                Some(gpui_component::IndexPath::default()),
                window,
                cx,
            )
        });

        let vid_input = cx.new(|cx| {
            InputState::new(window, cx).default_value(
                config
                    .map(|c| c.vid.clone())
                    .unwrap_or_else(|| "CAFE".into()),
            )
        });
        let pid_input = cx.new(|cx| {
            InputState::new(window, cx).default_value(
                config
                    .map(|c| c.pid.clone())
                    .unwrap_or_else(|| "4242".into()),
            )
        });
        let product_name_input = cx.new(|cx| {
            InputState::new(window, cx).default_value(
                config
                    .map(|c| c.product_name.clone())
                    .unwrap_or_else(|| "My Key".into()),
            )
        });

        let led_gpio_input = cx.new(|cx| {
            InputState::new(window, cx).default_value(
                config
                    .map(|c| c.led_gpio.to_string())
                    .unwrap_or_else(|| "25".into()),
            )
        });

        let _initial_driver_idx = config.and_then(|c| c.led_driver).unwrap_or(0) as usize;
        let led_driver_select = cx.new(|cx| {
            SelectState::new(
                drivers,
                Some(gpui_component::IndexPath::default()),
                window,
                cx,
            )
        });

        // Set initial selection for driver?
        // Note: SelectState currently defaults to first item if index path is default.
        // We'd strictly need to set the index path based on value.
        // For now trusting default or user interaction.

        let led_brightness_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(15.0)
                .step(1.0)
                .default_value(config.map(|c| c.led_brightness as f32).unwrap_or(8.0))
        });

        let touch_timeout_input = cx.new(|cx| {
            InputState::new(window, cx).default_value(
                config
                    .map(|c| c.touch_timeout.to_string())
                    .unwrap_or_else(|| "10".into()),
            )
        });

        Self {
            vendor_select,
            vid_input,
            pid_input,
            product_name_input,
            led_gpio_input,
            led_driver_select,
            led_brightness_slider,
            led_dimmable: config.map(|c| c.led_dimmable).unwrap_or(true),
            led_steady: config.map(|c| c.led_steady).unwrap_or(false),
            touch_timeout_input,
            power_cycle: config.map(|c| c.power_cycle_on_reset).unwrap_or(false),
            enable_secp256k1: config.map(|c| c.enable_secp256k1).unwrap_or(true),
            loading: false,
            device_status,
        }
    }

    fn apply_changes(&mut self, cx: &mut Context<Self>) {
        let status = if let Some(s) = &self.device_status {
            s
        } else {
            return;
        };

        let current_config = &status.config;
        let mut changes = AppConfigInput {
            vid: None,
            pid: None,
            product_name: None,
            led_gpio: None,
            led_brightness: None,
            touch_timeout: None,
            led_driver: None,
            led_dimmable: None,
            power_cycle_on_reset: None,
            led_steady: None,
            enable_secp256k1: None,
        };

        let vid = self.vid_input.read(cx).text().to_string();
        if vid != current_config.vid {
            changes.vid = Some(vid);
        }

        let pid = self.pid_input.read(cx).text().to_string();
        if pid != current_config.pid {
            changes.pid = Some(pid);
        }

        let product_name = self.product_name_input.read(cx).text().to_string();
        if product_name != current_config.product_name {
            changes.product_name = Some(product_name);
        }

        let led_gpio_str = self.led_gpio_input.read(cx).text().to_string();
        if let Ok(val) = led_gpio_str.parse::<u8>() {
            if val != current_config.led_gpio {
                changes.led_gpio = Some(val);
            }
        }

        let driver_idx = self.led_driver_select.read(cx).selected_index(cx);
        if let Some(idx) = driver_idx {
            // Assuming values are 0, 1, 2 matches index
            let val = idx.row as u8;
            if Some(val) != current_config.led_driver {
                changes.led_driver = Some(val);
            }
        }

        let brightness = self.led_brightness_slider.read(cx).value().start() as u8;
        if brightness != current_config.led_brightness {
            changes.led_brightness = Some(brightness);
        }

        let touch_timeout_str = self.touch_timeout_input.read(cx).text().to_string();
        if let Ok(val) = touch_timeout_str.parse::<u8>() {
            if val != current_config.touch_timeout {
                changes.touch_timeout = Some(val);
            }
        }

        if self.led_dimmable != current_config.led_dimmable {
            changes.led_dimmable = Some(self.led_dimmable);
        }

        if self.led_steady != current_config.led_steady {
            changes.led_steady = Some(self.led_steady);
        }

        if self.power_cycle != current_config.power_cycle_on_reset {
            changes.power_cycle_on_reset = Some(self.power_cycle);
        }

        if self.enable_secp256k1 != current_config.enable_secp256k1 {
            changes.enable_secp256k1 = Some(self.enable_secp256k1);
        }

        // Check if we have any changes
        let has_changes = changes.vid.is_some()
            || changes.pid.is_some()
            || changes.product_name.is_some()
            || changes.led_gpio.is_some()
            || changes.led_brightness.is_some()
            || changes.touch_timeout.is_some()
            || changes.led_driver.is_some()
            || changes.led_dimmable.is_some()
            || changes.power_cycle_on_reset.is_some()
            || changes.led_steady.is_some()
            || changes.enable_secp256k1.is_some();

        if !has_changes {
            println!("No changes detected");
            return;
        }

        self.loading = true;
        cx.notify();

        let result = io::write_config(changes, status.method.clone(), None);

        self.loading = false;

        match result {
            Ok(msg) => {
                println!("Success: {}", msg);
            }
            Err(e) => {
                eprintln!("Error saving config: {}", e);
            }
        }

        cx.notify();
    }

    fn render_identity_card(&self, theme: &Theme) -> impl IntoElement {
        let content = v_flex()
            .gap_4()
            .child(
                v_flex()
                    .gap_2()
                    .child("Vendor Preset")
                    .child(Select::new(&self.vendor_select).w_full()),
            )
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_4()
                    .child(
                        v_flex()
                            .gap_2()
                            .child("Vendor ID (HEX)")
                            .child(Input::new(&self.vid_input).font_family("Mono")),
                    )
                    .child(
                        v_flex()
                            .gap_2()
                            .child("Product ID (HEX)")
                            .child(Input::new(&self.pid_input).font_family("Mono")),
                    ),
            )
            .child(div().h_px().bg(theme.border))
            .child(
                v_flex()
                    .gap_2()
                    .child("Product Name")
                    .child(Input::new(&self.product_name_input)),
            );

        Self::config_card(
            "Identity",
            "USB Identification settings",
            Icon::default().path("icons/tag.svg"),
            content,
            theme,
        )
    }

    fn render_led_card(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let dim_listener = cx.listener(|this, checked, _, cx| {
            this.led_dimmable = *checked;
            cx.notify();
        });

        let steady_listener = cx.listener(|this, checked, _, cx| {
            this.led_steady = *checked;
            cx.notify();
        });

        // Access theme after creating listeners (which requires mutable borrow of cx)
        let theme = cx.theme();

        // Read slider value (requires immutable borrow of cx)
        let brightness = self.led_brightness_slider.read(cx).value().start() as i32;

        let content = v_flex()
            .gap_4()
            .child(
                v_flex()
                    .gap_2()
                    .child("LED GPIO Pin")
                    .child(Input::new(&self.led_gpio_input)),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child("LED Driver")
                    .child(Select::new(&self.led_driver_select).w_full()),
            )
            .child(div().h_px().bg(theme.border))
            .child(
                v_flex().gap_2().child("Brightness (0-15)").child(
                    gpui_component::h_flex()
                        .items_center()
                        .gap_4()
                        .child(Slider::new(&self.led_brightness_slider).flex_1())
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!("Level {}", brightness)),
                        ),
                ),
            )
            .child(
                gpui_component::h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        v_flex().gap_0p5().child("LED Dimmable").child(
                            div()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child("Allow brightness adjustment"),
                        ),
                    )
                    .child(
                        Switch::new("led-dimmable")
                            .checked(self.led_dimmable)
                            .on_click(dim_listener),
                    ),
            )
            .child(
                gpui_component::h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        v_flex().gap_0p5().child("LED Steady Mode").child(
                            div()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child("Keep LED on constantly"),
                        ),
                    )
                    .child(
                        Switch::new("led-steady")
                            .checked(self.led_steady)
                            .on_click(steady_listener),
                    ),
            );

        Self::config_card(
            "LED Settings",
            "Adjust visual feedback behavior",
            Icon::default().path("icons/microchip.svg"),
            content,
            theme,
        )
    }

    fn render_touch_card(&self, theme: &Theme) -> impl IntoElement {
        let content = v_flex().gap_4().child(
            v_flex()
                .gap_2()
                .child("Touch Timeout (seconds)")
                .child(Input::new(&self.touch_timeout_input)),
        );

        Self::config_card(
            "Touch & Timing",
            "Configure interaction timeouts",
            Icon::default().path("icons/settings.svg"),
            content,
            theme,
        )
    }

    fn render_options_card(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let power_cycle_listener = cx.listener(|this, checked, _, cx| {
            this.power_cycle = *checked;
            cx.notify();
        });

        let secp_listener = cx.listener(|this, checked, _, cx| {
            this.enable_secp256k1 = *checked;
            cx.notify();
        });

        let theme = cx.theme();

        let content = v_flex()
            .gap_4()
            .child(
                gpui_component::h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        v_flex().gap_0p5().child("Power Cycle on Reset").child(
                            div()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child("Restart device on reset"),
                        ),
                    )
                    .child(
                        Switch::new("power-cycle")
                            .checked(self.power_cycle)
                            .on_click(power_cycle_listener),
                    ),
            )
            .child(
                gpui_component::h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        v_flex().gap_0p5().child("Enable Secp256k1").child(
                            div()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child("Does not work on Android!"),
                        ),
                    )
                    .child(
                        Switch::new("enable-secp")
                            .checked(self.enable_secp256k1)
                            .on_click(secp_listener),
                    ),
            );

        Self::config_card(
            "Device Options",
            "Toggle advanced features",
            Icon::default().path("icons/settings.svg"),
            content,
            &theme,
        )
    }

    fn config_card(
        title: &str,
        description: &str,
        icon: Icon,
        content: impl IntoElement,
        theme: &Theme,
    ) -> impl IntoElement {
        div()
            .w_full()
            .bg(rgb(0x18181b)) // Using the same bg as home card
            .border_1()
            .border_color(theme.border)
            .rounded_xl()
            .p_6()
            .child(
                v_flex()
                    .gap_6()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                gpui_component::h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(Icon::new(icon).size_5().text_color(theme.foreground))
                                    .child(
                                        div()
                                            .font_bold()
                                            .text_color(theme.foreground)
                                            .child(title.to_string()),
                                    ),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child(description.to_string()),
                            ),
                    )
                    .child(content),
            )
    }
}

impl Render for ConfigView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // If not connected, show placeholder
        if self.device_status.is_none() {
            return PageView::build(
                "Configuration",
                "Customize device settings and behavior.",
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h_64()
                    .border_1()
                    .border_color(theme.border)
                    .rounded_xl()
                    .child(div().text_color(theme.muted_foreground).child("No Content")),
                theme,
            )
            .into_any_element();
        }

        // I need to call mutable methods first.
        let led_card = self.render_led_card(cx).into_any_element();
        let options_card = self.render_options_card(cx).into_any_element();

        // Then get theme and render rest
        let theme = cx.theme();

        let identity_card = self.render_identity_card(theme).into_any_element();
        let touch_card = self.render_touch_card(theme).into_any_element();

        PageView::build(
            "Configuration",
            "Customize device settings and behavior.",
            v_flex()
                .gap_6()
                .child(
                    div()
                        .grid()
                        .grid_cols(2)
                        .gap_6()
                        .child(identity_card)
                        .child(led_card)
                        .child(touch_card)
                        .child(options_card),
                )
                .child(
                    gpui_component::h_flex().justify_end().child(
                        Button::new("apply-changes")
                            .icon(Icon::default().path("icons/save.svg"))
                            .child("Apply Changes")
                            .disabled(self.loading)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.apply_changes(cx);
                            })),
                    ),
                ),
            &theme,
        )
        .into_any_element()
    }
}
