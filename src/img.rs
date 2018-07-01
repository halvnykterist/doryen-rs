use color::{color_blend, color_dist, Color};
use console::*;
use file::FileLoader;
use image;

/// An easy way to load PNG images and blit them on the console
pub struct Image {
    file_loader: FileLoader,
    img: Option<image::RgbaImage>,
}

impl Image {
    /// Create an image and load a PNG file.
    /// On the web platform, image loading is asynchronous.
    /// Using blit methods before the image is loaded has no impact on the console.
    pub fn new(file_path: &str) -> Self {
        let mut file_loader = FileLoader::new();
        file_loader.load_file(file_path);
        Self {
            file_loader,
            img: None,
        }
    }
    /// Check if the image has been loaded.
    /// Since there's no background thread doing the work for you, you have to call some method on image for it to actually load.
    /// Use either [`Image::is_loaded`], [`Image::get_size`], [`Image::blit`] or [`Image::blit_ex`] to run the loading code.
    pub fn is_loaded(&mut self) -> bool {
        if self.img.is_some() {
            return true;
        }
        if self.file_loader.is_file_ready(0) {
            let buf = self.file_loader.get_file_content(0);
            self.intialize_image(&buf);
            return true;
        }
        return false;
    }
    fn intialize_image(&mut self, buf: &Vec<u8>) {
        self.img = Some(image::load_from_memory(&buf).unwrap().to_rgba());
    }
    /// If the image has already been loaded, return its size, else return None
    pub fn get_size(&mut self) -> Option<(u32, u32)> {
        if self.is_loaded() {
            if let Some(ref img) = self.img {
                return Some((img.width(), img.height()));
            }
        }
        return None;
    }
    /// blit an image on a console
    ///
    /// x,y are the coordinate of the top left image pixel in the console
    ///
    /// image pixels using the transparent color will be ignored
    pub fn blit(&mut self, con: &mut Console, x: i32, y: i32, transparent: Option<Color>) {
        if !self.is_loaded() {
            return;
        }
        if let Some(ref img) = self.img {
            let width = img.width() as i32;
            let height = img.height() as i32;
            let minx = x.max(0);
            let miny = y.max(0);
            let maxx = (x + width).min(con.get_width() as i32);
            let maxy = (y + height).min(con.get_height() as i32);
            let mut offx = if x < 0 { -x } else { 0 };
            let mut offy = if y < 0 { -y } else { 0 };
            let con_width = con.get_pot_width();
            let back = con.borrow_mut_background();
            for cx in minx..maxx {
                for cy in miny..maxy {
                    let pixel = img
                        .get_pixel((cx - minx + offx) as u32, (cy - miny + offy) as u32)
                        .data;
                    let color = (pixel[0], pixel[1], pixel[2], pixel[3]);
                    if let Some(ref t) = transparent {
                        if color == *t {
                            continue;
                        }
                    }
                    let offset = (cx as u32 + cy as u32 * con_width) as usize;
                    back[offset] = color;
                }
            }
        }
    }
    /// blit an image on a console
    ///
    /// x,y are the coordinate of the image center in the console
    /// image can be scaled and rotated (angle is in radians)
    /// image pixels using the transparent color will be ignored
    pub fn blit_ex(
        &mut self,
        con: &mut Console,
        x: f32,
        y: f32,
        scalex: f32,
        scaley: f32,
        angle: f32,
        transparent: Option<Color>,
    ) {
        if !self.is_loaded() || scalex == 0.0 || scaley == 0.0 {
            return;
        }
        let size = self.get_size().unwrap();
        let rx = x - size.0 as f32 * 0.5;
        let ry = y - size.1 as f32 * 0.5;
        if scalex == 1.0 && scaley == 1.0 && angle == 0.0 && rx.floor() == rx && ry.floor() == ry {
            let ix = rx as i32;
            let iy = ry as i32;
            self.blit(con, ix, iy, transparent);
            return;
        }
        let iw = (size.0 / 2) as f32 * scalex;
        let ih = (size.1 / 2) as f32 * scaley;
        // get the coordinates of the image corners in the console
        let newx_x = angle.cos();
        let newx_y = -angle.sin();
        let newy_x = newx_y;
        let newy_y = -newx_x;
        // image corners coordinates
        // 0 = P - w/2 x' +h/2 y'
        let x0 = x - iw * newx_x + ih * newy_x;
        let y0 = y - iw * newx_y + ih * newy_y;
        // 1 = P + w/2 x' + h/2 y'
        let x1 = x + iw * newx_x + ih * newy_x;
        let y1 = y + iw * newx_y + ih * newy_y;
        // 2 = P + w/2 x' - h/2 y'
        let x2 = x + iw * newx_x - ih * newy_x;
        let y2 = y + iw * newx_y - ih * newy_y;
        // 3 = P - w/2 x' - h/2 y'
        let x3 = x - iw * newx_x - ih * newy_x;
        let y3 = y - iw * newx_y - ih * newy_y;
        // get the affected rectangular area in the console
        let rx = x0.min(x1).min(x2).min(x3) as i32;
        let ry = y0.min(y1).min(y2).min(y3) as i32;
        let rw = x0.max(x1).max(x2).max(x3) as i32 - rx;
        let rh = y0.max(y1).max(y2).max(y3) as i32 - ry;
        // clip it
        let minx = rx.max(0);
        let miny = ry.max(0);
        let maxx = (rx + rw).min(con.get_width() as i32);
        let maxy = (ry + rh).min(con.get_height() as i32);
        let invscalex = 1.0 / scalex;
        let invscaley = 1.0 / scaley;
        let con_width = con.get_pot_width();
        let back = con.borrow_mut_background();
        if let Some(ref img) = self.img {
            for cx in minx..maxx {
                for cy in miny..maxy {
                    // map the console pixel to the image world
                    let ix =
                        (iw + (cx as f32 - x) * newx_x + (cy as f32 - y) * (-newy_x)) * invscalex;
                    let iy =
                        (ih + (cx as f32 - x) * (newx_y) - (cy as f32 - y) * newy_y) * invscaley;
                    let color = if ix as i32 >= size.0 as i32
                        || ix < 0.0
                        || iy as i32 >= size.1 as i32
                        || iy < 0.0
                    {
                        (0, 0, 0, 255)
                    } else {
                        let pixel = img.get_pixel(ix as u32, iy as u32).data;
                        (pixel[0], pixel[1], pixel[2], pixel[3])
                    };
                    if let Some(ref t) = transparent {
                        if color == *t {
                            continue;
                        }
                    }
                    let offset = (cx as u32 + cy as u32 * con_width) as usize;
                    if scalex < 1.0 || scaley < 1.0 {
                        // todo mipmap
                    }
                    back[offset] = color;
                }
            }
        }
    }

