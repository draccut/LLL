mod data_segment;
mod c_bindings;
mod colorama;
mod kommulierte_metadaten;

use data_segment::flags;

use kommulierte_metadaten::{
    get_absolut_path, main_lll_impl, main_ls_impl, print_absolut_paths, print_help,
};

/**
 * Baut die Argument-Bitmap und extrahiert den optionalen Pfad.
 *
 * Rückgabe: `(bitmap, path)`
 * path ist `Some` wenn ein freier Name oder `-p <path>` erkannt wurde.
 * Freier Name fürt zu -> absoluter Pfad + Clipboard save
 */
fn parse_args(args: &[String]) -> (u32, Option<String>) {
    let mut bitmap: u32 = 0;
    let mut path: Option<String> = None;
    let mut i = 1usize;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "-s" | "--size" => {
                bitmap |= flags::SORT_BY_SIZE;
            }
            "-l" | "--ls" => {
                bitmap |= flags::LS_MODE;
            }
            "-h" | "--help" => {
                bitmap |= flags::HELP;
            }
            "-p" | "--path" => {
                bitmap |= flags::PATH;
                if i + 1 < args.len() {
                    path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "-q" | "--qqqq" => {
                bitmap |= flags::ABSOLUT_PATHS;
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {
                if !a.starts_with('-') {
                    bitmap |= flags::SINGLE_NAME;
                    path = Some(a.to_string());
                }
            }
        }
        i += 1;
    }
    (bitmap, path)
}

/**
 * Langer match auf den Bitmap-Wert.
 * Jede sinnvolle Kombination von Bits wird explizit aufgelöst.
 */
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (bitmap, path) = parse_args(&args);
    let exit_code: i32 = match bitmap {
        0 => main_lll_impl(false, "."),
        b if b == flags::SORT_BY_SIZE => main_lll_impl(true, "."),
        b if b == flags::LS_MODE => main_ls_impl(),
        b if b == flags::HELP => print_help(),
        b if b == flags::PATH => match path.as_deref() {
            Some(p) => main_lll_impl(false, p),
            None => {
                eprintln!("Unbekannter Parameter oder falsche Nutzung --path verlangt einen Wert.");
                1
            }
        },
        b if b == (flags::PATH | flags::SORT_BY_SIZE) => match path.as_deref() {
            Some(p) => main_lll_impl(true, p),
            None => {
                eprintln!("Unbekannter Parameter oder falsche Nutzung");
                1
            }
        },
        b if b == flags::ABSOLUT_PATHS => {
            print_absolut_paths(path.as_deref().unwrap_or("."));
            0
        }
        b if b == (flags::ABSOLUT_PATHS | flags::PATH) => {
            print_absolut_paths(path.as_deref().unwrap_or("."));
            0
        }
        b if b == flags::SINGLE_NAME => {
            if let Some(ref name) = path {
                get_absolut_path(name);
            }
            0
        }
        _ => {
            eprintln!("ERROR: falsche Nutzung");
            1
        }
    };
    std::process::exit(exit_code);
}
