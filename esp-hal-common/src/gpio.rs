//! General Purpose I/Os
//!
//! To get access to the pins, you first need to convert them into a HAL
//! designed struct from the pac struct `GPIO` and `IO_MUX` using `IO::new`.
//!
//! ```no_run
//! let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
//! let mut led = io.pins.gpio5.into_push_pull_output();
//! ```

use core::marker::PhantomData;

#[doc(hidden)]
#[cfg_attr(esp32, path = "gpio/esp32.rs")]
#[cfg_attr(esp32c2, path = "gpio/esp32c2.rs")]
#[cfg_attr(esp32c3, path = "gpio/esp32c3.rs")]
#[cfg_attr(esp32s2, path = "gpio/esp32s2.rs")]
#[cfg_attr(esp32s3, path = "gpio/esp32s3.rs")]
pub mod types;

use core::convert::Infallible;

pub use crate::types::*;
use crate::{
    pac::{GPIO, IO_MUX},
    types::{
        get_io_mux_reg,
        gpio_intr_enable,
        OutputSignalType,
        GPIO_FUNCTION,
        INPUT_SIGNAL_MAX,
        OUTPUT_SIGNAL_MAX,
    },
};

#[derive(Copy, Clone)]
pub enum Event {
    RisingEdge  = 1,
    FallingEdge = 2,
    AnyEdge     = 3,
    LowLevel    = 4,
    HighLevel   = 5,
}

pub struct Unknown {}

pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

pub struct RTCInput<MODE> {
    _mode: PhantomData<MODE>,
}

pub struct Floating;

pub struct PullDown;

pub struct PullUp;

pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

pub struct RTCOutput<MODE> {
    _mode: PhantomData<MODE>,
}

pub struct OpenDrain;

pub struct PushPull;

pub struct Analog;

pub struct Alternate<MODE> {
    _mode: PhantomData<MODE>,
}

#[doc(hidden)]
pub struct AF0;

#[doc(hidden)]
pub struct AF1;

#[doc(hidden)]
pub struct AF2;

pub enum DriveStrength {
    I5mA  = 0,
    I10mA = 1,
    I20mA = 2,
    I40mA = 3,
}

#[derive(PartialEq)]
pub enum AlternateFunction {
    Function0 = 0,
    Function1 = 1,
    Function2 = 2,
    Function3 = 3,
    Function4 = 4,
    Function5 = 5,
}

pub trait RTCPin {}

pub trait AnalogPin {}

pub trait Pin {
    fn number(&self) -> u8;

    fn sleep_mode(&mut self, on: bool) -> &mut Self;

    fn set_alternate_function(&mut self, alternate: AlternateFunction) -> &mut Self;

    fn listen(&mut self, event: Event) {
        self.listen_with_options(event, true, false, false)
    }

    fn listen_with_options(
        &mut self,
        event: Event,
        int_enable: bool,
        nmi_enable: bool,
        wake_up_from_light_sleep: bool,
    );

    fn unlisten(&mut self);

    fn clear_interrupt(&mut self);

    fn is_pcore_interrupt_set(&self) -> bool;

    fn is_pcore_non_maskable_interrupt_set(&self) -> bool;

    fn is_acore_interrupt_set(&self) -> bool;

    fn is_acore_non_maskable_interrupt_set(&self) -> bool;

    fn enable_hold(&mut self, on: bool);
}

pub trait InputPin: Pin {
    fn set_to_input(&mut self) -> &mut Self;

    fn enable_input(&mut self, on: bool) -> &mut Self;

    fn enable_input_in_sleep_mode(&mut self, on: bool) -> &mut Self;

    fn is_input_high(&self) -> bool;

    fn connect_input_to_peripheral(&mut self, signal: InputSignal) -> &mut Self {
        self.connect_input_to_peripheral_with_options(signal, false, false)
    }

    fn connect_input_to_peripheral_with_options(
        &mut self,
        signal: InputSignal,
        invert: bool,
        force_via_gpio_mux: bool,
    ) -> &mut Self;

    /// Remove a connected `signal` from this input pin.
    ///
    /// Clears the entry in the GPIO matrix / IO mux that associates this input
    /// pin with the given [input `signal`](`InputSignal`). Any other
    /// connected signals remain intact.
    fn disconnect_input_from_peripheral(&mut self, signal: InputSignal) -> &mut Self;
}

pub trait OutputPin: Pin {
    fn set_to_open_drain_output(&mut self) -> &mut Self;

    fn set_to_push_pull_output(&mut self) -> &mut Self;

    fn enable_output(&mut self, on: bool) -> &mut Self;

    fn set_output_high(&mut self, on: bool) -> &mut Self;

    fn set_drive_strength(&mut self, strength: DriveStrength) -> &mut Self;

    fn enable_open_drain(&mut self, on: bool) -> &mut Self;

    fn enable_output_in_sleep_mode(&mut self, on: bool) -> &mut Self;

    fn internal_pull_up_in_sleep_mode(&mut self, on: bool) -> &mut Self;

    fn internal_pull_down_in_sleep_mode(&mut self, on: bool) -> &mut Self;

    fn connect_peripheral_to_output(&mut self, signal: OutputSignal) -> &mut Self {
        self.connect_peripheral_to_output_with_options(signal, false, false, false, false)
    }

    fn connect_peripheral_to_output_with_options(
        &mut self,
        signal: OutputSignal,
        invert: bool,
        invert_enable: bool,
        enable_from_gpio: bool,
        force_via_gpio_mux: bool,
    ) -> &mut Self;

    /// Remove this output pin from a connected [signal](`InputSignal`).
    ///
    /// Clears the entry in the GPIO matrix / IO mux that associates this output
    /// pin with a previously connected [signal](`InputSignal`). Any other
    /// outputs connected to the signal remain intact.
    fn disconnect_peripheral_from_output(&mut self) -> &mut Self;

    fn internal_pull_up(&mut self, on: bool) -> &mut Self;

