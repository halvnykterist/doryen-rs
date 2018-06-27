extern crate doryen_rs;

use doryen_rs::{App, AppOptions, DoryenApi, Engine, TextAlign};

const CONSOLE_WIDTH: u32 = 40;
const CONSOLE_HEIGHT: u32 = 25;

const FONTS:[&str;15] = ["terminal8x8_aa_ro.png","720x225_SmoothWalls.png","Aesomatica_16x16.png","Bisasam_20x20.png","Buddy--graphical.png",
    "Cheepicus_8x8.png","Cheepicus_15x15.png","Cheepicus_16x16.png","Herrbdog_144.png","Kein_400x125.png",
    "Mkv_curses_480x150.png","Runeset_24x24.png","Teeto_K_18x18.png","Terbert_7x7.png","Yayo_tunur_1040x325.png"];

struct MyRoguelike {
    cur_font: usize,
    cur_font_name: String,
}

impl Engine for MyRoguelike {
    fn update(&mut self, api: &mut DoryenApi) {
        let mut font_path=None;
        {
            let input = api.input();
            if input.key_released("PageDown") {
                self.cur_font = (self.cur_font + 1) % FONTS.len();
                font_path=Some(FONTS[self.cur_font]);
            } else if input.key_released("PageUp") {
                self.cur_font = (self.cur_font + FONTS.len() - 1) % FONTS.len();
                font_path=Some(FONTS[self.cur_font]);
            }
        }
        if let Some(font_path)=font_path {
            self.cur_font_name = font_path.to_owned();
            api.set_font_path(font_path);
        }
    }
    fn render(&mut self, api: &mut DoryenApi) {
        let con = api.con();
        con.rectangle(
            0,
            0,
            CONSOLE_WIDTH,
            CONSOLE_HEIGHT,
            Some((128, 128, 128, 255)),
            None,
            Some('.' as u16),
        );
        con.area(
            10,
            10,
            5,
            5,
            Some((255, 64, 64, 255)),
            Some((128, 32, 32, 255)),
            Some('&' as u16),
        );
        con.ascii((CONSOLE_WIDTH/2) as i32, (CONSOLE_HEIGHT/2-10) as i32, '@' as u16);
        con.fore((CONSOLE_WIDTH/2) as i32, (CONSOLE_HEIGHT/2-10) as i32, (255, 255, 255, 255));
        con.rectangle(
            (CONSOLE_WIDTH / 2 - 20) as i32,
            (CONSOLE_HEIGHT / 2 - 2) as i32,
            40,
            7,
            Some((255, 255, 255, 255)),
            Some((0, 0, 0, 255)),
            Some(' ' as u16),
        );
        con.print(
            (CONSOLE_WIDTH / 2) as i32,
            (CONSOLE_HEIGHT / 2) as i32,
            &self.cur_font_name,
            TextAlign::Center,
            Some((255, 255, 255, 255)),
            None,
        );
        con.print(
            (CONSOLE_WIDTH / 2) as i32,
            (CONSOLE_HEIGHT / 2) as i32 + 2,
            "PageUp/PageDown to change font",
            TextAlign::Center,
            Some((255, 192, 128, 255)),
            None,
        );
    }
}

impl MyRoguelike {
    pub fn new() -> Self {
        Self {
            cur_font:0,
            cur_font_name:FONTS[0].to_owned(),
        }
    }
}

fn main() {
    let mut app = App::new(AppOptions {
        console_width: CONSOLE_WIDTH,
        console_height: CONSOLE_HEIGHT,
        screen_width: CONSOLE_WIDTH * 24,
        screen_height: CONSOLE_HEIGHT * 24,
        window_title: "doryen-rs font test".to_owned(),
        font_path: "terminal8x8_aa_ro.png".to_owned(),
        vsync: true,
        fullscreen: false,
        show_cursor: false,
    });
    app.set_engine(Box::new(MyRoguelike::new()));
    app.run();
}
