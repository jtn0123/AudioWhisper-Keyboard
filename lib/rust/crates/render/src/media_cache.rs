//! Bounded decoded buffers. Android decodes; native selects frames only for visible items.
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
#[derive(Clone, Debug)]
pub struct Frame {
    pub cpu: Arc<vello_cpu::Pixmap>,
    pub hybrid: Arc<vello_hybrid::Pixmap>,
    pub delay: i64,
}
impl PartialEq for Frame {
    fn eq(&self, o: &Self) -> bool {
        Arc::ptr_eq(&self.cpu, &o.cpu) && self.delay == o.delay
    }
}
impl Frame {
    pub fn argb(w: u16, h: u16, pixels: &[i32], delay: i64) -> Option<Self> {
        if w == 0 || h == 0 || pixels.len() != w as usize * h as usize {
            return None;
        }
        let rgba: Vec<u8> = pixels
            .iter()
            .flat_map(|&p| {
                let p = p as u32;
                [(p >> 16) as u8, (p >> 8) as u8, p as u8, (p >> 24) as u8]
            })
            .collect();
        let data = vello_cpu::peniko::ImageData {
            data: rgba.clone().into(),
            format: vello_cpu::peniko::ImageFormat::Rgba8,
            alpha_type: vello_cpu::peniko::ImageAlphaType::Alpha,
            width: w as u32,
            height: h as u32,
        };
        let vello_common_hybrid::paint::ImageSource::Pixmap(hybrid) =
            vello_common_hybrid::paint::ImageSource::from_peniko_image_data(&data)
        else {
            return None;
        };
        Some(Self {
            cpu: Arc::new(vello_cpu::Pixmap::from_parts(
                rgba,
                w,
                h,
                vello_cpu::PixelMetadata::default(),
            )),
            hybrid,
            delay: delay.max(10_000_000),
        })
    }
    pub fn bytes(&self) -> usize {
        self.cpu.width() as usize * self.cpu.height() as usize * 8
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    pub frames: Vec<Frame>,
    pub frame: usize,
    pub deadline: Option<i64>,
    pub touched: u64,
    pub complete: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MediaCache {
    pub entries: BTreeMap<i64, Animation>,
    pub visible: BTreeSet<i64>,
    pub budget: usize,
    pub per_item: usize,
    pub bytes: usize,
    clock: u64,
}
impl Default for MediaCache {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            visible: BTreeSet::new(),
            budget: 24 * 1024 * 1024,
            per_item: 4 * 1024 * 1024,
            bytes: 0,
            clock: 0,
        }
    }
}
impl MediaCache {
    pub fn upload(&mut self, id: i64, index: usize, frame: Frame, last: bool) -> bool {
        let size = frame.bytes();
        if index >= 64 || size > self.per_item || size > self.budget {
            return false;
        }
        if index == 0 {
            self.remove(id);
        }
        let old = self.entries.get(&id);
        if index != old.map_or(0, |e| e.frames.len())
            || old.is_some_and(|e| {
                e.frames.iter().map(Frame::bytes).sum::<usize>() + size > self.per_item
            })
        {
            return false;
        }
        while self.bytes + size > self.budget {
            let victim = self
                .entries
                .iter()
                .filter(|(k, _)| **k != id && !self.visible.contains(k))
                .min_by_key(|(_, e)| e.touched)
                .map(|(k, _)| *k);
            let Some(v) = victim else {
                return false;
            };
            self.remove(v);
        }
        self.clock += 1;
        let e = self.entries.entry(id).or_insert_with(|| Animation {
            frames: vec![],
            frame: 0,
            deadline: None,
            touched: self.clock,
            complete: false,
        });
        e.frames.push(frame);
        e.complete = last;
        e.touched = self.clock;
        self.bytes += size;
        true
    }
    pub fn remove(&mut self, id: i64) {
        if let Some(e) = self.entries.remove(&id) {
            self.bytes -= e.frames.iter().map(Frame::bytes).sum::<usize>();
        }
    }
    pub fn visibility(&mut self, ids: BTreeSet<i64>, now: i64) {
        self.visible = ids;
        for (&id, e) in &mut self.entries {
            if self.visible.contains(&id) {
                self.clock += 1;
                e.touched = self.clock;
                if e.frames.len() > 1 && e.deadline.is_none() {
                    e.deadline = Some(now + e.frames[e.frame].delay);
                }
            } else {
                e.deadline = None;
            }
        }
    }
    pub fn tick(&mut self, now: i64) -> bool {
        let mut changed = false;
        for e in self.entries.values_mut() {
            if e.deadline.is_some_and(|d| now >= d) {
                e.frame = (e.frame + 1) % e.frames.len();
                e.deadline = Some(now + e.frames[e.frame].delay);
                changed = true;
            }
        }
        changed
    }
    pub fn deadline(&self) -> Option<i64> {
        self.entries.values().filter_map(|e| e.deadline).min()
    }
    pub fn frame(&self, id: i64) -> Option<&Frame> {
        let e = self.entries.get(&id)?;
        e.frames.get(e.frame)
    }
    pub fn requests(&self) -> Vec<i64> {
        self.visible
            .iter()
            .copied()
            .filter(|i| !self.entries.contains_key(i))
            .collect()
    }
}