    fn internal_pull_down(&mut self, on: bool) -> &mut Self;
}

#[doc(hidden)]
pub struct SingleCoreInteruptStatusRegisterAccess {}
#[doc(hidden)]
pub struct DualCoreInteruptStatusRegisterAccess {}

#[doc(hidden)]
pub trait InteruptStatusRegisterAccess {
    fn pro_cpu_interrupt_status_read() -> u32;

    fn pro_cpu_nmi_status_read() -> u32;

    fn app_cpu_interrupt_status_read() -> u32;

    fn app_cpu_nmi_status_read() -> u32;
}

impl InteruptStatusRegisterAccess for SingleCoreInteruptStatusRegisterAccess {
    fn pro_cpu_interrupt_status_read() -> u32 {
        unsafe { &*GPIO::PTR }.pcpu_int.read().bits()
    }

    fn pro_cpu_nmi_status_read() -> u32 {
        unsafe { &*GPIO::PTR }.pcpu_nmi_int.read().bits()
    }

    fn app_cpu_interrupt_status_read() -> u32 {
        unsafe { &*GPIO::PTR }.pcpu_int.read().bits()
    }

    fn app_cpu_nmi_status_read() -> u32 {
        unsafe { &*GPIO::PTR }.pcpu_nmi_int.read().bits()
    }
}

// ESP32S3 is a dual-core chip however pro cpu and app cpu shares the same
// interrupt enable bit see
// https://github.com/espressif/esp-idf/blob/c04803e88b871a4044da152dfb3699cf47354d18/components/hal/esp32s3/include/hal/gpio_ll.h#L32
// Treating it as SingleCore in the gpio macro makes this work.
#[cfg(not(any(esp32c2, esp32c3, esp32s2, esp32s3)))]
impl InteruptStatusRegisterAccess for DualCoreInteruptStatusRegisterAccess {
    fn pro_cpu_interrupt_status_read() -> u32 {
        unsafe { &*GPIO::PTR }.pcpu_int.read().bits()
    }

    fn pro_cpu_nmi_status_read() -> u32 {
        unsafe { &*GPIO::PTR }.pcpu_nmi_int.read().bits()
    }

    fn app_cpu_interrupt_status_read() -> u32 {
        unsafe { &*GPIO::PTR }.acpu_int.read().bits()
    }

    fn app_cpu_nmi_status_read() -> u32 {
        unsafe { &*GPIO::PTR }.acpu_nmi_int.read().bits()
    }
}

#[doc(hidden)]
pub trait InterruptStatusRegisters<RegisterAccess>
where
    RegisterAccess: InteruptStatusRegisterAccess,
{
    fn pro_cpu_interrupt_status_read(&self) -> u32 {
        RegisterAccess::pro_cpu_interrupt_status_read()
    }

    fn pro_cpu_nmi_status_read(&self) -> u32 {
        RegisterAccess::pro_cpu_nmi_status_read()
    }

    fn app_cpu_interrupt_status_read(&self) -> u32 {
        RegisterAccess::app_cpu_interrupt_status_read()
    }

    fn app_cpu_nmi_status_read(&self) -> u32 {
        RegisterAccess::app_cpu_nmi_status_read()
    }
}

#[doc(hidden)]
pub struct Bank0GpioRegisterAccess;

#[doc(hidden)]
pub struct Bank1GpioRegisterAccess;

#[doc(hidden)]
pub trait BankGpioRegisterAccess {
    fn write_out_en_clear(&self, word: u32);

    fn write_out_en_set(&self, word: u32);

    fn read_input(&self) -> u32;

    fn read_output(&self) -> u32;

    fn write_interrupt_status_clear(&self, word: u32);

    fn write_output_set(&self, word: u32);

    fn write_output_clear(&self, word: u32);

    fn set_output_signal(&self, gpio_num: u8, signal: u32) {
        let gpio = unsafe { &*crate::pac::GPIO::PTR };
        gpio.func_out_sel_cfg[gpio_num as usize]
            .modify(|_, w| unsafe { w.out_sel().bits(signal as OutputSignalType) });
    }

    fn configure_out_sel(&self, gpio_num: u8, signal: u32, invert: bool, oen: bool, oen_inv: bool) {
        let gpio = unsafe { &*crate::pac::GPIO::PTR };
        gpio.func_out_sel_cfg[gpio_num as usize].modify(|_, w| unsafe {
            w.out_sel()
                .bits(signal as OutputSignalType)
                .inv_sel()
                .bit(invert)
                .oen_sel()
                .bit(oen)
                .oen_inv_sel()
                .bit(oen_inv)
        });
    }

    fn set_signal_to_level(&self, signal: u32, high: bool) {
        let gpio = unsafe { &*crate::pac::GPIO::PTR };
        gpio.func_in_sel_cfg[signal as usize].modify(|_, w| unsafe {
            w.sel()
                .set_bit()
                .in_inv_sel()
                .bit(false)
                .in_sel()
                .bits(if high { ONE_INPUT } else { ZERO_INPUT })
        });
    }

    fn clear_func_in_sel(&self, signal: u32) {
        let gpio = unsafe { &*crate::pac::GPIO::PTR };
        gpio.func_in_sel_cfg[signal as usize].modify(|_, w| w.sel().clear_bit());
    }

    fn set_int_enable(
        &self,
        gpio_num: u8,
        int_ena: u32,
        int_type: u8,
        wake_up_from_light_sleep: bool,
    ) {
        let gpio = unsafe { &*crate::pac::GPIO::PTR };
        gpio.pin[gpio_num as usize].modify(|_, w| unsafe {
            w.int_ena()
                .bits(int_ena as u8)
                .int_type()
                .bits(int_type as u8)
                .wakeup_enable()
                .bit(wake_up_from_light_sleep)
        });
    }