    /// blit an image on the console, using the subcell characters to achieve twice the normal resolution.
    /// This uses the CHAR_SUBCELL_* ascii codes (from 226 to 232):
    ///
    /// ![subcell_chars](http://roguecentral.org/~jice/doryen-rs/subcell_chars.png)
    ///
    /// COmparison before/after subcell in the chronicles of Doryen :
    ///
    /// ![subcell_comp](http://roguecentral.org/~jice/doryen-rs/subcell_comp.png)
    ///
    /// Pyromancer! screenshot, making full usage of subcell resolution:
    ///
    /// ![subcell_pyro](http://roguecentral.org/~jice/doryen-rs/subcell_pyro.png)
    pub fn blit_2x(
        &mut self,
        con: &mut Console,
        dx: i32,
        dy: i32,
        sx: i32,
        sy: i32,
        w: Option<i32>,
        h: Option<i32>,
        transparent: Option<Color>,
    ) {
        if !self.is_loaded() {
            return;
        }
        let mut grid: [Color; 4] = [(0, 0, 0, 0), (0, 0, 0, 0), (0, 0, 0, 0), (0, 0, 0, 0)];
        let mut back: Color = (0, 0, 0, 0);
        let mut front: Option<Color> = None;
        let mut ascii: i32 = ' ' as i32;

        if let Some(ref img) = self.img {
            let width = img.width() as i32;
            let height = img.height() as i32;
            let con_width = con.get_width() as i32;
            let con_height = con.get_height() as i32;
            let mut blit_w = w.unwrap_or(width);
            let mut blit_h = h.unwrap_or(height);
            let minx = sx.max(0);
            let miny = sy.max(0);
            blit_w = blit_w.min(width - minx);
            blit_h = blit_h.min(height - miny);
            let mut maxx = if dx + blit_w / 2 <= con_width {
                blit_w
            } else {
                (con_width - dx) * 2
            };
            let mut maxy = if dy + blit_h / 2 <= con_height {
                blit_h
            } else {
                (con_height - dy) * 2
            };
            maxx += minx;
            maxy += miny;
            let mut cx = minx;
            while cx < maxx {
                let mut cy = miny;
                while cy < maxy {
                    // get the 2x2 super pixel colors from the image
                    let conx = dx + (cx - minx) / 2;
                    let cony = dy + (cy - miny) / 2;
                    let console_back = con.unsafe_get_back(conx, cony);
                    let pixel = img.get_pixel(cx as u32, cy as u32).data;
                    grid[0] = (pixel[0], pixel[1], pixel[2], pixel[3]);
                    if let Some(ref t) = transparent {
                        if grid[0] == *t {
                            grid[0] = console_back;
                        }
                    }
                    if cx < maxx - 1 {
                        let pixel = img.get_pixel(cx as u32 + 1, cy as u32).data;
                        grid[1] = (pixel[0], pixel[1], pixel[2], pixel[3]);
                        if let Some(ref t) = transparent {
                            if grid[1] == *t {
                                grid[1] = console_back;
                            }
                        }
                    } else {
                        grid[1] = console_back;
                    }
                    if cy < maxy - 1 {
                        let pixel = img.get_pixel(cx as u32, cy as u32 + 1).data;
                        grid[2] = (pixel[0], pixel[1], pixel[2], pixel[3]);
                        if let Some(ref t) = transparent {
                            if grid[2] == *t {
                                grid[2] = console_back;
                            }
                        }
                    } else {
                        grid[2] = console_back;
                    }
                    if cx < maxx - 1 && cy < maxy - 1 {
                        let pixel = img.get_pixel(cx as u32 + 1, cy as u32 + 1).data;
                        grid[3] = (pixel[0], pixel[1], pixel[2], pixel[3]);
                        if let Some(ref t) = transparent {
                            if grid[3] == *t {
                                grid[3] = console_back;
                            }
                        }
                    } else {
                        grid[3] = console_back;
                    }
                    // analyse color, posterize, get pattern
                    compute_pattern(&grid, &mut back, &mut front, &mut ascii);
                    if front.is_none() {
                        // single color
                        con.unsafe_back(conx, cony, back);
                        con.unsafe_ascii(conx, cony, ascii as u16);
                    } else {
                        if ascii >= 0 {
                            con.unsafe_back(conx, cony, back);
                            con.unsafe_fore(conx, cony, front.unwrap());
                            con.unsafe_ascii(conx, cony, ascii as u16);
                        } else {
                            con.unsafe_back(conx, cony, front.unwrap());
                            con.unsafe_fore(conx, cony, back);
                            con.unsafe_ascii(conx, cony, (-ascii) as u16);
                        }
                    }
                    cy += 2;
                }
                cx += 2;
            }
        }
    }
}

