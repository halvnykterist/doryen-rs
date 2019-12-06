use doryen_fov::{FovAlgorithm, FovRestrictive, MapData};
use doryen_rs::{color_blend, color_mul, color_scale, Color, DoryenApi, Image};

use crate::light::Light;

const START_COLOR: Color = (255, 0, 0, 255);
const LIGHT_COLOR: Color = (255, 255, 0, 255);
const LIGHT_RADIUS: f32 = 15.0;
const PLAYER_LIGHT_RADIUS: f32 = 8.0;
const PLAYER_LIGHT_COLOR: Color = (150, 150, 150, 255);
const LIGHT_COEF: f32 = 1.5;
const WALL_COLOR: Color = (255, 255, 255, 255);
const GOBLIN_COLOR: Color = (0, 255, 0, 255);
const VISITED_BLEND_COLOR: Color = (10, 10, 40, 255);
const VISITED_BLEND_COEF: f32 = 0.8;

pub struct Level {
    level_img: Image,
    ground: Image,
    loaded: bool,
    lightmap: Image,
    size: (i32, i32),
    start: (i32, i32),
    walls: Vec<bool>,
    visited_2x: Vec<bool>,
    fov: FovRestrictive,
    map: MapData,
    render_output: Image,
    lights: Vec<Light>,
    player_light: Light,
    goblins: Vec<(i32, i32)>,
}