    fn set_open_drain(&self, gpio_num: u8, open_drain: bool) {
        let gpio = unsafe { &*crate::pac::GPIO::PTR };
        gpio.pin[gpio_num as usize].modify(|_, w| w.pad_driver().bit(open_drain));
    }
}

impl BankGpioRegisterAccess for Bank0GpioRegisterAccess {
    fn write_out_en_clear(&self, word: u32) {
        unsafe { &*GPIO::PTR }
            .enable_w1tc
            .write(|w| unsafe { w.bits(word) });
    }

    fn write_out_en_set(&self, word: u32) {
        unsafe { &*GPIO::PTR }
            .enable_w1ts
            .write(|w| unsafe { w.bits(word) });
    }

    fn read_input(&self) -> u32 {
        unsafe { &*GPIO::PTR }.in_.read().bits()
    }

    fn read_output(&self) -> u32 {
        unsafe { &*GPIO::PTR }.out.read().bits()
    }

    fn write_interrupt_status_clear(&self, word: u32) {
        unsafe { &*GPIO::PTR }
            .status_w1tc
            .write(|w| unsafe { w.bits(word) });
    }

    fn write_output_set(&self, word: u32) {
        unsafe { &*GPIO::PTR }
            .out_w1ts
            .write(|w| unsafe { w.bits(word) });
    }

    fn write_output_clear(&self, word: u32) {
        unsafe { &*GPIO::PTR }
            .out_w1tc
            .write(|w| unsafe { w.bits(word) });
    }
}

#[cfg(not(any(esp32c2, esp32c3)))]
impl BankGpioRegisterAccess for Bank1GpioRegisterAccess {
    fn write_out_en_clear(&self, word: u32) {
        unsafe { &*GPIO::PTR }
            .enable1_w1tc
            .write(|w| unsafe { w.bits(word) });
    }

    fn write_out_en_set(&self, word: u32) {
        unsafe { &*GPIO::PTR }
            .enable1_w1ts
            .write(|w| unsafe { w.bits(word) });
    }

    fn read_input(&self) -> u32 {
        unsafe { &*GPIO::PTR }.in1.read().bits()
    }

    fn read_output(&self) -> u32 {
        unsafe { &*GPIO::PTR }.out1.read().bits()
    }

    fn write_interrupt_status_clear(&self, word: u32) {
        unsafe { &*GPIO::PTR }
            .status1_w1tc
            .write(|w| unsafe { w.bits(word) });
    }

    fn write_output_set(&self, word: u32) {
        unsafe { &*GPIO::PTR }
            .out1_w1ts
            .write(|w| unsafe { w.bits(word) });
    }

    fn write_output_clear(&self, word: u32) {
        unsafe { &*GPIO::PTR }
            .out1_w1tc
            .write(|w| unsafe { w.bits(word) });
    }
}

pub fn connect_low_to_peripheral(signal: InputSignal) {
    unsafe { &*GPIO::PTR }.func_in_sel_cfg[signal as usize].modify(|_, w| unsafe {
        w.sel()
            .set_bit()
            .in_inv_sel()
            .bit(false)
            .in_sel()
            .bits(ZERO_INPUT)
    });
}

pub fn connect_high_to_peripheral(signal: InputSignal) {
    unsafe { &*GPIO::PTR }.func_in_sel_cfg[signal as usize].modify(|_, w| unsafe {
        w.sel()
            .set_bit()
            .in_inv_sel()
            .bit(false)
            .in_sel()
            .bits(ONE_INPUT)
    });
}

#[doc(hidden)]
pub trait PinType {}

#[doc(hidden)]
pub trait IsOutputPin: PinType {}

#[doc(hidden)]
pub trait IsInputPin: PinType {}

#[doc(hidden)]
pub trait IsAnalogPin: PinType {}

#[doc(hidden)]
pub struct InputOutputPinType;

#[doc(hidden)]
pub struct InputOnlyPinType;

#[doc(hidden)]
pub struct InputOutputAnalogPinType;

#[doc(hidden)]
pub struct InputOnlyAnalogPinType;

impl PinType for InputOutputPinType {}
impl IsOutputPin for InputOutputPinType {}
impl IsInputPin for InputOutputPinType {}

impl PinType for InputOnlyPinType {}
impl IsInputPin for InputOnlyPinType {}

impl PinType for InputOutputAnalogPinType {}
impl IsOutputPin for InputOutputAnalogPinType {}
impl IsInputPin for InputOutputAnalogPinType {}
impl IsAnalogPin for InputOutputAnalogPinType {}

impl PinType for InputOnlyAnalogPinType {}
impl IsInputPin for InputOnlyAnalogPinType {}
impl IsAnalogPin for InputOnlyAnalogPinType {}

pub struct GpioPin<MODE, RA, PINTYPE, const GPIONUM: u8>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: PinType,
{
    _mode: PhantomData<MODE>,
    _pintype: PhantomData<PINTYPE>,
    reg_access: RA,
    af_input_signals: [Option<InputSignal>; 6],
    af_output_signals: [Option<OutputSignal>; 6],
}

impl<MODE, RA, PINTYPE, const GPIONUM: u8> embedded_hal::digital::v2::InputPin
    for GpioPin<Input<MODE>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: PinType,
{
    type Error = Infallible;
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.reg_access.read_input() & (1 << (GPIONUM % 32)) != 0)
    }
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

impl<RA, PINTYPE, const GPIONUM: u8> embedded_hal::digital::v2::InputPin
    for GpioPin<Output<OpenDrain>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: PinType,
{
    type Error = Infallible;
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.reg_access.read_input() & (1 << (GPIONUM % 32)) != 0)
    }
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

#[cfg(feature = "eh1")]
impl<MODE, RA, PINTYPE, const GPIONUM: u8> embedded_hal_1::digital::ErrorType
    for GpioPin<Input<MODE>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: PinType,
{
    type Error = Infallible;
}

