use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::c_bindings::{
    format_mtime, gid_name, is_mount_point, store_modification_timestamp, uid_name,
};

use crate::colorama::{
    chngg, chngg_wrapper, mode_to_str, process_size, safe_terminal_string,
};

use crate::data_segment::{
    ERRR, FILE_SYSTEM_MAX_PATH, LLL_CLI_HELP_MESSAGE, NR_OF_SUPPORTET_LANG, TYPES,
};

pub fn path_buf_capped(base: &str) -> Option<PathBuf> {
    if base.len() > FILE_SYSTEM_MAX_PATH {
        return None;
    }
    let mut pb = PathBuf::with_capacity(base.len().max(64));
    if !base.is_empty() {
        pb.push(base);
    }
    Some(pb)
}

/**
 * Hängt `component` an `pb` an, sofern die Gesamtlänge ≤ `FILE_SYSTEM_MAX_PATH`.
 * Gibt `false` zurück, wenn der Puffer sonst überlaufen würde.
 */
pub fn path_push_capped(pb: &mut PathBuf, component: &str) -> bool {
    let extra = 1 + component.len();
    let cur = pb.as_os_str().len();
    if cur.saturating_add(extra) > FILE_SYSTEM_MAX_PATH {
        return false;
    }
    pb.push(component);
    if pb.as_os_str().len() > FILE_SYSTEM_MAX_PATH {
        pb.pop();
        return false;
    }
    true
}

fn get_type_ind_by_name(type_s: &str) -> i32 {
    unsafe {
        for i in 1..NR_OF_SUPPORTET_LANG {
            if !TYPES[i].name.is_empty() && TYPES[i].name == type_s {
                return i as i32;
            }
        }
    }
    ERRR
}

/**
 * zwei mutable Aliase eventuell UB, auch ohne zweiten Thread.
 * Grenzen zuvor gecheckt.
 */
fn add_type_value(pp_index: i32, filesize: u64) -> i32 {
    if pp_index < 0 || pp_index as usize >= NR_OF_SUPPORTET_LANG {
        return -1;
    }
    unsafe {
        TYPES[pp_index as usize].countet_size = TYPES[pp_index as usize].countet_size.saturating_add(filesize);
    }
    0
}

/**
 * Extrahiert die Erweiterung aus fullpath und akkumuliert die Größe.
 */
