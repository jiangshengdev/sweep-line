use crate::geom::fixed::PointI64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SegmentId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SegmentKey {
    pub a: PointI64,
    pub b: PointI64,
}

impl SegmentKey {
    pub fn new(mut a: PointI64, mut b: PointI64) -> Self {
        if b < a {
            core::mem::swap(&mut a, &mut b);
        }
        Self { a, b }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segment {
    pub a: PointI64,
    pub b: PointI64,
    pub source_index: usize,
}

impl Segment {
    pub fn key(&self) -> SegmentKey {
        SegmentKey::new(self.a, self.b)
    }

    pub fn is_vertical(&self) -> bool {
        self.a.x == self.b.x
    }
}

#[derive(Clone, Debug, Default)]
pub struct Segments {
    segments: Vec<Segment>,
}

impl Segments {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn get(&self, id: SegmentId) -> &Segment {
        &self.segments[id.0]
    }

    pub fn iter(&self) -> core::slice::Iter<'_, Segment> {
        self.segments.iter()
    }

    /// 追加一条（已规范化、非零长度的）线段并返回其 `SegmentId`。
    ///
    /// 说明：
    /// - 调用方需保证端点已按 `(x,y)` 字典序规范化（非垂直时满足 `a.x < b.x`）。
    /// - 调用方需保证不是零长度线段（`a != b`）。
    pub fn push(&mut self, segment: Segment) -> SegmentId {
        let id = SegmentId(self.segments.len());
        self.segments.push(segment);
        id
    }
}

impl core::ops::Index<SegmentId> for Segments {
    type Output = Segment;

    fn index(&self, index: SegmentId) -> &Self::Output {
        self.get(index)
    }
}