#[cfg(feature = "eh1")]
impl<MODE, RA, PINTYPE, const GPIONUM: u8> embedded_hal_1::digital::InputPin
    for GpioPin<Input<MODE>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: PinType,
{
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.reg_access.read_input() & (1 << (GPIONUM % 32)) != 0)
    }
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

impl<MODE, RA, PINTYPE, const GPIONUM: u8> GpioPin<MODE, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: PinType,
{
    fn init_input(&self, pull_down: bool, pull_up: bool) {
        let gpio = unsafe { &*GPIO::PTR };

        self.reg_access.write_out_en_clear(1 << (GPIONUM % 32));
        gpio.func_out_sel_cfg[GPIONUM as usize]
            .modify(|_, w| unsafe { w.out_sel().bits(OutputSignal::GPIO as OutputSignalType) });

        #[cfg(esp32)]
        types::errata36(GPIONUM, pull_up, pull_down);

        get_io_mux_reg(GPIONUM).modify(|_, w| unsafe {
            w.mcu_sel()
                .bits(GPIO_FUNCTION as u8)
                .fun_ie()
                .set_bit()
                .fun_wpd()
                .bit(pull_down)
                .fun_wpu()
                .bit(pull_up)
                .slp_sel()
                .clear_bit()
        });
    }

    pub fn into_floating_input(self) -> GpioPin<Input<Floating>, RA, PINTYPE, GPIONUM> {
        self.init_input(false, false);
        GpioPin {
            _mode: PhantomData,
            _pintype: PhantomData,
            reg_access: self.reg_access,
            af_input_signals: self.af_input_signals,
            af_output_signals: self.af_output_signals,
        }
    }

    pub fn into_pull_up_input(self) -> GpioPin<Input<PullUp>, RA, PINTYPE, GPIONUM> {
        self.init_input(false, true);
        GpioPin {
            _mode: PhantomData,
            _pintype: PhantomData,
            reg_access: self.reg_access,
            af_input_signals: self.af_input_signals,
            af_output_signals: self.af_output_signals,
        }
    }

    pub fn into_pull_down_input(self) -> GpioPin<Input<PullDown>, RA, PINTYPE, GPIONUM> {
        self.init_input(true, false);
        GpioPin {
            _mode: PhantomData,
            _pintype: PhantomData,
            reg_access: self.reg_access,
            af_input_signals: self.af_input_signals,
            af_output_signals: self.af_output_signals,
        }
    }
}

impl<MODE, RA, PINTYPE, const GPIONUM: u8> InputPin for GpioPin<MODE, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: PinType,
{
    fn set_to_input(&mut self) -> &mut Self {
        self.init_input(false, false);
        self
    }
    fn enable_input(&mut self, on: bool) -> &mut Self {
        get_io_mux_reg(GPIONUM).modify(|_, w| w.fun_ie().bit(on));
        self
    }
    fn enable_input_in_sleep_mode(&mut self, on: bool) -> &mut Self {
        get_io_mux_reg(GPIONUM).modify(|_, w| w.mcu_ie().bit(on));
        self
    }
    fn is_input_high(&self) -> bool {
        self.reg_access.read_input() & (1 << (GPIONUM % 32)) != 0
    }
    fn connect_input_to_peripheral_with_options(
        &mut self,
        signal: InputSignal,
        invert: bool,
        force_via_gpio_mux: bool,
    ) -> &mut Self {
        let af = if force_via_gpio_mux {
            GPIO_FUNCTION
        } else {
            let mut res = GPIO_FUNCTION;
            for (i, input_signal) in self.af_input_signals.iter().enumerate() {
                if let Some(input_signal) = input_signal {
                    if *input_signal == signal {
                        res = match i {
                            0 => AlternateFunction::Function0,
                            1 => AlternateFunction::Function1,
                            2 => AlternateFunction::Function2,
                            3 => AlternateFunction::Function3,
                            4 => AlternateFunction::Function4,
                            5 => AlternateFunction::Function5,
                            _ => unreachable!(),
                        };
                        break;
                    }
                }
            }
            res
        };
        if af == GPIO_FUNCTION && signal as usize > INPUT_SIGNAL_MAX as usize {
            panic!("Cannot connect GPIO to this peripheral");
        }
        self.set_alternate_function(af);
        if (signal as usize) <= INPUT_SIGNAL_MAX as usize {
            unsafe { &*GPIO::PTR }.func_in_sel_cfg[signal as usize].modify(|_, w| unsafe {
                w.sel()
                    .set_bit()
                    .in_inv_sel()
                    .bit(invert)
                    .in_sel()
                    .bits(GPIONUM)
            });
        }
        self
    }

    fn disconnect_input_from_peripheral(&mut self, signal: InputSignal) -> &mut Self {
        self.set_alternate_function(GPIO_FUNCTION);

        unsafe { &*GPIO::PTR }.func_in_sel_cfg[signal as usize].modify(|_, w| w.sel().clear_bit());
        self
    }
}

impl<MODE, RA, PINTYPE, const GPIONUM: u8> Pin for GpioPin<MODE, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: PinType,
{
    fn number(&self) -> u8 {
        GPIONUM
    }

    fn sleep_mode(&mut self, on: bool) -> &mut Self {
        get_io_mux_reg(GPIONUM).modify(|_, w| w.slp_sel().bit(on));

        self
    }

    fn set_alternate_function(&mut self, alternate: AlternateFunction) -> &mut Self {
        get_io_mux_reg(GPIONUM).modify(|_, w| unsafe { w.mcu_sel().bits(alternate as u8) });
        self
    }

    fn listen_with_options(
        &mut self,
        event: Event,
        int_enable: bool,
        nmi_enable: bool,
        wake_up_from_light_sleep: bool,
    ) {
        if wake_up_from_light_sleep {
            match event {
                Event::AnyEdge | Event::RisingEdge | Event::FallingEdge => {
                    panic!("Edge triggering is not supported for wake-up from light sleep");
                }
                _ => {}
            }
        }
        unsafe {
            (&*GPIO::PTR).pin[GPIONUM as usize].modify(|_, w| {
                w.int_ena()
                    .bits(gpio_intr_enable(int_enable, nmi_enable))
                    .int_type()
                    .bits(event as u8)
                    .wakeup_enable()
                    .bit(wake_up_from_light_sleep)
            });
        }
    }

