//! Items specific to the "native" implementation.

#![cfg_attr(nightly, feature(doc_cfg))]

pub mod gpio;

#[cfg(feature = "hwrng")]
pub mod hwrng;

pub mod identity;
pub mod peripherals {
    use std::sync::{Arc, Mutex, mpsc};

    //. Asynchronous stream of emulated hardware outputs
    pub struct OutStream<T> {
        /// The emulated board manager inside the user's application should listen on this for incoming messages and handle them
        pub recv: Arc<Mutex<mpsc::Receiver<T>>>,
        /// The native board will push events to the manager using this
        pub(crate) sender: mpsc::Sender<T>,
    }

    //. Asynchronous stream of emulated hardware inputs
    pub struct InStream<T> {
        /// The emulated board manager inside the user's application can use this to push events to the application.
        pub sender: mpsc::Sender<T>,
        /// The native board uses this to listen to events coming from the manager
        pub(crate) recv: Arc<Mutex<mpsc::Receiver<T>>>,
    }

    use crate::gpio::output::OutputPin;
    use crate::gpio::input::InputPin;
    pub struct GPIO0;
    pub struct GPIO1;

    impl OutputPin for GPIO0 {
        const OUT_PIN_NUMBER: usize = 0;
    }

    impl InputPin for GPIO1 {
        const IN_PIN_NUMBER: usize = 0;
    }
}

#[allow(non_snake_case)]
pub struct OptionalPeripherals {
    pub GPIO0: Option<crate::peripheral::Peri<'static, peripherals::GPIO0>>,
    pub GPIO1: Option<crate::peripheral::Peri<'static, peripherals::GPIO1>>,
}

#[must_use]
pub fn init() -> OptionalPeripherals {
    OptionalPeripherals {
        GPIO0: Some(crate::peripheral::Peri::empty()),
        GPIO1: Some(crate::peripheral::Peri::empty()),
    }
}

pub use ariel_os_dummy::peripheral;

pub trait IntoPeripheral<'a, P> {//: private::Sealed {
    /// Converts this peripheral instance into the type required by the HAL.
    #[must_use]
    fn into_hal_peripheral(self) -> crate::peripheral::Peri<'a, P>;
}

pub struct SWI {}

pub mod tuntap {
    pub type NetworkDevice = embassy_net_tuntap::TunTapDevice;

    /// Creates a TUN/TAP network device as configured in the environment.
    ///
    /// Unlike the hardware device initializers that are inline in `ariel_os_embassy::init_task`,
    /// this doesn't work automatically from init'ing with peripherals and a spawner, but goes
    /// through extra steps of finding which device to actually use.
    ///
    /// # Panics
    ///
    /// This function panics if the network interface can not be opened (eg. for lack of
    /// permission, when it does not exist, or when it is in use).
    #[must_use]
    pub fn create() -> NetworkDevice {
        let ifname = std::env::var("ARIEL_NATIVE_TUNTAP").unwrap_or_else(|_| "tap0".to_owned());
        match NetworkDevice::new(&ifname) {
            Ok(d) => d,
            Err(e) => panic!("Error opening interface {ifname}: {e}"),
        }
    }
}
