use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    landmarks: Vec<Landmark>,
}

impl Config {
    fn rustify(self) -> String {
        let size = self.landmarks.len();
        assert!(size > 0, "Must have at least one land mark!");

        format!(
            r#"
                pub const LANDMARKS: [crate::landmark::Landmark; {}] = [{}];
            "#,
            size,
            self.landmarks
                .into_iter()
                .map(|landmark| landmark.rustify())
                .collect::<Vec<_>>()
                .join(",\n")
        )
    }
}

#[derive(Deserialize)]
struct Landmark {
    name: String,
    lat: f64,
    lon: f64,
    elevation: f64,
}

impl Landmark {
    fn rustify(self) -> String {
        format!(
            r#"
                crate::landmark::Landmark {{
                    name: "{}",
                    lle: geoconv::Lle::new(
                        geoconv::Degrees::new({:.6}),
                        geoconv::Degrees::new({:.6}),
                        geoconv::Meters::new({:.6}),
                    )
                }}
            "#,
            self.name, self.lat, self.lon, self.elevation
        )
    }
}

fn main() {
    linker_be_nice();
    // make sure linkall.x is the last linker script (otherwise might cause problems with flip-link)
    println!("cargo:rustc-link-arg=-Tlinkall.x");

    let config: Config =
        toml::from_slice(&std::fs::read(std::env::var("COMPASS_CONFIG").unwrap()).unwrap())
            .unwrap();

    std::fs::write(
        Path::new(&std::env::var("OUT_DIR").unwrap()).join("generated_config.rs"),
        config.rustify(),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}

fn linker_be_nice() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let kind = &args[1];
        let what = &args[2];

        match kind.as_str() {
            "undefined-symbol" => match what.as_str() {
                "_defmt_timestamp" => {
                    eprintln!();
                    eprintln!("💡 `defmt` not found - make sure `defmt.x` is added as a linker script and you have included `use defmt_rtt as _;`");
                    eprintln!();
                }
                "_stack_start" => {
                    eprintln!();
                    eprintln!("💡 Is the linker script `linkall.x` missing?");
                    eprintln!();
                }
                "esp_wifi_preempt_enable"
                | "esp_wifi_preempt_yield_task"
                | "esp_wifi_preempt_task_create" => {
                    eprintln!();
                    eprintln!("💡 `esp-wifi` has no scheduler enabled. Make sure you have the `builtin-scheduler` feature enabled, or that you provide an external scheduler.");
                    eprintln!();
                }
                "embedded_test_linker_file_not_added_to_rustflags" => {
                    eprintln!();
                    eprintln!("💡 `embedded-test` not found - make sure `embedded-test.x` is added as a linker script for tests");
                    eprintln!();
                }
                _ => (),
            },
            // we don't have anything helpful for "missing-lib" yet
            _ => {
                std::process::exit(1);
            }
        }

        std::process::exit(0);
    }

    println!(
        "cargo:rustc-link-arg=--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}
