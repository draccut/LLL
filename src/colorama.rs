use crate::data_segment::{CARD_LEN, COLOR_LIST, NAME_LIST, NR_OF_SUPPORTET_LANG};

fn utf8_char_len_local(c: u8) -> usize {
    if c < 0x80 {
        1
    } else if (c >> 5) == 0x6 {
        2
    } else if (c >> 4) == 0xE {
        3
    } else if (c >> 3) == 0x1E {
        4
    } else {
        1
    }
}

/**
 * füllt mit Leerzeichen auf, bis genau `mse` Bytes erreicht sind.*
 */
pub fn pad_utf8(text: &str, mse: usize) -> String {
    let bytes = text.as_bytes();
    let mut res = Vec::with_capacity(mse + 4);
    let mut pos = 0usize;
    let mut i = 0usize;
    while pos < mse && i < bytes.len() {
        let clen = utf8_char_len_local(bytes[i]);
        if pos + clen > mse {
            break;
        }
        if i + clen > bytes.len() {
            break;
        }
        res.extend_from_slice(&bytes[i..i + clen]);
        pos += clen;
        i += clen;
    }
    while pos < mse {
        res.push(b' ');
        pos += 1;
    }
    String::from_utf8_lossy(&res).into_owned()
}

/**
 * Volle 48-Bit-Farbe (24 Bit FG + 24 Bit BG) als ANSI-Escape.
 *
 * Layout von `col`:
 * `0xBB_GG_RR_bb_gg_rr` (Groß = Hintergrund, Klein = Vordergrund)
 */
pub fn coloramaprintifarma(ptext: &str, col: u64) -> String {
    let bg_r = ((col >> 40) & 0xFF) as u8;
    let bg_g = ((col >> 32) & 0xFF) as u8;
    let bg_b = ((col >> 24) & 0xFF) as u8;
    let fg_r = ((col >> 16) & 0xFF) as u8;
    let fg_g = ((col >> 8) & 0xFF) as u8;
    let fg_b = (col & 0xFF) as u8;
    format!(
        "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}\x1b[0m",
        fg_r, fg_g, fg_b, bg_r, bg_g, bg_b, ptext
    )
}

pub fn coloramaprintifarmafillup(text: &str, advansed_color: u64) -> String {
    let padded = pad_utf8(text, CARD_LEN);
    coloramaprintifarma(&padded, advansed_color)
}

/**
 * Nur Vordergrundfarbe 24 Bit RGB
 */
pub fn coll(col: u64, ptext: &str) -> String {
    let r = ((col >> 16) & 0xFF) as u8;
    let g = ((col >> 8) & 0xFF) as u8;
    let b = (col & 0xFF) as u8;
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, ptext)
}

pub fn chngg(pp: &str, color: u64, more_space: i32) -> String {
    let mse = match more_space {
        1 => CARD_LEN * 3,
        2 => CARD_LEN + 3,
        3 => CARD_LEN.saturating_sub(3),
        _ => CARD_LEN,
    };
    let padded = pad_utf8(pp, mse);
    coll(color, &padded)
}

pub fn chngg_wrapper(index: i32) -> String {
    let idx = if index < 0 || index as usize >= NR_OF_SUPPORTET_LANG {
        0
    } else {
        index as usize
    };
    coloramaprintifarmafillup(NAME_LIST[idx], COLOR_LIST[idx])
}

/**
 * Mode-Bits (Unix-Permissions) -> rwxrwxrwx-String inkl. Dateityp-Zeichen
 */
pub fn mode_to_str(mode: u32) -> String {
    let mut out = String::with_capacity(10);
    let ft = mode & 0o170000;
    out.push(match ft {
        0o040000 => 'd',
        0o120000 => 'l',
        0o020000 => 'c',
        0o060000 => 'b',
        0o010000 => 'p',
        0o140000 => 's',
        _ => '-',
    });
    out.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    out.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    out.push(if mode & 0o100 != 0 { 'x' } else { '-' });
    out.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    out.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    out.push(if mode & 0o010 != 0 { 'x' } else { '-' });
    out.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    out.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    out.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    out
}

pub fn process_size(size: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut s = size as f64;
    let mut u = 0usize;
    while s >= 1024.0 && u < 4 {
        s /= 1024.0;
        u += 1;
    }
    format!("{:.1} {}", s, units[u])
}

