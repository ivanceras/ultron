use std::{fs, io, path::Path};
use syntect::{
    dumps,
    highlighting::ThemeSet,
    parsing::{SyntaxSet, SyntaxSetBuilder},
};

//NOTE: syntect is not always compatible with the latest
//version of the github.com/sublimehq/Packages repo.
// This is one is using the commit hash:`f36b8f8`
// which is compatible with current version of syntect.
// As of writing, https://github.com/getzola/zola is using the same
// commit hash
fn main() -> io::Result<()> {
    let (ss, ts) = load_sublime();
    dump_sublime(&ss, &ts)?;
    Ok(())
}

fn dump_sublime(syntaxset: &SyntaxSet, themeset: &ThemeSet) -> io::Result<()> {
    let syntaxset_pack_path = "./dump/syntaxes.packdump";
    let themeset_pack_path = "./dump/themes.themedump";
    create_parent_dir(syntaxset_pack_path)?;
    create_parent_dir(themeset_pack_path)?;
    dumps::dump_to_file(&syntaxset, syntaxset_pack_path).expect("must dump to file");
    dumps::dump_to_file(&themeset, themeset_pack_path).expect("must dump to file");

    Ok(())
}

fn create_parent_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let parent = path.as_ref().parent().expect("must have a parent dir");
    fs::create_dir_all(parent)?;
    Ok(())
}

fn load_syntaxset(package_dir: &str) -> SyntaxSet {
    let mut builder = SyntaxSetBuilder::new();
    builder.add_plain_text_syntax();
    match builder.add_from_folder(package_dir, true) {
        Ok(_) => (),
        Err(e) => println!("Loading error: {:?}", e),
    };

    builder.build()
}

fn load_themeset(theme_dir: &str) -> ThemeSet {
    ThemeSet::load_from_folder(theme_dir).expect("must load themeset")
}

fn load_sublime() -> (SyntaxSet, ThemeSet) {
    let package_dir = "./syntaxes";
    let theme_dir = "./themes";
    let syntaxset = load_syntaxset(package_dir);
    let themeset = load_themeset(theme_dir);
    (syntaxset, themeset)
}
