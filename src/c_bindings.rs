use std::mem;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::fs::Metadata;
use std::time::UNIX_EPOCH;

type LllLibcChar = i8;

/**
 *  C-Struktur <pwd.h>
 *  Layout muss exakt dem ABI der Plattform entsprechen
 */
#[repr(C)]
struct Passwd {
    pw_name: *mut LllLibcChar,
    pw_passwd: *mut LllLibcChar,
    pw_uid: u32,
    pw_gid: u32,
    pw_gecos: *mut LllLibcChar,
    pw_dir: *mut LllLibcChar,
    pw_shell: *mut LllLibcChar,
}

unsafe extern "C" {
    fn getpwuid(uid: u32) -> *mut Passwd;
}

pub fn uid_name(uid: u32) -> String {
    unsafe {
        let pw = getpwuid(uid);
        if pw.is_null() {
            return "unknown".to_string();
        }
        let name = (*pw).pw_name;
        if name.is_null() {
            return "unknown".to_string();
        }
        std::ffi::CStr::from_ptr(name)
            .to_string_lossy()
            .into_owned()
    }
}

#[repr(C)]
struct Group {
    gr_name: *mut LllLibcChar,
    gr_passwd: *mut LllLibcChar,
    gr_gid: u32,
    gr_mem: *mut *mut LllLibcChar,
}

unsafe extern "C" {
    fn getgrgid(gid: u32) -> *mut Group;// thread-lokaler Puffer, sofort kopieren.
}

/**
 * Wandelt eine numerische GID in den Gruppennamen um.
 *
 * Transformationskette identisch zu uid_name:
 * *mut Group` -> Null-Check -> `gr_name: *mut i8` -> `CStr` -> owned `String`
 */
pub fn gid_name(gid: u32) -> String {
    unsafe {
        let gr = getgrgid(gid);
        if gr.is_null() {
            return "unknown".to_string();
        }
        let name = (*gr).gr_name;
        if name.is_null() {
            return "unknown".to_string();
        }
        std::ffi::CStr::from_ptr(name)
            .to_string_lossy()
            .into_owned()
    }
}

/**
 * Felder müssen in der Reihenfolge und Größe dem System-ABI entsprechen.
 * <time.h>
 */
#[repr(C)]
struct LibcTm {
    tm_sec: i32,
    tm_min: i32,
    tm_hour: i32,
    tm_mday: i32,
    tm_mon: i32,
    tm_year: i32,
    tm_wday: i32,
    tm_yday: i32,
    tm_isdst: i32,
    tm_gmtoff: i64,
    tm_zone: *const i8,
}

unsafe extern "C" {
    fn localtime_r(timep: *const i64, result: *mut LibcTm) -> *mut LibcTm;

    fn strftime(
        s: *mut i8,
        max: usize,
        format: *const i8,
        tm: *const LibcTm,
    ) -> usize;
}

/**
 * Formatiert einen Unix-Zeitstempel (Sekunden seit Epoch) in die
 * lesbare Form `"%e. %b %H:%M"` (z. B. `" 6. Oct 08:56"`).
 *
 * # Transformationskette
 *
 * 1. Stack-allokiertes `LibcTm` wird mit `mem::zeroed()` initialisiert
 *   (alle Felder 0 / Null-Pointer).
 * 2. `localtime_r(&secs, &mut tm)` füllt die Struktur.
 *   - Rückgabe `NULL` -> `"?"`.
 * 3. Stack-Puffer `[0i8; 64]` nimmt das Formatierungsergebnis auf.
 * 4. `strftime` schreibt den formatierten String (C-Null-terminiert).
 * 5. `CStr::from_ptr` -> `to_string_lossy` -> `trim` -> owned `String`
 *
 * Alle Raw-Pointer leben nur innerhalb dieses `unsafe`-Blocks;
 * das Ergebnis ist ein reiner Rust-String
 */
pub fn format_mtime(secs: i64) -> String {
    unsafe {
        let mut tm: LibcTm = mem::zeroed();
        let t = secs;
        if localtime_r(&t, &mut tm).is_null() {
            return String::from("?");
        }
        let mut buf = [0i8; 64];
        let fmt = b"%e. %b %H:%M\0";
        let n = strftime(
            buf.as_mut_ptr(),
            buf.len(),
            fmt.as_ptr() as *const i8,
            &tm,
        );
        if n == 0 {
            return String::from("?");
        }
        std::ffi::CStr::from_ptr(buf.as_ptr())
            .to_string_lossy()
            .trim()
            .to_string()
    }
}

/**
 * Hilfsfunktionen auf Metadata (keine weiteren externen C-Calls)
 * Prüft, ob `path` ein Mount-Punkt ist (Device-ID des Verzeichnisses
 * unterscheidet sich von der des Elternverzeichnisses).
 * Nutzt nur sichere `std::fs`-APIs; die `dev()`-Methode stammt aus dem
 * Unix-Trait `MetadataExt` und liest intern `st_dev`.
 */
pub fn is_mount_point(path: &Path) -> bool {
    let st_path = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    let parent = path.join("..");
    let st_parent = match std::fs::metadata(&parent) {
        Ok(m) => m,
        Err(_) => return false,
    };
    st_path.dev() != st_parent.dev()
}

/**
 * Aktualisiert `timestamp` mit dem neuesten Änderungszeitpunkt aus metadata
 */
pub fn store_modification_timestamp(metadata: &Metadata, timestamp: &mut i64) {
    if let Ok(modified) = metadata.modified() {
        if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
            if *timestamp < duration.as_secs() as i64 {
                *timestamp = duration.as_secs() as i64;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uid_name_root_or_unknown() {
        let name = uid_name(0);// root
        assert!(!name.is_empty());
        let unknown = uid_name(u32::MAX);// unknown
        assert_eq!(unknown, "unknown");
    }

    #[test]
    fn format_mtime_epoch() {
        let s = format_mtime(0);
        assert_ne!(s, "?");
        assert!(!s.is_empty());
    }

    #[test]
    fn format_mtime_negative_still_string() {
        let s = format_mtime(-1);
        assert!(!s.is_empty());
    }

    #[test]
    fn gid_name_root_or_unknown() {
        let name = gid_name(0);
        assert!(!name.is_empty());
        let unknown = gid_name(u32::MAX);
        assert_eq!(unknown, "unknown");
    }

    #[test]
    fn is_mount_point_nonexistent() {
        assert!(!is_mount_point(Path::new("/nonexistent/path/xyz_lll_test")));
    }
}
