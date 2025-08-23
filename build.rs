#[cfg(windows)]
use winres::WindowsResource;

fn main() {
    cc::Build::new().file("src/api.c").compile("api");
    #[cfg(windows)]
    {
        WindowsResource::new()
            .set_icon("assets/icon_main.ico")
            .compile()
            .unwrap();
    }
    println!("cargo:rerun-if-changed=src/api.c");
}
