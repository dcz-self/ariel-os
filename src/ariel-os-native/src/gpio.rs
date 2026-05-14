pub use ariel_os_dummy::peripheral::Peri;

//impl private::Sealed for Peripheral {}

impl<T> crate::IntoPeripheral<'_, T> for Peri<'static, T> {
    fn into_hal_peripheral(self) -> Self {
        self
    }
}

pub mod input {
    use std::sync::{Arc, LazyLock, Mutex, mpsc};
    use ariel_os_embassy_common::gpio::Level;
    use embedded_hal::digital::InputPin as HalInputPin;
    use crate::peripherals::InStream;

    // TODO: send configuration: pullups, hi-Z, etc.
    // TODO: use a single atomic int
    pub static STATES: LazyLock<[Arc<Mutex<PinState>>; 1]> = LazyLock::new(|| {
        let init = || {
            Arc::new(Mutex::new(PinState { level: Level::Low }))
        };
        [init()]
    });

    pub trait InputPin {
        /// Index to the STREAMS array
        const IN_PIN_NUMBER: usize;
    }

    pub const SCHMITT_TRIGGER_CONFIGURABLE: bool = false;

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub struct PinState {
        pub level: Level,
    }

    pub struct Input<'d> {
        _marker: core::marker::PhantomData<&'d ()>,
        pin_number: usize,
    }

    impl Input<'_> {
        #[must_use]
        pub fn is_high(&self) -> bool {
            *STATES[self.pin_number].lock().unwrap() == PinState { level: Level::High }
        }

        #[must_use]
        pub fn is_low(&self) -> bool {
            *STATES[self.pin_number].lock().unwrap() == PinState { level: Level::Low }
        }

        #[must_use]
        pub fn get_level(&self) -> crate::gpio::input::Level {
            STATES[self.pin_number].lock().unwrap().level
        }

        pub async fn wait_for_high(&mut self) {
            unimplemented!();
        }

        pub async fn wait_for_low(&mut self) {
            unimplemented!();
        }

        pub async fn wait_for_rising_edge(&mut self) {
            unimplemented!();
        }

        pub async fn wait_for_falling_edge(&mut self) {
            unimplemented!();
        }

        pub async fn wait_for_any_edge(&mut self) {
            unimplemented!();
        }
    }

    impl embedded_hal::digital::ErrorType for Input<'_> {
        type Error = core::convert::Infallible;
    }

    impl embedded_hal::digital::InputPin for Input<'_> {
        fn is_low(&mut self) -> Result<bool, Self::Error> {
            todo!();
        }

        fn is_high(&mut self) -> Result<bool, Self::Error> {
            todo!();
        }
    }

    pub fn new<'a, T: InputPin>(
        pin: super::Peri<'a, T>,
        _pull: ariel_os_embassy_common::gpio::Pull,
        _schmitt_trigger: bool,
    ) -> Result<Input<'a>, ariel_os_embassy_common::gpio::input::Error> {
        let state = PinState { level: Level::Low };
        let ret = Input {
            _marker: Default::default(),
            // Don't carry the comm channel.
            // The channel should survive even if the pin gets destroyed.
            pin_number: T::IN_PIN_NUMBER,
        };
        // FIXME: notify the manager about the new pullup config
        //ret.send_update(state);
        Ok(ret)
    }

    ariel_os_embassy_common::define_into_level!();
}

pub mod output {
    use std::sync::{Arc, LazyLock, Mutex, mpsc};
    use ariel_os_embassy_common::gpio::Level;
    use embedded_hal::digital::StatefulOutputPin;

    use crate::peripherals::OutStream;

    pub static STREAMS: LazyLock<[OutStream<PinState>; 1]> = LazyLock::new(|| {
        let init = || {
            let (sender, recv) = mpsc::channel();
            OutStream {
                recv: Arc::new(Mutex::new(recv)),
                sender,
            }
        };
        [init()]
    });

    pub trait OutputPin {
        /// Index to the STREAMS array
        const OUT_PIN_NUMBER: usize;
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub struct PinState {
        pub level: ariel_os_embassy_common::gpio::Level,
        // TODO: add drive strength and other stuff
    }

    pub const DRIVE_STRENGTH_CONFIGURABLE: bool = false;
    pub const SPEED_CONFIGURABLE: bool = false;

    pub fn new<'a, T: OutputPin>(
        pin: super::Peri<'a, T>,
        initial_level: ariel_os_embassy_common::gpio::Level,
        _drive_strength: super::DriveStrength,
        _speed: super::Speed,
    ) -> Output<'a> {
        let state = PinState { level: initial_level };
        let ret = Output {
            _marker: Default::default(),
            // Don't carry the channel.
            // The channel should survive even if the pin gets destroyed.
            pin_number: T::OUT_PIN_NUMBER,
            state: state.clone(),
        };
        ret.send_update(state);
        ret
    }

    pub struct Output<'d> {
        _marker: core::marker::PhantomData<&'d ()>,
        pin_number: usize,
        state: PinState,
    }

    impl<'d> Output<'d> {
        fn send_update(&self, event: PinState) {
            STREAMS[self.pin_number].sender.send(event).unwrap();
        }
    }

    impl embedded_hal::digital::ErrorType for Output<'_> {
        type Error = core::convert::Infallible;
    }

    impl embedded_hal::digital::OutputPin for Output<'_> {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.state.level = Level::Low;
            self.send_update(self.state.clone());
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.state.level = Level::High;
            self.send_update(self.state.clone());
            Ok(())
        }
    }

    impl StatefulOutputPin for Output<'_> {
        fn is_set_high(&mut self) -> Result<bool, Self::Error> {
            Ok(self.state.level == Level::High)
        }

        fn is_set_low(&mut self) -> Result<bool, Self::Error> {
            Ok(self.state.level == Level::Low)
        }
    }
}

    /// Actual type is HAL-specific.
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum DriveStrength {
        #[doc(hidden)]
        Hidden,
    }

    impl ariel_os_embassy_common::gpio::FromDriveStrength for DriveStrength {
        fn from(_drive_strength: ariel_os_embassy_common::gpio::DriveStrength<Self>) -> Self {
            Self::Hidden
        }
    }

    /// Actual type is HAL-specific.
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum Speed {
        #[doc(hidden)]
        Hidden,
    }

    impl ariel_os_embassy_common::gpio::FromSpeed for Speed {
        fn from(_speed: ariel_os_embassy_common::gpio::Speed<Self>) -> Self {
            Self::Hidden
        }
    }
