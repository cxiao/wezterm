use crate::mux::renderable::Renderable;
use ::window::*;
use portable_pty::PtySize;
use term::VisibleRowIndex;

pub enum ScrollHit {
    Above,
    OnThumb(isize),
    Below,
}

pub struct ThumbInfo {
    /// Offset from the top of the window in pixels
    pub top: usize,
    /// Height of the thumb, in pixels.
    pub height: usize,
    /// Number of rows that correspond to the thumb in rows.
    /// This is normally == viewport height, but in the case
    /// where there are a sufficient number of rows of scrollback
    /// that the pixel height of the thumb would be too small,
    /// we will scale things in order to remain useful.
    pub rows: usize,
}

impl ScrollHit {
    /// Given a mouse y value, determine whether the cursor is above, over
    /// or below the thumb.
    /// If above the thumb, return the offset from the top of the thumb.
    pub fn test(y: isize, render: &dyn Renderable, size: PtySize, dims: &Dimensions) -> Self {
        let info = Self::thumb(render, size, dims);
        if y < info.top as isize {
            Self::Above
        } else if y < (info.top + info.height) as isize {
            Self::OnThumb(y - info.top as isize)
        } else {
            Self::Below
        }
    }

    /// Compute the y-coordinate for the top of the scrollbar thumb
    /// and the height of the thumb and return them.
    pub fn thumb(render: &dyn Renderable, size: PtySize, dims: &Dimensions) -> ThumbInfo {
        let render_dims = render.get_dimensions();
        let scroll_top = render_dims.viewport_offset;
        let scroll_size = render_dims.scrollback_rows;

        let thumb_size = (size.rows as f32 / scroll_size as f32) * dims.pixel_height as f32;

        const MIN_HEIGHT: f32 = 10.;
        let (thumb_size, rows) = if thumb_size < MIN_HEIGHT {
            let scale = MIN_HEIGHT / thumb_size;
            let rows = size.rows as f32 * scale;
            (MIN_HEIGHT, rows as usize)
        } else {
            (thumb_size, size.rows as usize)
        };

        let thumb_top = (1. - (scroll_top + rows as i64) as f32 / scroll_size as f32)
            * size.pixel_height as f32;

        let thumb_size = thumb_size.ceil() as usize;
        let thumb_top = thumb_top.ceil() as usize;

        ThumbInfo {
            top: thumb_top,
            height: thumb_size,
            rows,
        }
    }

    /// Given a new thumb top coordinate (produced by draggin the thumb),
    /// compute the equivalent viewport offset.
    pub fn thumb_top_to_scroll_top(
        thumb_top: usize,
        render: &dyn Renderable,
        size: PtySize,
        dims: &Dimensions,
    ) -> VisibleRowIndex {
        let render_dims = render.get_dimensions();
        let scroll_size = render_dims.scrollback_rows;
        let thumb = Self::thumb(render, size, dims);

        let rows_from_top =
            ((thumb_top as f32 + thumb.height as f32) / thumb.height as f32) * thumb.rows as f32;
        scroll_size.saturating_sub(rows_from_top as usize) as VisibleRowIndex
    }
}
