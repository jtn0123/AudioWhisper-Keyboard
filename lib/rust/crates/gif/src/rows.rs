//! How the GIF panel's values cross to Kotlin (`wire::gif`): a GIF as a [`row`]
//! of six fields, flags, numbers, and the fields of a call.

use wire::flag;
use wire::gif::row;

use crate::giphy::Gif;

/// `text` as UTF-16 units.
pub fn text(value: impl ToString) -> Vec<u16> {
    value.to_string().encode_utf16().collect()
}

/// The fields of one call, read in order.
#[derive(Debug)]
pub struct Fields<'a> {
    fields: &'a [Vec<u16>],
    next: usize,
}

impl<'a> Fields<'a> {
    pub fn new(fields: &'a [Vec<u16>]) -> Self {
        Fields { fields, next: 0 }
    }

    /// The next field.
    pub fn take(&mut self) -> Result<&'a [u16], String> {
        let field = self.fields.get(self.next).ok_or("a GIF field is missing")?;
        self.next += 1;
        Ok(field)
    }

    /// The next field, a flag.
    pub fn flag(&mut self) -> Result<bool, String> {
        self.take().map(flag::decode)
    }

    /// The next field, a number.
    pub fn int(&mut self) -> Result<i32, String> {
        let field = self.take()?;
        String::from_utf16_lossy(field)
            .parse()
            .map_err(|_| "a GIF field isn't a number".to_owned())
    }

    /// The next six fields, a GIF.
    pub fn gif(&mut self) -> Result<Gif, String> {
        let id = self.take()?.to_vec();
        let title = self.take()?.to_vec();
        let preview_url = self.take()?.to_vec();
        let ratio = f32::from_bits(self.int()? as u32);
        let send_url = self.take()?.to_vec();
        let page_url = self.take()?.to_vec();
        Ok(Gif {
            id,
            title,
            preview_url,
            aspect_ratio: ratio,
            send_url,
            page_url,
        })
    }

    /// The rest, GIFs of six fields each.
    pub fn gifs(&mut self) -> Result<Vec<Gif>, String> {
        let rest = self.fields.len() - self.next;
        if !rest.is_multiple_of(row::SIZE) {
            return Err("GIF rows are cut short".into());
        }
        (0..rest / row::SIZE).map(|_| self.gif()).collect()
    }
}

/// `gif` as it crosses: its six fields, the ratio as its bits so it comes back exactly.
pub fn gif(gif: &Gif) -> [Vec<u16>; row::SIZE] {
    [
        gif.id.clone(),
        gif.title.clone(),
        gif.preview_url.clone(),
        text(gif.aspect_ratio.to_bits() as i32),
        gif.send_url.clone(),
        gif.page_url.clone(),
    ]
}

/// `gifs`, one row after another.
pub fn gifs(gifs: &[Gif]) -> impl Iterator<Item = Vec<u16>> + '_ {
    gifs.iter().flat_map(gif)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(ratio: f32) -> Gif {
        Gif {
            id: text("a"),
            title: text("A"),
            preview_url: text("p"),
            aspect_ratio: ratio,
            send_url: text("s"),
            page_url: text(""),
        }
    }

    #[test]
    fn a_gif_crosses_and_comes_back_exactly() {
        for ratio in [1.5, 0.1, f32::MIN_POSITIVE, f32::INFINITY, -0.0] {
            let rows: Vec<_> = gifs(&[sample(ratio), sample(2.0)]).collect();
            let back = Fields::new(&rows).gifs().unwrap();
            assert_eq!(back[0].aspect_ratio.to_bits(), ratio.to_bits());
            assert_eq!(back[1], sample(2.0));
        }
        let nan = f32::from_bits(0x7fc0_0001);
        let rows: Vec<_> = gifs(&[sample(nan)]).collect();
        let back = Fields::new(&rows).gifs().unwrap();
        assert_eq!(back[0].aspect_ratio.to_bits(), nan.to_bits());
    }

    #[test]
    fn short_or_wrong_fields_are_refused() {
        let rows: Vec<_> = gifs(&[sample(1.0)]).take(5).collect();
        assert!(Fields::new(&rows).gifs().is_err());
        let rows = vec![text("x")];
        assert!(Fields::new(&rows).int().is_err());
        assert!(Fields::new(&[]).take().is_err());
        let mut fields = Fields::new(&rows);
        fields.take().unwrap();
        assert!(fields.flag().is_err());
    }
}
