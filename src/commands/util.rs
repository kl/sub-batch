use crate::scanner::SubAndFile;
use anyhow::Result as AnyResult;
use core::result::Result::Ok;

pub fn ask_user_ok(renames: &[SubAndFile]) -> AnyResult<bool> {
    for rename in renames.iter() {
        println!(
            "{} -> {}",
            rename.sub_path.to_string_lossy(),
            rename.file_path.to_string_lossy()
        );
    }
    println!("Ok? (y/n)");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.to_lowercase().starts_with('y'))
}