    fn unlisten(&mut self) {
        unsafe {
            (&*GPIO::PTR).pin[GPIONUM as usize]
                .modify(|_, w| w.int_ena().bits(0).int_type().bits(0).int_ena().bits(0));
        }
    }

    fn clear_interrupt(&mut self) {
        self.reg_access
            .write_interrupt_status_clear(1 << (GPIONUM % 32));
    }

    fn is_pcore_interrupt_set(&self) -> bool {
        (self.pro_cpu_interrupt_status_read() & (1 << (GPIONUM % 32))) != 0
    }

    fn is_pcore_non_maskable_interrupt_set(&self) -> bool {
        (self.pro_cpu_nmi_status_read() & (1 << (GPIONUM % 32))) != 0
    }

    fn is_acore_interrupt_set(&self) -> bool {
        (self.app_cpu_interrupt_status_read() & (1 << (GPIONUM % 32))) != 0
    }

    fn is_acore_non_maskable_interrupt_set(&self) -> bool {
        (self.app_cpu_nmi_status_read() & (1 << (GPIONUM % 32))) != 0
    }

    fn enable_hold(&mut self, _on: bool) {
        todo!();
    }
}

impl<MODE, RA, PINTYPE, const GPIONUM: u8> embedded_hal::digital::v2::OutputPin
    for GpioPin<Output<MODE>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    type Error = Infallible;
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.reg_access.write_output_set(1 << (GPIONUM % 32));
        Ok(())
    }
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.reg_access.write_output_clear(1 << (GPIONUM % 32));
        Ok(())
    }
}

impl<MODE, RA, PINTYPE, const GPIONUM: u8> embedded_hal::digital::v2::StatefulOutputPin
    for GpioPin<Output<MODE>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.reg_access.read_output() & (1 << (GPIONUM % 32)) != 0)
    }
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.is_set_high()?)
    }
}

impl<MODE, RA, PINTYPE, const GPIONUM: u8> embedded_hal::digital::v2::ToggleableOutputPin
    for GpioPin<Output<MODE>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    type Error = Infallible;
    fn toggle(&mut self) -> Result<(), Self::Error> {
        use embedded_hal::digital::v2::{OutputPin as _, StatefulOutputPin as _};
        if self.is_set_high()? {
            Ok(self.set_low()?)
        } else {
            Ok(self.set_high()?)
        }
    }
}

#[cfg(feature = "eh1")]
impl<MODE, RA, PINTYPE, const GPIONUM: u8> embedded_hal_1::digital::ErrorType
    for GpioPin<Output<MODE>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    type Error = Infallible;
}

#[cfg(feature = "eh1")]
impl<MODE, RA, PINTYPE, const GPIONUM: u8> embedded_hal_1::digital::OutputPin
    for GpioPin<Output<MODE>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.reg_access.write_output_clear(1 << (GPIONUM % 32));
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.reg_access.write_output_set(1 << (GPIONUM % 32));
        Ok(())
    }
}

#[cfg(feature = "eh1")]
impl<MODE, RA, PINTYPE, const GPIONUM: u8> embedded_hal_1::digital::StatefulOutputPin
    for GpioPin<Output<MODE>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.reg_access.read_output() & (1 << (GPIONUM % 32)) != 0)
    }
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.is_set_high()?)
    }
}

#[cfg(feature = "eh1")]
impl<MODE, RA, PINTYPE, const GPIONUM: u8> embedded_hal_1::digital::ToggleableOutputPin
    for GpioPin<Output<MODE>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    fn toggle(&mut self) -> Result<(), Self::Error> {
        use embedded_hal_1::digital::{OutputPin as _, StatefulOutputPin as _};
        if self.is_set_high()? {
            Ok(self.set_low()?)
        } else {
            Ok(self.set_high()?)
        }
    }
}

impl<RA, PINTYPE, const GPIONUM: u8> From<GpioPin<Unknown, RA, PINTYPE, GPIONUM>>
    for GpioPin<Input<Floating>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    fn from(
        pin: GpioPin<Unknown, RA, PINTYPE, GPIONUM>,
    ) -> GpioPin<Input<Floating>, RA, PINTYPE, GPIONUM> {
        pin.into_floating_input()
    }
}

impl<RA, PINTYPE, const GPIONUM: u8> From<GpioPin<Unknown, RA, PINTYPE, GPIONUM>>
    for GpioPin<Input<PullUp>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    fn from(
        pin: GpioPin<Unknown, RA, PINTYPE, GPIONUM>,
    ) -> GpioPin<Input<PullUp>, RA, PINTYPE, GPIONUM> {
        pin.into_pull_up_input()
    }
}

impl<RA, PINTYPE, const GPIONUM: u8> From<GpioPin<Unknown, RA, PINTYPE, GPIONUM>>
    for GpioPin<Input<PullDown>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsInputPin,
{
    fn from(
        pin: GpioPin<Unknown, RA, PINTYPE, GPIONUM>,
    ) -> GpioPin<Input<PullDown>, RA, PINTYPE, GPIONUM> {
        pin.into_pull_down_input()
    }
}

impl<RA, PINTYPE, const GPIONUM: u8> From<GpioPin<Unknown, RA, PINTYPE, GPIONUM>>
    for GpioPin<Output<PushPull>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    fn from(
        pin: GpioPin<Unknown, RA, PINTYPE, GPIONUM>,
    ) -> GpioPin<Output<PushPull>, RA, PINTYPE, GPIONUM> {
        pin.into_push_pull_output()
    }
}

