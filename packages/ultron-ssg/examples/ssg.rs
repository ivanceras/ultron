fn main() {
    let content = include_str!("../../ultron-app/test_data/hello.rs");
    let html = ultron_ssg::render_to_string(content, "rust", None);
    std::fs::create_dir_all("out").expect("must create dir");
    std::fs::write("out/hello.html", html).expect("must write to file");
}
