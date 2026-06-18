use core::task::Poll::Pending;

use crate::{io, video::{self, MAX_HEIGHT, MAX_WIDTH, clear_screen, unpack_color}};

const FONTSIZE: usize = 8;
const PANDDING: usize = 5;
const MAXCOLS: usize = (MAX_WIDTH - PANDDING*2) / FONTSIZE;
const MAXROWS: usize = (MAX_HEIGHT - PANDDING*2) / FONTSIZE;

struct Terminal 
{
    cursor_x: usize,
    cursor_y: usize,
    cols: usize,
    rows: usize,
    fg_color: u32,
    bg_color: u32,
    buffer: [[u8; MAXCOLS]; MAXROWS]
}



static mut TERMINAL: Terminal = Terminal {
    cursor_x: 0,
    cursor_y: 0,
    cols: 0,
    rows: 0,
    fg_color: 0xFF883333,
    bg_color: 0xFFece8ba,
    buffer: [[b' '; MAXCOLS]; MAXROWS],
};


pub fn init()
{
    unsafe 
    {
        TERMINAL.cols = video::get_width()/FONTSIZE;
        TERMINAL.rows = video::get_height()/FONTSIZE;
    }
    self::clear();
}


pub fn clear()
{
    unsafe 
    {
        TERMINAL.cursor_x = 0;
        TERMINAL.cursor_y = 0;

        for r in 0..TERMINAL.rows 
        {
            for c in 0..TERMINAL.cols 
            {
                TERMINAL.buffer[r][c] = b' ';
            }
        }
    }

}


pub fn scroll_up()
{
    unsafe 
    {
        for r in 1..TERMINAL.rows {
            TERMINAL.buffer[r-1] = TERMINAL.buffer[r];
        }

        for c in 0..TERMINAL.cols {
            TERMINAL.buffer[TERMINAL.rows - 1][c] = b' ';
        }

        TERMINAL.cursor_y = TERMINAL.rows - 1;
    }
}


pub fn write_char(c: u8)
{
    unsafe 
    {
        match c 
        {
            b'\n' => {
                TERMINAL.cursor_x = 0;
                TERMINAL.cursor_y += 1;
            }

            b'\r' => {
                TERMINAL.cursor_x = 0;
            }

            _ => {
                if TERMINAL.cursor_x < TERMINAL.cols && TERMINAL.cursor_y < TERMINAL.rows {
                    TERMINAL.buffer[TERMINAL.cursor_y][TERMINAL.cursor_x] = c;
                    TERMINAL.cursor_x += 1;
                }
            }
        }
        if TERMINAL.cursor_y >= TERMINAL.rows {
            scroll_up();
        }
    }
}


pub fn write_str(s: &str)
{
    for b in s.bytes() 
    {
        write_char(b);
    }
    io::pss(s);
}


pub fn render()
{
    unsafe 
    {
        clear_screen(TERMINAL.bg_color);
        for r in 0..TERMINAL.rows
        {
            for c in 0..TERMINAL.cols
            {
                let caracter = TERMINAL.buffer[r][c] as char;
                
                if caracter != ' ' 
                {
                    let pixel_x = c * FONTSIZE;
                    let pixel_y = r * FONTSIZE;

                    video::draw_char(pixel_x + PANDDING, pixel_y+ PANDDING, caracter, TERMINAL.fg_color);
                }
            }
        }
    }
}