impl<RA, PINTYPE, const GPIONUM: u8> From<GpioPin<Unknown, RA, PINTYPE, GPIONUM>>
    for GpioPin<Output<OpenDrain>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    fn from(
        pin: GpioPin<Unknown, RA, PINTYPE, GPIONUM>,
    ) -> GpioPin<Output<OpenDrain>, RA, PINTYPE, GPIONUM> {
        pin.into_open_drain_output()
    }
}

impl<RA, PINTYPE, const GPIONUM: u8> From<GpioPin<Unknown, RA, PINTYPE, GPIONUM>>
    for GpioPin<Alternate<AF1>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    fn from(
        pin: GpioPin<Unknown, RA, PINTYPE, GPIONUM>,
    ) -> GpioPin<Alternate<AF1>, RA, PINTYPE, GPIONUM> {
        pin.into_alternate_1()
    }
}

impl<RA, PINTYPE, const GPIONUM: u8> From<GpioPin<Unknown, RA, PINTYPE, GPIONUM>>
    for GpioPin<Alternate<AF2>, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    fn from(
        pin: GpioPin<Unknown, RA, PINTYPE, GPIONUM>,
    ) -> GpioPin<Alternate<AF2>, RA, PINTYPE, GPIONUM> {
        pin.into_alternate_2()
    }
}

impl<MODE, RA, PINTYPE, const GPIONUM: u8> GpioPin<MODE, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    fn init_output(&self, alternate: AlternateFunction, open_drain: bool) {
        let gpio = unsafe { &*GPIO::PTR };

        self.reg_access.write_out_en_set(1 << (GPIONUM % 32));
        gpio.pin[GPIONUM as usize].modify(|_, w| w.pad_driver().bit(open_drain));

        gpio.func_out_sel_cfg[GPIONUM as usize]
            .modify(|_, w| unsafe { w.out_sel().bits(OutputSignal::GPIO as OutputSignalType) });

        get_io_mux_reg(GPIONUM).modify(|_, w| unsafe {
            w.mcu_sel()
                .bits(alternate as u8)
                .fun_ie()
                .bit(open_drain)
                .fun_wpd()
                .clear_bit()
                .fun_wpu()
                .clear_bit()
                .fun_drv()
                .bits(DriveStrength::I20mA as u8)
                .slp_sel()
                .clear_bit()
        });
    }

    pub fn into_push_pull_output(self) -> GpioPin<Output<PushPull>, RA, PINTYPE, GPIONUM> {
        self.init_output(GPIO_FUNCTION, false);
        GpioPin {
            _mode: PhantomData,
            _pintype: PhantomData,
            reg_access: self.reg_access,
            af_input_signals: self.af_input_signals,
            af_output_signals: self.af_output_signals,
        }
    }

    pub fn into_open_drain_output(self) -> GpioPin<Output<OpenDrain>, RA, PINTYPE, GPIONUM> {
        self.init_output(GPIO_FUNCTION, true);
        GpioPin {
            _mode: PhantomData,
            _pintype: PhantomData,
            reg_access: self.reg_access,
            af_input_signals: self.af_input_signals,
            af_output_signals: self.af_output_signals,
        }
    }

    pub fn into_alternate_1(self) -> GpioPin<Alternate<AF1>, RA, PINTYPE, GPIONUM> {
        self.init_output(AlternateFunction::Function1, false);
        GpioPin {
            _mode: PhantomData,
            _pintype: PhantomData,
            reg_access: self.reg_access,
            af_input_signals: self.af_input_signals,
            af_output_signals: self.af_output_signals,
        }
    }

    pub fn into_alternate_2(self) -> GpioPin<Alternate<AF2>, RA, PINTYPE, GPIONUM> {
        self.init_output(AlternateFunction::Function2, false);
        GpioPin {
            _mode: PhantomData,
            _pintype: PhantomData,
            reg_access: self.reg_access,
            af_input_signals: self.af_input_signals,
            af_output_signals: self.af_output_signals,
        }
    }
}

