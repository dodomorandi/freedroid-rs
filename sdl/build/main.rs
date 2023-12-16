use std::path::PathBuf;

#[derive(Debug)]
enum VideoDriver {
    X11,
    Nanox,
    WinDib,
    Ddraw,
    Gapi,
    RiscOs,
    Photon,
}

#[derive(Debug)]
struct Features {
    video_driver: Option<VideoDriver>,
}

fn get_features(include_dirs: &[PathBuf]) -> Features {
    println!("cargo:rerun-if-changed=build/detector.c");
    let mut gcc = cc::Build::new();
    for include_dir in include_dirs {
        gcc.include(include_dir);
    }
    let expanded = match gcc.file("build/detector.c").try_expand() {
        Ok(expanded) => expanded,
        Err(e) => {
            panic!("Cannot compile the SDL features detector: {e}");
        }
    };
    let expanded = String::from_utf8(expanded).unwrap();

    let mut video_driver = None;
    for line in expanded.lines() {
        if let Some(raw_driver) = line.strip_prefix("RUST_SDL_VIDEO_DRIVER_") {
            use VideoDriver::{Ddraw, Gapi, Nanox, Photon, RiscOs, WinDib, X11};
            let driver = match raw_driver {
                "X11" => X11,
                "NANOX" => Nanox,
                "WINDIB" => WinDib,
                "DDRAW" => Ddraw,
                "GAPI" => Gapi,
                "RISCOS" => RiscOs,
                "PHOTON" => Photon,
                _ => panic!("invalid video driver {raw_driver}"),
            };

            assert!(
                video_driver.replace(driver).is_none(),
                "SDL video driver already set"
            );
        }
    }

    Features { video_driver }
}

fn main() {
    let sdl = pkg_config::probe_library("sdl").expect("cannot find SDL library using pkg-config");

    let features = get_features(&sdl.include_paths);
    if let Some(video_driver) = &features.video_driver {
        use VideoDriver::{Ddraw, Gapi, Nanox, Photon, RiscOs, WinDib, X11};
        let video_driver = match video_driver {
            X11 => "x11",
            Nanox => "nanox",
            WinDib => "windib",
            Ddraw => "ddraw",
            Gapi => "gapi",
            RiscOs => "riscos",
            Photon => "photon",
        };

        println!("cargo:rustc-cfg=sdl_video_driver_{video_driver}");
    }
}
