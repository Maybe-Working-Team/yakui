use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use fontdue::layout::GlyphRasterConfig;
use fontdue::Font;
use yakui_core::geometry::{URect, UVec2};
use yakui_core::paint::{PaintDom, Texture, TextureFormat};
use yakui_core::TextureId;

#[derive(Clone)]
pub struct TextGlobalState {
    pub glyph_cache: Rc<RefCell<dyn GlyphCache>>,
}

impl TextGlobalState {
    pub fn new<T: GlyphCache + 'static>(glyph_cache: T) -> Self {
        Self {
            glyph_cache: Rc::new(RefCell::new(glyph_cache)) as Rc<RefCell<dyn GlyphCache>>,
        }
    }

    pub fn new_late_binding() -> Self {
        Self {
            glyph_cache: Rc::new(RefCell::new(LateBindingGlyphCache::new()))
                as Rc<RefCell<dyn GlyphCache>>,
        }
    }
}

pub trait GlyphCache {
    fn get_or_insert(
        &mut self,
        paint: &mut PaintDom,
        font: &Font,
        key: GlyphRasterConfig,
    ) -> (TextureId, URect);

    fn texture_size(&self, font: &Font, key: &GlyphRasterConfig) -> UVec2;
}

#[derive(Debug)]
pub struct LateBindingGlyphCache {
    font_atlas_id: Option<TextureId>,
    glyphs: HashMap<GlyphRasterConfig, URect>,
    next_pos: UVec2,
    row_height: u32,
}

impl LateBindingGlyphCache {
    /// This is somewhat a default right now
    const TEXTURE_SIZE: u32 = 4096;

    /// Creates a new LateBindingGlyphCache
    pub fn new() -> Self {
        LateBindingGlyphCache {
            font_atlas_id: None,
            glyphs: HashMap::new(),
            next_pos: UVec2::ONE,
            row_height: 0,
        }
    }

    pub fn font_atlas_id(&mut self, paint: &mut PaintDom, _font: &Font) -> TextureId {
        match self.font_atlas_id {
            Some(v) => v,
            None => {
                let empty_texture = Texture::new(
                    TextureFormat::R8,
                    UVec2::new(Self::TEXTURE_SIZE, Self::TEXTURE_SIZE),
                    vec![0; (Self::TEXTURE_SIZE * Self::TEXTURE_SIZE) as usize],
                );
                let font_atlas_id = paint.add_texture(empty_texture);
                self.font_atlas_id = Some(font_atlas_id);

                font_atlas_id
            }
        }
    }
}

impl GlyphCache for LateBindingGlyphCache {
    fn get_or_insert(
        &mut self,
        paint: &mut PaintDom,
        font: &Font,
        key: GlyphRasterConfig,
    ) -> (TextureId, URect) {
        let font_atlas_id = self.font_atlas_id(paint, font);

        let u_rect = *self.glyphs.entry(key).or_insert_with(|| {
            let font_atlas = paint
                .texture_mut(font_atlas_id)
                .expect("after calling `font_atlas_id` we always have a valid texture to lookup");

            let atlas_size = font_atlas.size();

            let (metrics, bitmap) = font.rasterize_indexed(key.glyph_index, key.px);
            let glyph_size = UVec2::new(metrics.width as u32, metrics.height as u32);

            let glyph_max = self.next_pos + glyph_size;
            let pos = if glyph_max.x < atlas_size.x {
                let pos = self.next_pos;
                self.row_height = self.row_height.max(glyph_size.y + 1);
                pos
            } else {
                let pos = UVec2::new(0, self.row_height);
                self.row_height = 0;
                pos
            };
            self.next_pos = pos + UVec2::new(glyph_size.x + 1, 0);

            blit(pos, &bitmap, glyph_size, font_atlas.data_mut(), atlas_size);

            // let the painter know that we modified the texture
            let yak_tex = match font_atlas_id {
                TextureId::Yak(v) => v,
                _ => panic!(),
            };
            paint.modify_texture(yak_tex);

            URect::from_pos_size(pos, glyph_size)
        });

        (font_atlas_id, u_rect)
    }

    fn texture_size(&self, _font: &Font, _key: &GlyphRasterConfig) -> UVec2 {
        UVec2::new(Self::TEXTURE_SIZE, Self::TEXTURE_SIZE)
    }
}

fn get_pixel(data: &[u8], size: UVec2, pos: UVec2) -> u8 {
    let offset = pos.y * size.x + pos.x;
    data[offset as usize]
}

fn set_pixel(data: &mut [u8], size: UVec2, pos: UVec2, value: u8) {
    let offset = pos.y * size.x + pos.x;
    data[offset as usize] = value;
}

pub fn blit(
    dest_pos: UVec2,
    source_data: &[u8],
    source_size: UVec2,
    dest_data: &mut [u8],
    dest_size: UVec2,
) {
    for h in 0..source_size.y {
        for w in 0..source_size.x {
            let pos = UVec2::new(dest_pos.x + w, dest_pos.y + h);

            let value = get_pixel(source_data, source_size, UVec2::new(w, h));
            set_pixel(dest_data, dest_size, pos, value);
        }
    }
}