pub fn get_type(fullpath: &str, filesize: u64) {
    let bytes = fullpath.as_bytes();
    let size = bytes.len() as i32;
    if size <= 0 {
        return;
    }
    let mut point_index: i32 = -1;
    let mut i = size - 1;
    loop {
        if bytes[i as usize] == b'.' {
            point_index = i;
            break;
        }
        if bytes[i as usize] == b'/' {
            point_index = -1;
            break;
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    if point_index <= 0 {
        return;
    }
    let start = (point_index + 1) as usize;
    if start >= bytes.len() {
        return;
    }
    let type_s = match std::str::from_utf8(&bytes[start..]) {
        Ok(s) => s,
        Err(_) => return,
    };
    let index = get_type_ind_by_name(type_s);
    if index != ERRR {
        add_type_value(index, filesize);
    }
}

/**
 * Berechnet den dominanten Sprach-Index (größte akkumulierte Größe)
 * und setzt alle Zähler zurück.
 */
pub fn get_dom_lang_index() -> i32 {
    let mut local_sums = [0u64; NR_OF_SUPPORTET_LANG];
    unsafe {
        for i in 1..NR_OF_SUPPORTET_LANG {
            let g = TYPES[i].globalsumindex;
            if g >= 0 && (g as usize) < NR_OF_SUPPORTET_LANG {
                local_sums[g as usize] = local_sums[g as usize].saturating_add(TYPES[i].countet_size);
            }
        }
    }
    let mut max_idx = 0i32;
    let mut max_val = 0u64;
    for i in 0..NR_OF_SUPPORTET_LANG {
        if local_sums[i] > max_val {
            max_val = local_sums[i];
            max_idx = i as i32;
        }
    }
    unsafe {
        for i in 0..NR_OF_SUPPORTET_LANG {
            TYPES[i].countet_size = 0;
        }
    }
    max_idx
}

/**
 * Typ-Index einer einzelnen Datei anhand ihrer Erweiterung.
 */
pub fn get_dom_single_file_index_from_path(path: &str) -> i32 {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    unsafe {
        for i in 0..NR_OF_SUPPORTET_LANG {
            if !TYPES[i].name.is_empty() && TYPES[i].name == ext {
                return i as i32;
            }
        }
    }
    0
}

pub fn folder_size(path: &Path, ts_max: &mut i64) -> u64 {
    if is_mount_point(path) {
        return 0;
    }
    let mut total: u64 = 0;
    let rd = match fs::read_dir(path) {
        Ok(d) => d,
        Err(_) => return 0,
    };
    for entry in rd.flatten() {
        let name = entry.file_name();
        if name == "." || name == ".." {
            continue;
        }
        let full = entry.path();
        let st = match fs::symlink_metadata(&full) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if st.file_type().is_dir() {
            total = total.saturating_add(folder_size(&full, ts_max));
        } else if st.file_type().is_symlink() {
            continue;
        } else {
            store_modification_timestamp(&st, ts_max);
            let sz = st.len();
            total = total.saturating_add(sz);
            if let Some(s) = full.to_str() {
                get_type(s, sz);
            }
        }
    }
    total
}

pub fn clipper(content: &str) {
    let mut child = match Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return,
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(content.as_bytes());
    }
    let _ = child.wait();
}

pub fn sort_by_timestamp(unix_timestamps: &mut [i64], rows: &mut [String]) {
    let n = unix_timestamps.len();
    if n <= 1 {
        return;
    }
    for i in 0..n - 1 {
        let mut max_index = i;
        for j in i + 1..n {
            if unix_timestamps[j] > unix_timestamps[max_index] {
                max_index = j;
            }
        }
        if max_index != i {
            unix_timestamps.swap(i, max_index);
            rows.swap(i, max_index);
        }
    }
    rows.reverse();
    unix_timestamps.reverse();
}

pub fn sort_by_size(sizes: &mut [u64], rows: &mut [String]) {
    let n = sizes.len();
    if n <= 1 {
        return;
    }
    for i in 0..n - 1 {
        let mut max_index = i;
        for j in i + 1..n {
            if sizes[j] < sizes[max_index] {
                max_index = j;
            }
        }
        if max_index != i {
            sizes.swap(i, max_index);
            rows.swap(i, max_index);
        }
    }
}

/**
 * Gibt absolute Pfade aller Einträge in path_to_process aus.
 */
pub fn print_absolut_paths(path_to_process: &str) {
    let rd = match fs::read_dir(path_to_process) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("opendir: {}", e);
            return;
        }
    };
    let is_dot = path_to_process.starts_with('.');
    let cwd = env::current_dir().ok();
    for entry in rd.flatten() {
        let name = entry.file_name();
        if name == "." || name == ".." {
            continue;
        }
        let name_s = name.to_string_lossy();
        if is_dot {
            if let Some(ref p) = cwd {
                println!("{}/{}", p.display(), name_s);
            }
        } else {
            println!("{}/{}", path_to_process.trim_end_matches('/'), name_s);
        }
    }
}

/**
 * Baut den absoluten Pfad zu name
 * und kopiert ihn in die Zwischenablage.
 */
pub fn get_absolut_path(name: &str) {
    let cwd = match env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ERROR: getcwd: {}", e);
            return;
        }
    };
    let mut absolut = match path_buf_capped(&cwd.to_string_lossy()) {
        Some(pb) => pb,
        None => {
            eprintln!("ERROR: cwd exceeds FILE_SYSTEM_MAX_PATH");
            return;
        }
    };
    if !path_push_capped(&mut absolut, name) {
        eprintln!("ERROR: path exceeds FILE_SYSTEM_MAX_PATH ({})", FILE_SYSTEM_MAX_PATH);
        return;
    }
    let s = absolut.to_string_lossy().into_owned();
    println!("{}", s);
    clipper(&s);
}

/**
 * Detaillierte Verzeichnis-Ausgabe (ähnlich `ls -l` + Sprach-Farben).
 * sort_by_size_flag = true -> nach Größe, sonst nach Änderungszeit.
 * Init drei wichtigsten Pointer.
 */
