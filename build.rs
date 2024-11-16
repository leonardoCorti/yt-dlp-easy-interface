fn main() {
    slint_build::compile("ui/main.slint")
        .expect("Slint build failed");
    embed_resource::compile("icon.rc", embed_resource::NONE)
        .manifest_optional().unwrap();
}
