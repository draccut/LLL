pub const FILE_SYSTEM_MAX_PATH: usize = 0xFFF;
pub const NR_OF_SUPPORTET_LANG: usize = 13;
pub const CARD_LEN: usize = 12;
pub const ERRR: i32 = 0x0FFF_AFFF_u32 as i32;

pub const LLL_CLI_HELP_MESSAGE: &str = "\
rustc -O -C panic=abort lll.rs -o lll
-s --size   sort by size
-l --ls     ls like output
-h --help   print help
-p --path   set path default is current
-q --qqqq   print all absolut paths of a folder
@VERSION    12.0.0
";

/**
 * gruppiert verwandte Erweiterungen z. B. .c und .h -> C
 */
#[derive(Clone, Copy)]
pub struct TypeEntry {
    pub name: &'static str,
    pub countet_size: u64,
    pub globalsumindex: i32,
}

pub static mut TYPES: [TypeEntry; NR_OF_SUPPORTET_LANG] = [
    TypeEntry {
        name: "",
        countet_size: 0,
        globalsumindex: 0,
    },
    TypeEntry {
        name: "c",
        countet_size: 0,
        globalsumindex: 1,
    },
    TypeEntry {
        name: "cpp",
        countet_size: 0,
        globalsumindex: 2,
    },
    TypeEntry {
        name: "ino",
        countet_size: 0,
        globalsumindex: 3,
    },
    TypeEntry {
        name: "py",
        countet_size: 0,
        globalsumindex: 4,
    },
    TypeEntry {
        name: "java",
        countet_size: 0,
        globalsumindex: 5,
    },
    TypeEntry {
        name: "sh",
        countet_size: 0,
        globalsumindex: 6,
    },
    TypeEntry {
        name: "js",
        countet_size: 0,
        globalsumindex: 7,
    },
    TypeEntry {
        name: "html",
        countet_size: 0,
        globalsumindex: 8,
    },
    TypeEntry {
        name: "asm",
        countet_size: 0,
        globalsumindex: 9,
    },
    TypeEntry {
        name: "rs",
        countet_size: 0,
        globalsumindex: 10,
    },
    TypeEntry {
        name: "sql",
        countet_size: 0,
        globalsumindex: 11,
    },
    TypeEntry {
        name: "h",
        countet_size: 0,
        globalsumindex: 1,
    },
];

pub static NAME_LIST: [&str; NR_OF_SUPPORTET_LANG] = [
    "Undefined",
    "C",
    "C++",
    "ESP",
    "Python",
    "Java",
    "ShellScript",
    "JavaScript",
    "HTML",
    "Assembly",
    "Rust",
    "SQL",
    "C",
];

pub static COLOR_LIST: [u64; NR_OF_SUPPORTET_LANG] = [
    0x2222_0778_77CC,
    0x2222_0566_6666,
    0x2222_0A22_22FF,
    0x2222_0FEE_AA00,
    0x2222_00A8_A862,
    0x2222_00FF_FF00,
    0x2222_00FF_0010,
    0x2222_00BB_0010,
    0x2222_00CC_66FF,
    0x2222_00FF_8800,
    0x2222_00DE_A584,
    0x2222_0000_00FF,
    0x2222_0566_6666,
];

pub mod flags {
    pub const SORT_BY_SIZE: u32 = 1 << 0;
    pub const LS_MODE: u32 = 1 << 1;
    pub const HELP: u32 = 1 << 2;
    pub const PATH: u32 = 1 << 3;
    pub const ABSOLUT_PATHS: u32 = 1 << 4;
    pub const SINGLE_NAME: u32 = 1 << 5;
}
