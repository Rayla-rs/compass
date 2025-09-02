#![no_std]
#![no_main]
#![feature(f16)]

use esp_hal::{
    i2c::master::{Error, I2c, I2cAddress},
    uart::{RxError, TxError, Uart},
    Async,
};
use ublox::{
    CfgPrtUartBuilder, FixedLinearBuffer, GnssFixType, InProtoMask, NavPvtFlags, NavPvtFlags2,
    OutProtoMask, PacketRef, Parser, UartMode,
};

#[allow(dead_code)]
pub struct NavPvtState {
    pub time_tag: f64,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub valid: u8,
    pub time_accuracy: u32,
    pub nanosecond: i32,
    pub utc_time_accuracy: u32,
    pub lat: f64,
    pub lon: f64,
    pub height: f64,
    pub msl: f64,
    pub vel_ned: (f64, f64, f64),
    pub speed_over_ground: f64,
    pub heading_motion: f64,
    pub heading_vehicle: f64,
    pub magnetic_declination: f64,

    pub pdop: f64,
    pub satellites_used: u8,

    pub position_fix_type: GnssFixType,
    pub fix_flags: NavPvtFlags,
    pub invalid_llh: bool,
    pub position_accuracy: (f64, f64),
    pub velocity_accuracy: f64,
    pub heading_accuracy: f64,
    pub magnetic_declination_accuracy: f64,
    pub flags2: NavPvtFlags2,
}

impl Default for NavPvtState {
    fn default() -> Self {
        Self {
            time_tag: f64::NAN,
            year: 0,
            month: 0,
            day: 0,
            hour: 0,
            min: 0,
            sec: 0,
            valid: 0,
            time_accuracy: 0,
            nanosecond: 0,
            lat: f64::NAN,
            lon: f64::NAN,
            height: f64::NAN,
            msl: f64::NAN,
            vel_ned: (f64::NAN, f64::NAN, f64::NAN),
            speed_over_ground: f64::NAN,
            heading_motion: f64::NAN,
            heading_vehicle: f64::NAN,
            magnetic_declination: f64::NAN,
            pdop: f64::NAN,
            satellites_used: 0,
            utc_time_accuracy: 0,
            invalid_llh: true,
            position_accuracy: (f64::NAN, f64::NAN),
            velocity_accuracy: f64::NAN,
            heading_accuracy: f64::NAN,
            magnetic_declination_accuracy: f64::NAN,
            position_fix_type: GnssFixType::NoFix,
            fix_flags: NavPvtFlags::empty(),
            flags2: NavPvtFlags2::empty(),
        }
    }
}

struct Hmc5883IState {
    x_guass: f32,
    z_guass: f32,
    y_guass: f32,
}

impl Default for Hmc5883IState {
    fn default() -> Self {
        Hmc5883IState {
            x_guass: f32::NAN,
            z_guass: f32::NAN,
            y_guass: f32::NAN,
        }
    }
}

struct Device {
    uart_port: Uart<'static, Async>,
    i2c_port: I2c<'static, Async>,
    nav_pvt_state: NavPvtState,
    hmc5882i_state: Hmc5883IState,
}

impl Device {
    async fn new(
        mut uart_port: Uart<'static, Async>,
        i2c_port: I2c<'static, Async>,
    ) -> Result<Self, TxError> {
        uart_port
            .write_async(
                CfgPrtUartBuilder {
                    portid: ublox::UartPortId::Uart1,
                    reserved0: 0,
                    tx_ready: 0,
                    mode: UartMode::new(
                        ublox::DataBits::Eight,
                        ublox::Parity::None,
                        ublox::StopBits::One,
                    ),
                    baud_rate: 115200,
                    in_proto_mask: InProtoMask::all(),
                    out_proto_mask: OutProtoMask::UBLOX,
                    flags: 0,
                    reserved5: 0,
                }
                .into_packet_bytes()
                .as_slice(),
            )
            .await?;

        Ok(Self {
            uart_port,
            i2c_port,
            nav_pvt_state: NavPvtState::default(),
            hmc5882i_state: Hmc5883IState::default(),
        })
    }

    async fn run(&mut self) -> Result<(), RxError> {
        // loop {
        const MAX_PAYLOAD_LEN: usize = 1240;

        let mut buf = [0u8; MAX_PAYLOAD_LEN];
        let mut parser = Parser::new(FixedLinearBuffer::new(&mut buf));

        let mut local_buf = [0; MAX_PAYLOAD_LEN];
        let nbytes = self.read_uart(&mut local_buf).await?;
        if nbytes == 0 {
            // break;
        };

        let mut it = parser.consume_ubx(&local_buf[..nbytes]);
        loop {
            match it.next() {
                Some(Ok(packet)) => {
                    self.handle_packet(packet);
                }
                Some(Err(e)) => {
                    // bad packet
                }
                None => {
                    // debug!("Parsed all data in buffer ...");
                    break;
                }
            }
        }
        Ok(())
    }

