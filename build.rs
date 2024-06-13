fn main() {
    cc::Build::new().file("src/api.c").compile("api");
    embed_resource::compile("assets/icon.rc", embed_resource::NONE);

    print!("cargo:rerun-if-changed=src/api.c");
}