/**
 * Entfernt Steuerzeichen aus einem Dateinamen Terminal-sicher
 */
pub fn safe_terminal_string(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_control() {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_char_len_ascii() {
        assert_eq!(utf8_char_len_local(b'A'), 1);
    }

    #[test]
    fn utf8_char_len_2byte() {
        assert_eq!(utf8_char_len_local(0xC3), 2);
    }

    #[test]
    fn utf8_char_len_3byte() {
        assert_eq!(utf8_char_len_local(0xE2), 3);
    }

    #[test]
    fn utf8_char_len_4byte() {
        assert_eq!(utf8_char_len_local(0xF0), 4);// Emoji
    }

    #[test]
    fn pad_utf8_exact() {
        let s = pad_utf8("hi", 5);
        assert_eq!(s.len(), 5);
        assert!(s.starts_with("hi"));
        assert!(s.ends_with("   "));
    }

    #[test]
    fn pad_utf8_truncate_multibyte() {
        let s = pad_utf8("äx", 2);
        assert_eq!(s, "ä");
        let s1 = pad_utf8("äx", 1);
        assert_eq!(s1, " ");
        let s4 = pad_utf8("äx", 4);
        assert_eq!(s4, "äx ");
    }

    #[test]
    fn pad_utf8_empty() {
        let s = pad_utf8("", 3);
        assert_eq!(s, "   ");
    }

    #[test]
    fn process_size_bytes() {
        assert_eq!(process_size(500), "500.0 B");
    }

    #[test]
    fn process_size_kb() {
        assert_eq!(process_size(2048), "2.0 KB");
    }

    #[test]
    fn process_size_mb() {
        assert_eq!(process_size(5 * 1024 * 1024), "5.0 MB");
    }

    #[test]
    fn mode_to_str_regular_file() {
        let s = mode_to_str(0o100644);
        assert_eq!(s, "-rw-r--r--");
    }

    #[test]
    fn mode_to_str_directory() {
        let s = mode_to_str(0o040755);
        assert_eq!(s, "drwxr-xr-x");
    }

    #[test]
    fn mode_to_str_symlink() {
        let s = mode_to_str(0o120777);
        assert_eq!(s, "lrwxrwxrwx");
    }

    #[test]
    fn coll_contains_ansi() {
        let s = coll(0xFF0000, "red");
        assert!(s.contains("\x1b[38;2;255;0;0m"));
        assert!(s.contains("red"));
        assert!(s.ends_with("\x1b[0m"));
    }

    #[test]
    fn coloramaprintifarma_contains_fg_and_bg() {
        let col = 0x1122_3344_5566u64;
        let s = coloramaprintifarma("x", col);
        assert!(s.contains("38;2;68;85;102"));
        assert!(s.contains("48;2;17;34;51"));
    }

    #[test]
    fn chngg_wrapper_out_of_range() {
        let s = chngg_wrapper(-1);
        assert!(s.contains("Undefined") || s.contains("\x1b"));
        let s2 = chngg_wrapper(999);
        assert!(s2.contains("Undefined") || s2.contains("\x1b"));
    }

    #[test]
    fn safe_terminal_string_control() {
        let s = safe_terminal_string("a\nb");
        assert!(!s.contains('\n'));
        assert!(s.contains("\\n") || s.contains("a") && s.contains("b"));
    }

    #[test]
    fn safe_terminal_string_normal() {
        assert_eq!(safe_terminal_string("hello"), "hello");
    }

    #[test]
    fn pad_utf8_zero_width() {
        assert_eq!(pad_utf8("abc", 0), "");
    }

    #[test]
    fn process_size_zero() {
        assert_eq!(process_size(0), "0.0 B");
    }

    #[test]
    fn process_size_huge() {
        // 1 TB-ish
        let s = process_size(1024u64.pow(4));
        assert!(s.contains("TB") || s.contains("GB"));
    }

    #[test]
    fn chngg_more_space_variants() {
        let a = chngg("x", 0xFF0000, 1);
        let b = chngg("x", 0xFF0000, 2);
        let c = chngg("x", 0xFF0000, 3);
        let d = chngg("x", 0xFF0000, 0);
        assert!(a.contains("x"));
        assert!(b.contains("x"));
        assert!(c.contains("x"));
        assert!(d.contains("x"));
        assert!(a.len() >= b.len());
    }
}
