#![allow(dead_code)]
/// Pancurses assumes a different backend than the one we've replaced with a
/// hack in the fork of pancurses, so we need to redefine these curses attributes

use pancurses;

pub const A_ALTCHARSET: pancurses::chtype = 0x00010000;
pub const A_RIGHT: pancurses::chtype = 0x00020000;
pub const A_LEFT: pancurses::chtype = 0x00040000;
pub const A_ITALIC: pancurses::chtype = 0x00080000;
pub const A_UNDERLINE: pancurses::chtype = 0x00100000;
pub const A_REVERSE: pancurses::chtype = 0x00200000;
pub const A_BLINK: pancurses::chtype = 0x00400000;
pub const A_BOLD: pancurses::chtype = 0x00800000;
pub const A_NORMAL: pancurses::chtype = 0x0;
