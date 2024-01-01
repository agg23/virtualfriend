use crate::util::sign_extend_16;

pub struct Object {
    /// The signed horizontal coordinate of the left edge of the object from the left edge of the image.
    pub display_pointer_x: i16,
    /// The vertical coordinate of the top edge of the object from the top edge of the image.
    ///
    /// This value is psuedo-signed. See docs.
    pub display_pointer_y: u8,

    /// If set, the object will be drawn to the left image.
    pub render_to_left_display: bool,
    /// If set, the object will be drawn to the right image.
    pub render_to_right_display: bool,

    /// The signed parallax offset applied to the horizontal coordinate.
    pub parallax: i16,

    pub palette: u8,

    pub horizontal_flip: bool,
    pub vertical_flip: bool,

    pub character_index: u16,
}

impl Object {
    pub fn parse(halfwords: &[u16]) -> Self {
        assert!(halfwords.len() == 4);

        let display_pointer_x = sign_extend_16(halfwords[0] & 0x3FF, 10);

        let render_to_left_display = halfwords[1] & 0x8000 != 0;
        let render_to_right_display = halfwords[1] & 0x4000 != 0;
        let parallax = sign_extend_16(halfwords[1] & 0x3FF, 10);

        // TODO: I don't understand how the signed -8 value works
        let display_pointer_y = (halfwords[2] & 0xFF) as u8;

        let palette = ((halfwords[3] >> 14) & 0x3) as u8;

        let horizontal_flip = halfwords[3] & 0x2000 != 0;
        let vertical_flip = halfwords[3] & 0x1000 != 0;

        let character_index = halfwords[3] & 0x7FF;

        Object {
            display_pointer_x,
            display_pointer_y,
            render_to_left_display,
            render_to_right_display,
            parallax,
            palette,
            horizontal_flip,
            vertical_flip,
            character_index,
        }
    }
}