    /// Reads the serial port, converting timeouts into "no data received"
    async fn read_uart(&mut self, output: &mut [u8]) -> Result<usize, RxError> {
        self.uart_port.read_async(output).await
    }

    async fn write_all_uart(&mut self, data: &[u8]) -> Result<usize, TxError> {
        self.uart_port.write_async(data).await
    }

    async fn update_hmc5883i(&mut self) -> Result<(), Error> {
        let mut buf = [0u8; 6];
        let mut gain = [0u8; 1];

        self.i2c_port.read_async(1, &mut gain[0..=0]).await?;
        let gain = gain[0] >> 5;
        let resolution: f32 = match gain {
            0x00 => 0.73,
            0x01 => 0.92,
            0x02 => 1.22,
            0x03 => 1.52,
            0x04 => 2.27,
            0x05 => 2.56,
            0x06 => 3.03,
            0x07 => 4.35,
            _ => 0.0,
        };

        self.i2c_port.read_async(3, &mut buf[0..=0]).await?; // x
        self.i2c_port.read_async(4, &mut buf[1..=1]).await?;
        self.i2c_port.read_async(5, &mut buf[2..=2]).await?; // z
        self.i2c_port.read_async(6, &mut buf[3..=3]).await?;
        self.i2c_port.read_async(7, &mut buf[4..=4]).await?; // y
        self.i2c_port.read_async(8, &mut buf[5..=5]).await?;

        let raw = [
            (((buf[0] as i16) << 8) | buf[1] as i16),
            (((buf[2] as i16) << 8) | buf[6] as i16),
            (((buf[4] as i16) << 8) | buf[5] as i16),
        ];

        self.hmc5882i_state = Hmc5883IState {
            x_guass: (raw[0] as f32) * resolution,
            y_guass: (raw[1] as f32) * resolution,
            z_guass: (raw[2] as f32) * resolution,
        };

        Ok(())
    }

    fn handle_packet(&mut self, packet: PacketRef<'_>) {
        match packet {
            PacketRef::NavPvt(pkg) => {
                self.nav_pvt_state = NavPvtState {
                    time_tag: (pkg.itow() / 1000) as f64,
                    ..Default::default()
                };

                self.nav_pvt_state.flags2 = pkg.flags2();

                if pkg.flags2().contains(NavPvtFlags2::CONFIRMED_AVAI) {
                    self.nav_pvt_state.day = pkg.day();
                    self.nav_pvt_state.month = pkg.month();
                    self.nav_pvt_state.year = pkg.year();
                    self.nav_pvt_state.hour = pkg.hour();
                    self.nav_pvt_state.min = pkg.min();
                    self.nav_pvt_state.sec = pkg.sec();
                    self.nav_pvt_state.nanosecond = pkg.nanosec();

                    self.nav_pvt_state.utc_time_accuracy = pkg.time_accuracy();
                }

                self.nav_pvt_state.position_fix_type = pkg.fix_type();
                self.nav_pvt_state.fix_flags = pkg.flags();

                self.nav_pvt_state.lat = pkg.latitude();
                self.nav_pvt_state.lon = pkg.longitude();
                self.nav_pvt_state.height = pkg.height_above_ellipsoid();
                self.nav_pvt_state.msl = pkg.height_msl();

                self.nav_pvt_state.vel_ned = (pkg.vel_north(), pkg.vel_east(), pkg.vel_down());

                self.nav_pvt_state.speed_over_ground = pkg.ground_speed_2d();
                self.nav_pvt_state.heading_motion = pkg.heading_motion();
                self.nav_pvt_state.heading_vehicle = pkg.heading_vehicle();

                self.nav_pvt_state.magnetic_declination = pkg.magnetic_declination();

                self.nav_pvt_state.pdop = pkg.pdop();

                self.nav_pvt_state.satellites_used = pkg.num_satellites();

                self.nav_pvt_state.invalid_llh = pkg.flags3().invalid_llh();
                self.nav_pvt_state.position_accuracy =
                    (pkg.horizontal_accuracy(), pkg.vertical_accuracy());
                self.nav_pvt_state.velocity_accuracy = pkg.speed_accuracy();
                self.nav_pvt_state.heading_accuracy = pkg.heading_accuracy();
                self.nav_pvt_state.magnetic_declination_accuracy =
                    pkg.magnetic_declination_accuracy();
            }
            _ => {}
        }
    }
}
