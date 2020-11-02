use std::fs;

fn main() {
    let files: Vec<_> = fs::read_dir("./c_src")
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| extension == "c")
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .collect();

    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
    }

    let sdl = pkg_config::probe_library("sdl").unwrap();
    cc::Build::new()
        .files(files)
        .includes(&sdl.include_paths)
        .compile("freedroidc");

    for lib in sdl.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }
    println!("cargo:rustc-link-lib=SDL_image");
    println!("cargo:rustc-link-lib=SDL_mixer");
    println!("cargo:rustc-link-lib=SDL_gfx");
}
