use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build/wrapper.h");

    let sdl = pkg_config::probe_library("sdl").expect("cannot find SDL library using pkg-config");
    for lib in &sdl.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    let sdl_mixer = pkg_config::probe_library("SDL_mixer")
        .expect("cannot find SDL Mixer library using pkg-config");
    for lib in &sdl_mixer.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    let sdl_image = pkg_config::probe_library("SDL_image")
        .expect("cannot find SDL Image library using pkg-config");
    for lib in &sdl_image.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    let sdl_gfx =
        pkg_config::probe_library("SDL_gfx").expect("cannot find SDL Gfx library using pkg-config");
    for lib in &sdl_gfx.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    let bindings = bindgen::Builder::default()
        .header("build/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_args(
            sdl.include_paths
                .iter()
                .map(|path| format!("-I{}", path.display())),
        )
        .clang_args(sdl.defines.iter().map(|(name, value)| match value {
            Some(value) => format!("-D{}={}", name, value),
            None => format!("-D{}", name),
        }))
        .allowlist_var("SDL_.*")
        .allowlist_type("SDL_.*")
        .allowlist_function("SDL_.*")
        .allowlist_var("IMG_.*")
        .allowlist_type("IMG_.*")
        .allowlist_function("IMG_.*")
        .allowlist_var("SDLK_.*")
        .allowlist_type("SDLK_.*")
        .allowlist_function("SDLK_.*")
        .allowlist_function("zoom.*")
        .allowlist_function("rotozoom.*")
        .allowlist_function("shrinkSurface")
        .allowlist_function("rotateSurface90Degrees")
        .allowlist_var("Mix_.*")
        .allowlist_type("Mix_.*")
        .allowlist_function("Mix_.*")
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .derive_debug(true)
        .derive_default(true)
        .derive_eq(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