const FLAG_TO_ASCII: [i32; 8] = [
    0,
    CHAR_SUBP_NE as i32,
    CHAR_SUBP_SW as i32,
    -(CHAR_SUBP_DIAG as i32),
    CHAR_SUBP_SE as i32,
    CHAR_SUBP_E as i32,
    -(CHAR_SUBP_N as i32),
    -(CHAR_SUBP_NW as i32),
];

fn compute_pattern(
    desired: &[Color; 4],
    back: &mut Color,
    front: &mut Option<Color>,
    ascii: &mut i32,
) {
    // adapted from Jeff Lait's code posted on r.g.r.d
    let mut flag = 0;
    /*
		pixels have following flag values :
			X 1
			2 4
		flag indicates which pixels uses foreground color (top left pixel always uses foreground color except if all pixels have the same color)
	*/
    let mut weight: [f32; 2] = [0.0, 0.0];
    // First colour trivial.
    *back = desired[0];

    // Ignore all duplicates...
    let mut i = 1;
    while i < 4 {
        if desired[i].0 != back.0 || desired[i].1 != back.1 || desired[i].2 != back.2 {
            break;
        }
        i += 1;
    }

    // All the same.
    if i == 4 {
        *front = None;
        *ascii = ' ' as i32;
        return;
    }
    weight[0] = i as f32;

    // Found a second colour...
    let mut tmp_front = desired[i];
    weight[1] = 1.0;
    flag |= 1 << (i - 1);
    // remaining colours
    i += 1;
    while i < 4 {
        if desired[i].0 == back.0 && desired[i].1 == back.1 && desired[i].2 == back.2 {
            weight[0] += 1.0;
        } else if desired[i].0 == tmp_front.0
            && desired[i].1 == tmp_front.1
            && desired[i].2 == tmp_front.2
        {
            flag |= 1 << (i - 1);
            weight[1] += 1.0;
        } else {
            // Bah, too many colours,
            // merge the two nearest
            let dist0i = color_dist(&desired[i], back);
            let dist1i = color_dist(&desired[i], &tmp_front);
            let dist01 = color_dist(back, &tmp_front);
            if dist0i < dist1i {
                if dist0i <= dist01 {
                    // merge 0 and i
                    *back = color_blend(&desired[i], back, weight[0] / (1.0 + weight[0]));
                    weight[0] += 1.0;
                } else {
                    // merge 0 and 1
                    *back = color_blend(back, &tmp_front, weight[1] / (weight[0] + weight[1]));
                    weight[0] += 1.0;
                    tmp_front = desired[i];
                    flag = 1 << (i - 1);
                }
            } else {
                if dist1i <= dist01 {
                    // merge 1 and i
                    tmp_front = color_blend(&desired[i], &tmp_front, weight[1] / (1.0 + weight[1]));
                    weight[1] += 1.0;
                    flag |= 1 << (i - 1);
                } else {
                    // merge 0 and 1
                    *back = color_blend(back, &tmp_front, weight[1] / (weight[0] + weight[1]));
                    weight[0] += 1.0;
                    tmp_front = desired[i];
                    flag = 1 << (i - 1);
                }
            }
        }
        i += 1;
    }
    *front = Some(tmp_front);
    *ascii = FLAG_TO_ASCII[flag as usize];
}