impl Level {
    pub fn new(img_path: &str) -> Self {
        Self {
            level_img: Image::new(&(img_path.to_owned() + ".png")),
            ground: Image::new(&(img_path.to_owned() + "_color.png")),
            loaded: false,
            lightmap: Image::new_empty(1, 1),
            render_output: Image::new_empty(1, 1),
            size: (0, 0),
            start: (0, 0),
            walls: Vec::new(),
            visited_2x: Vec::new(),
            fov: FovRestrictive::new(),
            map: MapData::new(1, 1),
            lights: Vec::new(),
            goblins: Vec::new(),
            player_light: Light::new((0, 0), PLAYER_LIGHT_RADIUS, PLAYER_LIGHT_COLOR),
        }
    }
    pub fn try_load(&mut self) -> bool {
        if !self.loaded {
            if self.level_img.try_load() {
                self.compute_walls_2x_and_start_pos();
                self.compute_walls();
                self.lightmap = Image::new_empty(self.size.0 as u32 * 2, self.size.1 as u32 * 2);
                self.render_output =
                    Image::new_empty(self.size.0 as u32 * 2, self.size.1 as u32 * 2);
                self.loaded = true;
                // free memory
                self.level_img = Image::new_empty(1, 1);
            } else {
                return false;
            }
        }
        true
    }
    pub fn start_pos(&self) -> (i32, i32) {
        self.start
    }
    pub fn is_wall(&self, pos: (i32, i32)) -> bool {
        self.walls[self.offset(pos)]
    }
    pub fn light_at(&self, (x, y): (i32, i32)) -> Color {
        self.lightmap.pixel(x as u32 * 2, y as u32 * 2).unwrap()
    }
    pub fn update(&mut self) {
        for light in self.lights.iter_mut() {
            light.update();
        }
    }
    fn penumbra(color: Color, level: usize) -> bool {
        (color.0 as usize + color.1 as usize + color.2 as usize) < level
    }
    pub fn render(&mut self, api: &mut dyn DoryenApi, player_pos: (i32, i32)) {
        if self.ground.try_load() {
            self.compute_lightmap(player_pos);
            let mut con = api.con();
            for y in 0..self.size.1 as usize * 2 {
                for x in 0..self.size.0 as usize * 2 {
                    let off = self.offset_2x((x as i32, y as i32));
                    if self.map.is_in_fov(x, y) {
                        let ground_col = self.ground.pixel(x as u32, y as u32).unwrap();
                        let light_col = self.lightmap.pixel(x as u32, y as u32).unwrap();
                        let penumbra = Level::penumbra(light_col, 50);
                        let mut r =
                            f32::from(ground_col.0) * f32::from(light_col.0) * LIGHT_COEF / 255.0;
                        let mut g =
                            f32::from(ground_col.1) * f32::from(light_col.1) * LIGHT_COEF / 255.0;
                        let mut b =
                            f32::from(ground_col.2) * f32::from(light_col.2) * LIGHT_COEF / 255.0;
                        r = r.min(255.0);
                        g = g.min(255.0);
                        b = b.min(255.0);
                        self.render_output.put_pixel(
                            x as u32,
                            y as u32,
                            (r as u8, g as u8, b as u8, 255),
                        );
                        if !penumbra {
                            self.visited_2x[off] = true;
                        }
                    } else if self.visited_2x[off] {
                        let col = self.ground.pixel(x as u32, y as u32).unwrap();
                        let dark_col = color_blend(col, VISITED_BLEND_COLOR, VISITED_BLEND_COEF);
                        self.render_output.put_pixel(x as u32, y as u32, dark_col);
                    } else {
                        self.render_output
                            .put_pixel(x as u32, y as u32, (0, 0, 0, 255));
                    }
                }
            }
            self.render_output
                .blit_2x(&mut con, 0, 0, 0, 0, None, None, None);
            self.render_goblins(api);
        }
    }
    fn render_goblins(&self, api: &mut dyn DoryenApi) {
        for goblin in self.goblins.iter() {
            if self
                .map
                .is_in_fov(goblin.0 as usize * 2, goblin.1 as usize * 2)
            {
                self.render_creature(api, 'g' as u16, *goblin, GOBLIN_COLOR);
            }
        }
    }
    pub fn compute_fov(&mut self, (x, y): (i32, i32), radius: usize) {
        self.map.clear_fov();
        self.fov
            .compute_fov(&mut self.map, x as usize * 2, y as usize * 2, radius, true);
    }
    fn add_light(&mut self, pos: (i32, i32)) {
        self.lights.push(Light::new(pos, LIGHT_RADIUS, LIGHT_COLOR));
    }
    fn render_creature(&self, api: &mut dyn DoryenApi, c: u16, (x, y): (i32, i32), color: Color) {
        let light = self.lightmap.pixel(x as u32 * 2, y as u32 * 2).unwrap();
        let penumbra = Level::penumbra(light, 130);
        let mut color = color_mul(color, light);
        if penumbra {
            color = color_scale(color, LIGHT_COEF);
        }
        api.con().ascii(x, y, if penumbra { '?' as u16 } else { c });
        api.con().fore(x, y, color);
    }
    fn compute_lightmap(&mut self, (px, py): (i32, i32)) {
        // TODO check if filling with black pixels is faster
        self.lightmap = Image::new_empty(self.size.0 as u32 * 2, self.size.1 as u32 * 2);
        let mut fov = FovRestrictive::new();
        *self.player_light.pos_mut() = ((px * 2) as f32, (py * 2) as f32);
        self.player_light
            .render(&mut self.map, &mut fov, &mut self.lightmap, false);
        for light in self.lights.iter() {
            light.render(&mut self.map, &mut fov, &mut self.lightmap, true);
        }
    }
    fn compute_walls_2x_and_start_pos(&mut self) {
        let size = self.level_img.try_get_size().unwrap();
        self.map = MapData::new(size.0 as usize, size.1 as usize);
        for y in 0..size.1 {
            for x in 0..size.0 {
                let p = self.level_img.pixel(x, y).unwrap();
                self.map
                    .set_transparent(x as usize, y as usize, p != WALL_COLOR);
                self.visited_2x.push(false);
                match p {
                    START_COLOR => self.start = (x as i32 / 2, y as i32 / 2),
                    LIGHT_COLOR => self.add_light((x as i32, y as i32)),
                    GOBLIN_COLOR => self.goblins.push((x as i32 / 2, y as i32 / 2)),
                    _ => (),
                }
            }
        }
        self.size = (size.0 as i32 / 2, size.1 as i32 / 2);
    }
    fn compute_walls(&mut self) {
        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                let mut count = 0;
                let x2 = x as usize * 2;
                let y2 = y as usize * 2;
                if self.map.is_transparent(x2, y2) {
                    count += 1;
                }
                if self.map.is_transparent(x2 + 1, y2) {
                    count += 1;
                }
                if self.map.is_transparent(x2, y2 + 1) {
                    count += 1;
                }
                if self.map.is_transparent(x2 + 1, y2 + 1) {
                    count += 1;
                }
                self.walls.push(count < 2);
            }
        }
        for goblin in self.goblins.iter() {
            let off = self.offset(*goblin);
            self.walls[off] = true;
        }
    }
    fn offset(&self, (x, y): (i32, i32)) -> usize {
        (x + y * self.size.0 as i32) as usize
    }
    fn offset_2x(&self, (x, y): (i32, i32)) -> usize {
        (x + y * self.size.0 as i32 * 2) as usize
    }
}