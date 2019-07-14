use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use lazy_static::lazy_static;

lazy_static! {
    static ref CRUSH_WHITESPACE: regex::bytes::Regex =
        regex::bytes::Regex::new(r"\s+").expect("static data");
}

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let mut stdout = io::BufWriter::new(stdout);

    let suffix = env::args().nth(1).expect("usage: .suffix");

    let real_path = PathBuf::from(".");

    process(&mut stdout, &suffix, &real_path)?;

    Ok(())
}

fn process<W: Write>(out: &mut W, suffix: &str, real_path: &Path) -> Result<(), io::Error> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in fs::read_dir(real_path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        if name_str.starts_with('.') {
            continue;
        }

        if meta.is_dir() {
            dirs.push(file_name);
        } else if meta.is_file() && name_str.ends_with(&suffix) {
            files.push(file_name);
        }
    }

    dirs.sort();
    files.sort();

    for file in files {
        let mut data = Vec::with_capacity(4_096);
        let mut file_path = real_path.to_path_buf();
        file_path.push(file);
        let printable_path = file_path.to_string_lossy();
        out.write_all(printable_path.as_bytes())?;
        out.write_all(b"\t")?;
        io::BufReader::new(fs::File::open(file_path)?).read_to_end(&mut data)?;
        let data = CRUSH_WHITESPACE.replace_all(&data, &b" "[..]);
        out.write_all(&data)?;
        out.write_all(b"\n")?;
    }

    for dir in dirs {
        let mut sub = real_path.to_path_buf();
        sub.push(dir);
        process(out, suffix, &sub)?;
    }

    Ok(())
}
