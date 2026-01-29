use crate::device::types::{FidoDeviceInfo, FullDeviceStatus};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActiveView {
    Home,
    Passkeys,
    Configuration,
    Security,
    Logs,
    About,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalDeviceState {
    pub device_status: Option<FullDeviceStatus>,
    pub fido_info: Option<FidoDeviceInfo>,
    pub error: Option<String>,
}

impl GlobalDeviceState {
    pub fn new() -> Self {
        Self {
            device_status: None,
            fido_info: None,
            error: None,
        }
    }
}
