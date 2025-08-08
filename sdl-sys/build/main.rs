use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build/wrapper.h");

    let sdl = pkg_config::probe_library("sdl").expect("cannot find SDL library using pkg-config");
    for lib in &sdl.libs {
        println!("cargo:rustc-link-lib={lib}");
    }

    let sdl_mixer = pkg_config::probe_library("SDL_mixer")
        .expect("cannot find SDL Mixer library using pkg-config");
    for lib in &sdl_mixer.libs {
        println!("cargo:rustc-link-lib={lib}");
    }

    let sdl_image = pkg_config::probe_library("SDL_image")
        .expect("cannot find SDL Image library using pkg-config");
    for lib in &sdl_image.libs {
        println!("cargo:rustc-link-lib={lib}");
    }

    let sdl_gfx =
        pkg_config::probe_library("SDL_gfx").expect("cannot find SDL Gfx library using pkg-config");
    for lib in &sdl_gfx.libs {
        println!("cargo:rustc-link-lib={lib}");
    }

    let bindings_builder = bindgen::Builder::default()
        .header("build/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_args(
            sdl.include_paths
                .iter()
                .map(|path| format!("-I{}", path.display())),
        )
        .clang_args(sdl.defines.iter().map(|(name, value)| match value {
            Some(value) => format!("-D{name}={value}"),
            None => format!("-D{name}"),
        }))
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .derive_debug(true)
        .derive_default(true);

    let normal_bindings = bindings_builder
        .clone()
        .derive_eq(true)
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
        .blocklist_type("SDL_SysWMinfo.*")
        .blocklist_type("SDL_AudioSpec")
        .generate()
        .expect("Unable to generate normal bindings");

    let special_bindings = bindings_builder
        .blocklist_var("SDL_.*")
        .blocklist_function("SDL_.*")
        .blocklist_var("IMG_.*")
        .blocklist_type("IMG_.*")
        .blocklist_function("IMG_.*")
        .blocklist_var("SDLK_.*")
        .blocklist_type("SDLK_.*")
        .blocklist_function("SDLK_.*")
        .blocklist_function("zoom.*")
        .blocklist_function("rotozoom.*")
        .blocklist_function("shrinkSurface")
        .blocklist_function("rotateSurface90Degrees")
        .blocklist_var("Mix_.*")
        .blocklist_type("Mix_.*")
        .blocklist_function("Mix_.*")
        .blocklist_type("_XDisplay")
        .blocklist_type("Display")
        .blocklist_type("XID")
        .blocklist_type("Window")
        .blocklist_type("Uint(8|16|32)")
        .blocklist_type("SDL_version")
        .blocklist_item("SDL_SYSWM_.*")
        .allowlist_type("SDL_SysWMinfo")
        .allowlist_type("SDL_AudioSpec")
        .generate()
        .expect("Unable to generate special bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut out_file =
        File::create(out_path.join("bindings.rs")).expect("unable to create bindings file");
    normal_bindings
        .write(Box::new(&mut out_file))
        .expect("Couldn't write normal bindings!");
    special_bindings
        .write(Box::new(&mut out_file))
        .expect("Couldn't write special bindings!");
}
