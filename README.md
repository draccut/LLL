# LLL 

wie ll -A oder ls oder ls -A in der Linux Shell

Ein Directory-Lister in Rust mit rekursiver Größenaggregation, dominanter Programmiersprachen-Erkennung und farbiger Terminalausgabe. Bewusst nah an einer ausgereiften C-Variante gehalten.

----

## Features & Funktionsweise

### Bessere Speicherdarstellung

Statt roher Bytezahlen zeigt `lll` Größen human-readable (`process_size`): B → KB → MB → GB → TB, jeweils mit einer Nachkommastelle. Verzeichnisse werden **rekursiv** summiert (`folder_size`), Symlinks und Mountpoints ausgelassen bzw. abgeschnitten, die angezeigte Größe ist die tatsächliche Nutzlast unterhalb des Eintrags, nicht nur `st_size` des Directory-Inodes.

Pro Eintrag werden Metadaten parallel in drei Vektoren gehalten:

| Vektor | Typ | Zweck |
|--------|-----|--------|
| `rows` | `Vec<String>` | fertig formatierte stdout Zeile (Permissions, Owner, Size, Zeit, Sprache, Name) |
| `sizes` | `Vec<u64>` | Rohgrößen in Bytes für `-s` / `--size` |
| `unix_timestamps` | `Vec<i64>` | mtime (bei Dirs: max. mtime im Teilbaum) für Default-Sortierung |

Auswahl-Sortierung (Selection Sort) tauscht Indizes in `sizes`/`unix_timestamps` und denselben Index in `rows`, stabile Zuordnung ohne Struct-of-Arrays-Umbau zur Laufzeit. Default: neueste zuerst, `-s`: kleinste zuerst.

### Programmiersprachen-Infos

Statische Tabellen (`TYPES`, `NAME_LIST`, `COLOR_LIST`) mappe Extension, Anzeigename, RGB-Farbe:

`c`/`h`, `cpp`, `ino`, `py`, `java`, `sh`, `js`, `html`, `asm`, `rs`, `sql`

Beim Durchlauf schreibt `get_type` die Dateigröße in den passenden `TypeEntry::countet_size`. `globalsumindex` fasst verwandte Extensions zusammen (z. B. `.c` und `.h` werden zu C). Nach dem Scan eines Verzeichnisses liefert `get_dom_lang_index` den Index mit der größten kumulierten Byte-Masse und **nullt** die Zähler, so trägt jeder Ordner nur seine eigene Dominanz, ohne globale Drift zwischen Geschwister-Einträgen.

Einzeldateien nutzen `get_dom_single_file_index_from_path` (nur Extension, kein Aggregat). Die Karte (`chngg_wrapper` / `coloramaprintifarmafillup`) ist fest `CARD_LEN` Zeichen breit, UTF-8-sicher gepaddet (`pad_utf8` + `utf8_char_len_local`), mit Truecolor-FG/BG.

## CLI Arguments

```text
lll              # Detail, Sortierung nach mtime
lll -s|--size    # Detail, Sortierung nach Größe
lll -l|--ls      # kompaktes Mehrspalten-Layout
lll -p|--path P  # anderes Startverzeichnis
lll -q|--qqqq [P]# absolute Pfade auflisten
lll NAME         # absolut + xclip
lll -h|--help
```

## Build

```bash
cargo test
cargo clean
cargo build --release
```

----

## C-std-lib in c_bindings.rs kapseln

C-Bindings für Standard-Linux-Libs (libc).

Dieses Modul kapselt alle `unsafe`-Aufrufe an die C-Bibliothek und
liefert ausschließlich idiomatische, sichere Rust-Typen zurück.

## Sicherheitsmodell

1. **Rohzeiger aus libc** (`*mut Passwd`, `*mut Group`, `*mut LibcTm`)
    werden nie nach außen exponiert.

2.  Jeder Funktionszeiger wird nur innerhalb eines `unsafe`-Blocks
    dereferenziert; danach werden die Felder sofort in eigene
    `String`/`i64`-Werte kopiert.

3.  Null-Pointer-Checks und Lebensdauer-Garantien (libc hält die
    Passwd/Group-Strukturen in Thread-lokalen Puffern) werden explizit
    dokumentiert und abgesichert.

4.  Das Ergebnis ist immer ein owned Rust-Wert.