impl<MODE, RA, PINTYPE, const GPIONUM: u8> OutputPin for GpioPin<MODE, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    fn set_to_open_drain_output(&mut self) -> &mut Self {
        self.init_output(GPIO_FUNCTION, true);
        self
    }

    fn set_to_push_pull_output(&mut self) -> &mut Self {
        self.init_output(GPIO_FUNCTION, false);
        self
    }

    fn enable_output(&mut self, on: bool) -> &mut Self {
        if on {
            self.reg_access.write_out_en_set(1 << (GPIONUM % 32));
        } else {
            self.reg_access.write_out_en_clear(1 << (GPIONUM % 32));
        }
        self
    }

    fn set_output_high(&mut self, high: bool) -> &mut Self {
        if high {
            self.reg_access.write_output_set(1 << (GPIONUM % 32));
        } else {
            self.reg_access.write_output_clear(1 << (GPIONUM % 32));
        }
        self
    }

    fn set_drive_strength(&mut self, strength: DriveStrength) -> &mut Self {
        get_io_mux_reg(GPIONUM).modify(|_, w| unsafe { w.fun_drv().bits(strength as u8) });

        self
    }

    fn enable_open_drain(&mut self, on: bool) -> &mut Self {
        unsafe { &*GPIO::PTR }.pin[GPIONUM as usize].modify(|_, w| w.pad_driver().bit(on));
        self
    }

    fn internal_pull_up_in_sleep_mode(&mut self, on: bool) -> &mut Self {
        get_io_mux_reg(GPIONUM).modify(|_, w| w.mcu_wpu().bit(on));
        self
    }
    fn internal_pull_down_in_sleep_mode(&mut self, on: bool) -> &mut Self {
        get_io_mux_reg(GPIONUM).modify(|_, w| w.mcu_wpd().bit(on));
        self
    }
    fn enable_output_in_sleep_mode(&mut self, on: bool) -> &mut Self {
        get_io_mux_reg(GPIONUM).modify(|_, w| w.mcu_oe().bit(on));
        self
    }

    fn connect_peripheral_to_output_with_options(
        &mut self,
        signal: OutputSignal,
        invert: bool,
        invert_enable: bool,
        enable_from_gpio: bool,
        force_via_gpio_mux: bool,
    ) -> &mut Self {
        let af = if force_via_gpio_mux {
            GPIO_FUNCTION
        } else {
            let mut res = GPIO_FUNCTION;
            for (i, output_signal) in self.af_output_signals.iter().enumerate() {
                if let Some(output_signal) = output_signal {
                    if *output_signal == signal {
                        res = match i {
                            0 => AlternateFunction::Function0,
                            1 => AlternateFunction::Function1,
                            2 => AlternateFunction::Function2,
                            3 => AlternateFunction::Function3,
                            4 => AlternateFunction::Function4,
                            5 => AlternateFunction::Function5,
                            _ => unreachable!(),
                        };
                        break;
                    }
                }
            }
            res
        };
        if af == GPIO_FUNCTION && signal as usize > OUTPUT_SIGNAL_MAX as usize {
            panic!("Cannot connect this peripheral to GPIO");
        }
        self.set_alternate_function(af);
        let clipped_signal = if signal as usize <= OUTPUT_SIGNAL_MAX as usize {
            signal as OutputSignalType
        } else {
            OUTPUT_SIGNAL_MAX
        };
        unsafe { &*GPIO::PTR }.func_out_sel_cfg[GPIONUM as usize].modify(|_, w| unsafe {
            w.out_sel()
                .bits(clipped_signal)
                .inv_sel()
                .bit(invert)
                .oen_sel()
                .bit(enable_from_gpio)
                .oen_inv_sel()
                .bit(invert_enable)
        });
        self
    }

    fn disconnect_peripheral_from_output(&mut self) -> &mut Self {
        self.set_alternate_function(GPIO_FUNCTION);
        unsafe { &*GPIO::PTR }.func_out_sel_cfg[GPIONUM as usize]
            .modify(|_, w| unsafe { w.out_sel().bits(OutputSignal::GPIO as OutputSignalType) });
        self
    }

    fn internal_pull_up(&mut self, on: bool) -> &mut Self {
        get_io_mux_reg(GPIONUM).modify(|_, w| w.fun_wpu().bit(on));
        self
    }
    fn internal_pull_down(&mut self, on: bool) -> &mut Self {
        get_io_mux_reg(GPIONUM).modify(|_, w| w.fun_wpd().bit(on));
        self
    }
}

impl<MODE, RA, PINTYPE, const GPIONUM: u8> GpioPin<MODE, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: IsOutputPin,
{
    pub fn into_analog(self) -> GpioPin<Analog, RA, PINTYPE, GPIONUM> {
        types::internal_into_analog(GPIONUM);

        GpioPin {
            _mode: PhantomData,
            _pintype: PhantomData,
            reg_access: self.reg_access,
            af_input_signals: self.af_input_signals,
            af_output_signals: self.af_output_signals,
        }
    }
}

pub struct IO {
    _io_mux: IO_MUX,
    pub pins: types::Pins,
}
impl IO {
    pub fn new(gpio: GPIO, io_mux: IO_MUX) -> Self {
        let pins = gpio.split();
        let io = IO {
            _io_mux: io_mux,
            pins,
        };
        io
    }
}

// while ESP32-S3 is multicore it is more like single core in terms of GPIO
// interrupts
#[cfg(esp32s3)]
impl<MODE, RA, PINTYPE, const GPIONUM: u8>
    InterruptStatusRegisters<SingleCoreInteruptStatusRegisterAccess>
    for GpioPin<MODE, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: PinType,
{
}

#[cfg(esp32)]
impl<MODE, RA, PINTYPE, const GPIONUM: u8>
    InterruptStatusRegisters<DualCoreInteruptStatusRegisterAccess>
    for GpioPin<MODE, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: PinType,
{
}

#[cfg(esp32c3)]
impl<MODE, RA, PINTYPE, const GPIONUM: u8>
    InterruptStatusRegisters<SingleCoreInteruptStatusRegisterAccess>
    for GpioPin<MODE, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: PinType,
{
}

#[cfg(esp32s2)]
impl<MODE, RA, PINTYPE, const GPIONUM: u8>
    InterruptStatusRegisters<SingleCoreInteruptStatusRegisterAccess>
    for GpioPin<MODE, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: PinType,
{
}

#[cfg(esp32c2)]
impl<MODE, RA, PINTYPE, const GPIONUM: u8>
    InterruptStatusRegisters<SingleCoreInteruptStatusRegisterAccess>
    for GpioPin<MODE, RA, PINTYPE, GPIONUM>
where
    RA: BankGpioRegisterAccess,
    PINTYPE: PinType,
{
}

#[doc(hidden)]
#[macro_export]
macro_rules! gpio {
    (
        $(
            ($gpionum:literal, $bank:literal, $type:ident
                $(
                    ( $( $af_input_num:literal => $af_input_signal:ident )* )
                    ( $( $af_output_num:literal => $af_output_signal:ident )* )
                )?
            )
        )+
    ) => {
        #[doc(hidden)]
        pub trait GpioExt {
            type Parts;
            fn split(self) -> Self::Parts;
        }

        paste!{
            impl GpioExt for GPIO {
                type Parts = Pins;
                fn split(self) -> Self::Parts {
                    Pins {
                        $(
                            [< gpio $gpionum >]: {
                                #[allow(unused_mut)]
                                let mut input_signals = [None,None,None,None,None,None];

                                #[allow(unused_mut)]
                                let mut output_signals = [None,None,None,None,None,None];

                                $(
                                    $(
                                        input_signals[ $af_input_num ] = Some( InputSignal::$af_input_signal );
                                    )*

                                    $(
                                        output_signals[ $af_output_num ] = Some( OutputSignal::$af_output_signal );
                                    )*
                                )?

                                 GpioPin {
                                    _mode: PhantomData,
                                    _pintype: PhantomData,
                                    reg_access: [< Bank $bank GpioRegisterAccess >] {},
                                    af_input_signals: input_signals,
                                    af_output_signals: output_signals,
                                }
                            },
                        )+
                    }
                }
            }

            pub struct Pins {
                $(
                    pub [< gpio $gpionum >] : GpioPin<Unknown, [< Bank $bank GpioRegisterAccess >], [< $type PinType >], $gpionum>,
                )+
            }

            $(
                pub type [<Gpio $gpionum >]<MODE> = GpioPin<MODE, [< Bank $bank GpioRegisterAccess >], [< $type PinType >], $gpionum>;
            )+
        }
    };
}

