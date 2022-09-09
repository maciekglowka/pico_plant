// these values should be calibrated by measuring the ADC output
// firstly of the completely dry sensor
// secondly of the sensor submerged in water
const DRY_VALUE: f32 = 1670.0;
const WET_VALUE: f32 = 920.0;

pub struct SoilSensor {
    pub pin_idx: u8
}

impl SoilSensor {
    pub fn new(pin_idx: u8) -> SoilSensor {
        SoilSensor { pin_idx }
    }

    pub fn init(&self) {
        let adc = embassy_rp::pac::ADC;
        unsafe {
            // start ADC
            adc.cs().write(|w| w.set_en(true) );
            // wait for ADC to be ready
            while !adc.cs().read().ready() {
                cortex_m::asm::nop();
            };
        }
    }

    pub fn read_single(&self) -> f32 {
        let adc = embassy_rp::pac::ADC;
        unsafe {
            adc.cs().modify(|w| {
                // set ainsel to read required pin
                w.set_ainsel(self.pin_idx);
                // request single read
                adc.cs().modify(|w| w.set_start_once(true));
            });
            // wait for the ADC conversion
            while !adc.cs().read().ready() {
                cortex_m::asm::nop();
            };

            let r = adc.result().read().result();
            self.to_ratio(r)
        }
    }

    fn to_ratio(&self, val: u16) -> f32 {
        let r = 100.0 * (DRY_VALUE - val as f32) / (DRY_VALUE - WET_VALUE);
        // just in case we get beyond calibrated boundaries
        r.max(0.0).min(100.0)
    }
}