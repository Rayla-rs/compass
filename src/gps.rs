use core::cell::Cell;

use critical_section::Mutex;
use embassy_time::{Duration, Ticker};
use esp_hal::{
    peripherals::*,
    uart::{self, RxError, TxError, Uart},
    Async,
};
use esp_println::println;
use geoconv::{Degrees, Lle, Meters, Wgs84};
use ublox::{
    CfgPrtUartBuilder, FixedLinearBuffer, GnssFixType, InProtoMask, NavPvtFlags2, OutProtoMask,
    PacketRef, Parser, UartMode,
};

pub static NAV_PVT_STATE: Mutex<Cell<NavPvtState>> = Mutex::new(Cell::new(NavPvtState::new()));

#[embassy_executor::task]
pub async fn gps_task(uart: UART1<'static>, rx: GPIO17<'static>, tx: GPIO16<'static>) -> ! {
    println!("Started Gps Task");

    let mut gps = Gps::new(uart, rx, tx).await.unwrap();
    println!("Gps Ready!");

    // let mut ticker = Ticker::every(Duration::from_millis(100));

    loop {
        if let Err(err) = gps.process().await {
            println!("GPS RX Err:{}", err)
        }
        // ticker.next().await;
    }
}

#[derive(Clone, Copy)]
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
    /// Latitude, Longitude, Elevation
    pub lle: Option<Lle<Wgs84, Degrees>>,

    pub msl: f64,
    pub vel_ned: (f64, f64, f64),
    pub speed_over_ground: f64,
    pub heading_motion: f64,
    pub heading_vehicle: f64,
    pub magnetic_declination: f64,

    pub pdop: f64,
    pub satellites_used: u8,

    pub position_fix_type: GnssFixType,
}

impl NavPvtState {
    pub const fn new() -> Self {
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
            lle: None,
            msl: f64::NAN,
            vel_ned: (f64::NAN, f64::NAN, f64::NAN),
            speed_over_ground: f64::NAN,
            heading_motion: f64::NAN,
            heading_vehicle: f64::NAN,
            magnetic_declination: f64::NAN,
            pdop: f64::NAN,
            satellites_used: 0,
            utc_time_accuracy: 0,
            position_fix_type: GnssFixType::NoFix,
        }
    }
}

struct Gps {
    uart_port: Uart<'static, Async>,
}

impl Gps {
    pub async fn new(
        uart: UART1<'static>,
        rx: GPIO17<'static>,
        tx: GPIO16<'static>,
    ) -> Result<Self, TxError> {
        let config = uart::Config::default().with_baudrate(115200);
        let mut uart_port = Uart::new(uart, config)
            .unwrap()
            .with_rx(rx)
            .with_tx(tx)
            .into_async();

        uart_port
            .write_async(
                CfgPrtUartBuilder {
                    portid: ublox::UartPortId::Usb, // Double check this is not supposed to be Uart1
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

        Ok(Self { uart_port })
    }

    async fn process(&mut self) -> Result<(), RxError> {
        const MAX_PAYLOAD_LEN: usize = 1280;

        let mut buf = [0u8; MAX_PAYLOAD_LEN];
        let mut parser = Parser::new(FixedLinearBuffer::new(&mut buf));

        let mut local_buf = [0; MAX_PAYLOAD_LEN];
        let nbytes = self.read_uart(&mut local_buf).await?;

        let mut it = parser.consume_ubx(&local_buf[..nbytes]);

        loop {
            match it.next() {
                Some(Ok(packet)) => {
                    self.handle_packet(packet);
                }
                Some(Err(err)) => {
                    println!("Bad Packet! Err:{}", err);
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

    fn handle_packet(&mut self, packet: PacketRef<'_>) {
        println!("Handle Packet:{:?}", packet);

        let parsed = match packet {
            PacketRef::NavPvt(pkg) => {
                let mut nav_pvt_state = NavPvtState {
                    time_tag: (pkg.itow() / 1000) as f64,
                    ..NavPvtState::new()
                };

                if pkg.flags2().contains(NavPvtFlags2::CONFIRMED_AVAI) {
                    nav_pvt_state.day = pkg.day();
                    nav_pvt_state.month = pkg.month();
                    nav_pvt_state.year = pkg.year();
                    nav_pvt_state.hour = pkg.hour();
                    nav_pvt_state.min = pkg.min();
                    nav_pvt_state.sec = pkg.sec();
                    nav_pvt_state.nanosecond = pkg.nanosec();

                    nav_pvt_state.utc_time_accuracy = pkg.time_accuracy();
                }

                nav_pvt_state.position_fix_type = pkg.fix_type();

                nav_pvt_state.lle = Some(Lle::new(
                    Degrees::new(pkg.latitude()),
                    Degrees::new(pkg.longitude()),
                    Meters::new(pkg.height_above_ellipsoid()),
                ));
                //nav_pvt_state.lat = pkg.latitude();
                //nav_pvt_state.lon = pkg.longitude();
                //nav_pvt_state.height = pkg.height_above_ellipsoid();
                nav_pvt_state.msl = pkg.height_msl();

                nav_pvt_state.vel_ned = (pkg.vel_north(), pkg.vel_east(), pkg.vel_down());

                nav_pvt_state.speed_over_ground = pkg.ground_speed_2d();
                nav_pvt_state.heading_motion = pkg.heading_motion();
                nav_pvt_state.heading_vehicle = pkg.heading_vehicle();

                nav_pvt_state.magnetic_declination = pkg.magnetic_declination();

                nav_pvt_state.pdop = pkg.pdop();

                nav_pvt_state.satellites_used = pkg.num_satellites();

                Some(nav_pvt_state)
            }
            _ => None,
        };

        if let Some(parsed) = parsed {
            critical_section::with(|cs| {
                NAV_PVT_STATE.borrow(cs).set(parsed);
            })
        }
    }
}