pub fn main_lll_impl(sort_by_size_flag: bool, path_to_process: &str) -> i32 {
    let rd = match fs::read_dir(path_to_process) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("opendir: {}", e);
            return 1;
        }
    };
    let mut rows: Vec<String> = Vec::new();
    let mut sizes: Vec<u64> = Vec::new();
    let mut unix_timestamps: Vec<i64> = Vec::new();
    for entry in rd.flatten() {
        let name_os = entry.file_name();
        if name_os == "." || name_os == ".." {
            continue;
        }
        let name = safe_terminal_string(&name_os.to_string_lossy());
        let mut full = match path_buf_capped(path_to_process) {
            Some(pb) => pb,
            None => {
                eprintln!("ERROR: path exceeds FILE_SYSTEM_MAX_PATH");
                continue;
            }
        };
        if !path_push_capped(&mut full, &name) {
            eprintln!("ERROR: path exceeds FILE_SYSTEM_MAX_PATH for {}", name);
            continue;
        }
        let st = match fs::symlink_metadata(&full) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let mode = st.mode();
        let perms = mode_to_str(mode);
        let links = st.nlink();
        let owner = uid_name(st.uid());
        let group = gid_name(st.gid());
        let ft = st.file_type();
        let raw_size: u64;
        let sizebuf: String;
        let mut timestamp: i64 = 0;
        if ft.is_dir() {
            raw_size = folder_size(&full, &mut timestamp);
            sizebuf = process_size(raw_size);
            if let Some(s) = full.to_str() {
                get_type(s, raw_size);
            }
        } else {
            store_modification_timestamp(&st, &mut timestamp);
            raw_size = st.len();
            sizebuf = process_size(raw_size);
            if let Some(s) = full.to_str() {
                get_type(s, raw_size);
            }
        }
        let timestr = format_mtime(timestamp);
        let ja_index = get_dom_lang_index();
        let line = if ft.is_symlink() {
            match fs::read_link(&full) {
                Ok(target) => {
                    let t = target.to_string_lossy();
                    format!(
                        "{} {:3} {} {} {:>8} {} {} -> {}",
                        perms, links, owner, group, sizebuf, timestr, name, t
                    )
                }
                Err(_) => format!(
                    "{} {:3} {} {} {:>8} {} {}",
                    perms, links, owner, group, sizebuf, timestr, name
                ),
            }
        } else {
            let owner_col = chngg(&owner, 0x44FF44, 0);
            let group_col = chngg(&group, 0x88FF88, 0);
            let size_col = chngg(&sizebuf, 0xFF0000, 3);
            let time_col = chngg(&timestr, 0x888866, 2);
            let type_fld = chngg_wrapper(ja_index);
            let name_col = chngg(
                &name,
                if ft.is_dir() { 0x9999FF } else { 0xFF33FF },
                1,
            );
            format!(
                "{} {:3} {} {} {:>8} {} {} {}",
                perms, links, owner_col, group_col, size_col, time_col, type_fld, name_col
            )
        };
        rows.push(line);
        sizes.push(raw_size);
        unix_timestamps.push(timestamp);
    }
    if !rows.is_empty() {
        if sort_by_size_flag {
            sort_by_size(&mut sizes, &mut rows);
        } else {
            sort_by_timestamp(&mut unix_timestamps, &mut rows);
        }
    }
    for row in &rows {
        println!("{}", row);
    }
    0
}

/**
 * Mehrspaltige Kurz-Ausgabe wie: ls
 */
pub fn main_ls_impl() -> i32 {
    let rd = match fs::read_dir(".") {
        Ok(d) => d,
        Err(e) => {
            eprintln!("opendir: {}", e);
            return 1;
        }
    };
    let mut entries: Vec<String> = Vec::new();
    for entry in rd.flatten() {
        let name_os = entry.file_name();
        if name_os == "." || name_os == ".." {
            continue;
        }
        let name = safe_terminal_string(&name_os.to_string_lossy());
        let full = PathBuf::from(format!("./{}", name));
        let st = match fs::symlink_metadata(&full) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let ft = st.file_type();
        let mut max_date_ts: i64 = 0;
        if ft.is_dir() {
            let _ = folder_size(&full, &mut max_date_ts);
        }
        let links = st.nlink();
        let lang_idx = if ft.is_dir() {
            get_dom_lang_index()
        } else {
            get_dom_single_file_index_from_path(full.to_str().unwrap_or(""))
        };
        let name_color: u64 = if ft.is_dir() { 0x9999FF } else { 0xFF33FF };
        let type_fld = chngg_wrapper(lang_idx);
        let project_name = chngg(&name, name_color, 1);
        let _ = {
            let b = project_name.as_bytes();
            let take = b.len().min(45);
            String::from_utf8_lossy(&b[..take]).into_owned()
        };
        let line = format!("{:5} {:>8} {:>46}", links, type_fld, project_name);
        entries.push(line);
    }
    let mut i = 0usize;
    while i < entries.len() {
        let end = (i + 4).min(entries.len());
        let mut line = String::new();
        for j in i..end {
            if j > i {
                line.push(' ');
            }
            line.push_str(&entries[j]);
        }
        println!("{}", line);
        i = end;
    }
    0
}

