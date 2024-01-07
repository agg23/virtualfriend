pub struct RenderState {
    /// BG color palette control register
    ///
    /// When set, new background color is not applied until the first 8 rows of the next frame are rendered.
    pub bkcol: u8,
    pub last_bkcol: u8,

    // SB Allows user to watch for the render of a set of 8 rows
    /// The current group of 8 rows of pixels, relative to the top of the image, currently being drawn.
    pub sbcount: u8,
    /// The group of 8 rows of pixels, relative to the top of the image, to compare with while drawing.
    pub sbcmp: u8,
    pub sbout: bool,

    pub object_group_end0: u16,
    pub object_group_end1: u16,
    pub object_group_end2: u16,
    pub object_group_end3: u16,

    pub brightness_control_reg_a: u8,
    pub brightness_control_reg_b: u8,
    pub brightness_control_reg_c: u8,

    // TODO: Implement
    pub led_rest_duration: u8,

    pub background_palette_control0: PaletteRegister,
    pub background_palette_control1: PaletteRegister,
    pub background_palette_control2: PaletteRegister,
    pub background_palette_control3: PaletteRegister,

    pub object_palette_control0: PaletteRegister,
    pub object_palette_control1: PaletteRegister,
    pub object_palette_control2: PaletteRegister,
    pub object_palette_control3: PaletteRegister,

    /// High if drawing from framebuffer 1. Otherwise drawing from framebuffer 0.
    pub drawing_framebuffer_1: bool,
}

impl RenderState {
    pub fn new() -> Self {
        RenderState {
            bkcol: 0,
            last_bkcol: 0,
            sbcount: 0,
            sbcmp: 0,
            sbout: false,
            object_group_end0: 0,
            object_group_end1: 0,
            object_group_end2: 0,
            object_group_end3: 0,
            brightness_control_reg_a: 0,
            brightness_control_reg_b: 0,
            brightness_control_reg_c: 0,
            led_rest_duration: 0,
            background_palette_control0: PaletteRegister::new(),
            background_palette_control1: PaletteRegister::new(),
            background_palette_control2: PaletteRegister::new(),
            background_palette_control3: PaletteRegister::new(),
            object_palette_control0: PaletteRegister::new(),
            object_palette_control1: PaletteRegister::new(),
            object_palette_control2: PaletteRegister::new(),
            object_palette_control3: PaletteRegister::new(),
            drawing_framebuffer_1: false,
        }
    }
}

#[derive(Copy, Clone)]
pub struct PaletteRegister {
    pub character1: u8,
    pub character2: u8,
    pub character3: u8,
}

impl PaletteRegister {
    fn new() -> Self {
        PaletteRegister {
            character1: 0,
            character2: 0,
            character3: 0,
        }
    }

    pub fn get(&self) -> u16 {
        ((self.character3 << 6) | (self.character2 << 4) | (self.character1 << 2)) as u16
    }

    pub fn set(&mut self, value: u16) {
        self.character1 = ((value >> 2) & 0x3) as u8;
        self.character2 = ((value >> 4) & 0x3) as u8;
        self.character3 = ((value >> 6) & 0x3) as u8;
    }
}

pub fn framebuffer_addresses(use_fb_1: bool) -> (usize, usize) {
    let left_framebuffer_address = if use_fb_1 { 0x8000 } else { 0 };
    let right_framebuffer_address = left_framebuffer_address + 0x1_0000;

    (left_framebuffer_address, right_framebuffer_address)
}

pub fn framebuffer_address_at_side(left_eye: bool, use_fb_1: bool) -> usize {
    let (left_framebuffer_address, right_framebuffer_address) = framebuffer_addresses(use_fb_1);

    match left_eye {
        true => left_framebuffer_address,
        false => right_framebuffer_address,
    }
}