// Following code enables `into_analog`

#[doc(hidden)]
pub fn enable_iomux_clk_gate() {
    #[cfg(esp32s2)]
    {
        use crate::pac::SENS;
        let sensors = unsafe { &*SENS::ptr() };
        sensors
            .sar_io_mux_conf
            .modify(|_, w| w.iomux_clk_gate_en().set_bit());
    }
}

#[cfg(not(any(esp32c2, esp32c3, esp32s2)))]
#[doc(hidden)]
#[macro_export]
macro_rules! analog {
    (
        $(
            (
                $pin_num:expr, $rtc_pin:expr, $pin_reg:expr,
                $mux_sel:ident, $fun_sel:ident, $fun_ie:ident $(, $rue:ident, $rde:ident)?
            )
        )+
    ) => {
        pub(crate) fn internal_into_analog(pin: u8) {
            use crate::pac::RTCIO;
            let rtcio = unsafe{ &*RTCIO::ptr() };
            $crate::gpio::enable_iomux_clk_gate();

            match pin {
                $(
                    $pin_num => {
                        // disable input
                        paste! {
                            rtcio.$pin_reg.modify(|_,w| w.$fun_ie().bit(false));

                            // disable output
                            rtcio.enable_w1tc.write(|w| unsafe { w.enable_w1tc().bits(1 << $rtc_pin) });

                            // disable open drain
                            rtcio.pin[$rtc_pin].modify(|_,w| w.pad_driver().bit(false));

                                rtcio.$pin_reg.modify(|_,w| {
                                    w.$fun_ie().clear_bit();

                                    // Connect pin to analog / RTC module instead of standard GPIO
                                    w.$mux_sel().set_bit();

                                    // Select function "RTC function 1" (GPIO) for analog use
                                    unsafe { w.$fun_sel().bits(0b00) }
                                });

                            // Disable pull-up and pull-down resistors on the pin, if it has them
                            $(
                                rtcio.$pin_reg.modify(|_,w| {
                                    w
                                    .$rue().bit(false)
                                    .$rde().bit(false)
                                });
                            )?
                        }
                    }
                )+
                    _ => unreachable!(),
            }
        }
    }
}

#[cfg(esp32s2)]
#[doc(hidden)]
#[macro_export]
macro_rules! analog {
    (
        $(
            (
                $pin_num:expr, $rtc_pin:expr, $pin_reg:expr,
                $mux_sel:ident, $fun_sel:ident, $fun_ie:ident $(, $rue:ident, $rde:ident)?
            )
        )+
    ) => {
        pub(crate) fn internal_into_analog(pin: u8) {
            use crate::pac::RTCIO;
            let rtcio = unsafe{ &*RTCIO::ptr() };
            $crate::gpio::enable_iomux_clk_gate();

            match pin {
                $(
                    $pin_num => {

                        paste!{
                            use $crate::gpio::types::[< esp32s2_get_rtc_pad_ $pin_reg>];
                            let rtc_pad = [< esp32s2_get_rtc_pad_ $pin_reg>]();
                        }

                        // disable input
                        rtc_pad.modify(|_,w| w.$fun_ie().bit(false));

                        // disable output
                        rtcio.enable_w1tc.write(|w| unsafe { w.enable_w1tc().bits(1 << $rtc_pin) });

                        // disable open drain
                        rtcio.pin[$rtc_pin].modify(|_,w| w.pad_driver().bit(false));

                        rtc_pad.modify(|_,w| {
                            w.$fun_ie().clear_bit();

                            // Connect pin to analog / RTC module instead of standard GPIO
                            w.$mux_sel().set_bit();

                            // Select function "RTC function 1" (GPIO) for analog use
                            unsafe { w.$fun_sel().bits(0b00) }
                        });

                        // Disable pull-up and pull-down resistors on the pin, if it has them
                        $(
                            rtc_pad.modify(|_,w| {
                                w
                                .$rue().bit(false)
                                .$rde().bit(false)
                            });
                        )?
                    }
                )+
                    _ => unreachable!(),
            }
        }
    }
}

#[cfg(any(esp32c2, esp32c3))]
#[doc(hidden)]
#[macro_export]
macro_rules! analog {
    (
        $($pin_num:literal)+
    ) => {
        pub(crate) fn internal_into_analog(pin: u8) {
            use crate::pac::IO_MUX;
            use crate::pac::GPIO;

            let io_mux = unsafe{ &*IO_MUX::PTR };
            let gpio = unsafe{ &*GPIO::PTR };

            match pin {
                $(
                    $pin_num => {
                        io_mux.gpio[$pin_num].modify(|_,w| unsafe {
                            w.mcu_sel().bits(1)
                                .fun_ie().clear_bit()
                                .fun_wpu().clear_bit()
                                .fun_wpd().clear_bit()
                        });

                        gpio.enable_w1tc.write(|w| unsafe { w.bits(1 << $pin_num) });
                    }
                )+
                _ => unreachable!()
            }

        }
    }
}

pub(crate) use analog;
pub(crate) use gpio;

pub use self::types::{InputSignal, OutputSignal};
use self::types::{ONE_INPUT, ZERO_INPUT};