pub fn print_help() -> i32 {
    print!("{}", LLL_CLI_HELP_MESSAGE);
    println!("FILE_SYSTEM_MAX_PATH : {}", FILE_SYSTEM_MAX_PATH);
    0
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_type_ind_known() {
        assert_eq!(get_type_ind_by_name("rs"), 10);
        assert_eq!(get_type_ind_by_name("py"), 4);
        assert_eq!(get_type_ind_by_name("c"), 1);
        assert_eq!(get_type_ind_by_name("h"), 12);
    }

    #[test]
    fn get_type_ind_unknown() {
        assert_eq!(get_type_ind_by_name("xyz"), ERRR);
        assert_eq!(get_type_ind_by_name(""), ERRR);
    }

    #[test]
    fn process_size_via_colorama() {
        assert!(crate::colorama::process_size(0x400).contains("KB"));
    }

    #[test]
    fn sort_by_size_ascending() {
        let mut sizes = vec![30u64, 10, 20];
        let mut rows = vec!["c".into(), "a".into(), "b".into()];
        sort_by_size(&mut sizes, &mut rows);
        assert_eq!(sizes, vec![10, 20, 30]);
        assert_eq!(rows, vec!["a", "b", "c"]);
    }

    /**
     * Nach dem internen max-Selection-sort + reverse-array stehen die kleinsten
     * (ältesten) Timestamps vorne.*
     */
    #[test]
    fn sort_by_timestamp_oldest_first_after_reverse() {
        let mut ts = vec![100i64, 300, 200];
        let mut rows = vec!["a".into(), "c".into(), "b".into()];
        sort_by_timestamp(&mut ts, &mut rows);
        assert_eq!(ts, vec![100, 200, 300]);
        assert_eq!(rows, vec!["a", "b", "c"]);
    }

    #[test]
    fn get_dom_single_file() {
        assert_eq!(get_dom_single_file_index_from_path("foo.rs"), 10);
        assert_eq!(get_dom_single_file_index_from_path("bar.py"), 4);
        assert_eq!(get_dom_single_file_index_from_path("noext"), 0);
    }

    #[test]
    fn path_buf_capped_ok() {
        let pb = path_buf_capped("/tmp").unwrap();
        assert!(pb.starts_with("/tmp"));
    }

    #[test]
    fn path_buf_capped_too_long() {
        let long = "x".repeat(FILE_SYSTEM_MAX_PATH + 1);
        assert!(path_buf_capped(&long).is_none());
    }

    #[test]
    fn path_push_capped_respects_limit() {
        let mut pb = path_buf_capped("").unwrap();
        let chunk = "a".repeat(0x400);
        let mut pushes = 0;
        while path_push_capped(&mut pb, &chunk) {
            pushes += 1;
            if pushes > 70 {
                break;
            }
        }
        assert!(pb.as_os_str().len() <= FILE_SYSTEM_MAX_PATH);
        assert!(pushes > 0);
    }

    /**
     * Buffer already at limit -> further push must fail
     */
    #[test]
    fn path_push_capped_rejects_when_full() {
        let almost = "x".repeat(FILE_SYSTEM_MAX_PATH);
        let mut pb = path_buf_capped(&almost).unwrap();
        assert!(!path_push_capped(&mut pb, "y"));
        assert_eq!(pb.as_os_str().len(), FILE_SYSTEM_MAX_PATH);
    }

    #[test]
    fn path_buf_capped_exact_limit() {
        let exact = "z".repeat(FILE_SYSTEM_MAX_PATH);
        let pb = path_buf_capped(&exact);
        assert!(pb.is_some());
        assert_eq!(pb.unwrap().as_os_str().len(), FILE_SYSTEM_MAX_PATH);
    }

    #[test]
    fn add_type_value_invalid_index() {
        assert_eq!(add_type_value(-1, 100), -1);
        assert_eq!(add_type_value(NR_OF_SUPPORTET_LANG as i32, 100), -1);
        assert_eq!(add_type_value(999, 100), -1);
    }

    /**
     * should not panic; no extension -> silent return
     */
    #[test]
    fn get_type_no_extension() {
        get_type("/tmp/noext", 42);
        get_type("", 1);
        get_type(".", 1);
        get_type("/tmp/.", 1);
    }

    #[test]
    fn sort_empty_and_single() {
        let mut sizes: Vec<u64> = vec![];
        let mut rows: Vec<String> = vec![];
        sort_by_size(&mut sizes, &mut rows);
        assert!(rows.is_empty());
        let mut sizes = vec![5u64];
        let mut rows = vec!["only".into()];
        sort_by_size(&mut sizes, &mut rows);
        assert_eq!(rows, vec!["only"]);
        let mut ts: Vec<i64> = vec![];
        let mut rows: Vec<String> = vec![];
        sort_by_timestamp(&mut ts, &mut rows);
        assert!(rows.is_empty());
    }

    /**
     * 0 -> regular file with no perms
     */
    #[test]
    fn mode_via_colorama_edge() {
        assert_eq!(crate::colorama::mode_to_str(0), "----------");
        assert_eq!(crate::colorama::mode_to_str(0o140000), "s---------");// socket
        assert_eq!(crate::colorama::mode_to_str(0o010000), "p---------");// fifo
        assert_eq!(crate::colorama::mode_to_str(0o060000), "b---------");// block device
        assert_eq!(crate::colorama::mode_to_str(0o020000), "c---------");// char device
    }
}
