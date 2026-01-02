use core::fmt;

use crate::geom::segment::{SegmentId, Segments};
use crate::rational::Rational;
use crate::sweep::segment_order::{cmp_segments_at_x_plus_epsilon, y_at_x};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SweepStatusError {
    VerticalSegmentNotAllowed,
    DuplicateSegmentId,
    SegmentNotFound,
}

impl fmt::Display for SweepStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SweepStatusError::VerticalSegmentNotAllowed => {
                write!(f, "垂直线段不允许插入状态结构")
            }
            SweepStatusError::DuplicateSegmentId => write!(f, "重复的 SegmentId"),
            SweepStatusError::SegmentNotFound => write!(f, "状态结构中不存在该线段"),
        }
    }
}

pub trait SweepStatus {
    fn set_sweep_x(&mut self, sweep_x: Rational);
    fn sweep_x(&self) -> Rational;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn insert(&mut self, segments: &Segments, id: SegmentId) -> Result<(), SweepStatusError>;
    fn remove(&mut self, id: SegmentId) -> Result<(), SweepStatusError>;

    fn pred(&self, id: SegmentId) -> Option<SegmentId>;
    fn succ(&self, id: SegmentId) -> Option<SegmentId>;

    fn range_by_y(&self, segments: &Segments, y_min: Rational, y_max: Rational) -> Vec<SegmentId>;

    fn snapshot_order(&self) -> Vec<SegmentId>;

    fn validate_invariants(&self, segments: &Segments) -> Result<(), String>;
}

/// 一个确定性的基准实现：用有序 `Vec<SegmentId>` 表示活动集合。
///
/// 目的：
/// - 先验证接口语义与稳定性；
/// - 作为将来 Treap 实现的对照与回归测试基线。
#[derive(Clone, Debug)]
pub struct VecSweepStatus {
    sweep_x: Rational,
    active: Vec<SegmentId>,
}

impl VecSweepStatus {
    pub fn new(sweep_x: Rational) -> Self {
        Self {
            sweep_x,
            active: Vec::new(),
        }
    }

    fn position(&self, id: SegmentId) -> Option<usize> {
        self.active.iter().position(|v| *v == id)
    }
}

impl SweepStatus for VecSweepStatus {
    fn set_sweep_x(&mut self, sweep_x: Rational) {
        self.sweep_x = sweep_x;
    }

    fn sweep_x(&self) -> Rational {
        self.sweep_x
    }

    fn len(&self) -> usize {
        self.active.len()
    }

    fn insert(&mut self, segments: &Segments, id: SegmentId) -> Result<(), SweepStatusError> {
        if segments.get(id).is_vertical() {
            return Err(SweepStatusError::VerticalSegmentNotAllowed);
        }

        let sweep_x = self.sweep_x;
        match self.active.binary_search_by(|probe| {
            cmp_segments_at_x_plus_epsilon(segments, *probe, id, sweep_x)
        }) {
            Ok(_) => Err(SweepStatusError::DuplicateSegmentId),
            Err(index) => {
                self.active.insert(index, id);
                Ok(())
            }
        }
    }

    fn remove(&mut self, id: SegmentId) -> Result<(), SweepStatusError> {
        let Some(index) = self.position(id) else {
            return Err(SweepStatusError::SegmentNotFound);
        };
        self.active.remove(index);
        Ok(())
    }

    fn pred(&self, id: SegmentId) -> Option<SegmentId> {
        let index = self.position(id)?;
        index.checked_sub(1).map(|i| self.active[i])
    }

    fn succ(&self, id: SegmentId) -> Option<SegmentId> {
        let index = self.position(id)?;
        self.active.get(index + 1).copied()
    }

    fn range_by_y(&self, segments: &Segments, y_min: Rational, y_max: Rational) -> Vec<SegmentId> {
        let (y_min, y_max) = if y_min <= y_max { (y_min, y_max) } else { (y_max, y_min) };
        let sweep_x = self.sweep_x;

        let mut out = Vec::new();
        for id in &self.active {
            let segment = segments.get(*id);
            let y = y_at_x(segment, sweep_x);
            if y >= y_min && y <= y_max {
                out.push(*id);
            }
        }
        out
    }

    fn snapshot_order(&self) -> Vec<SegmentId> {
        self.active.clone()
    }

    fn validate_invariants(&self, segments: &Segments) -> Result<(), String> {
        for id in &self.active {
            if segments.get(*id).is_vertical() {
                return Err("状态结构中不应包含垂直线段".to_string());
            }
        }

        for i in 1..self.active.len() {
            let prev = self.active[i - 1];
            let curr = self.active[i];
            let ord = cmp_segments_at_x_plus_epsilon(segments, prev, curr, self.sweep_x);
            if ord != core::cmp::Ordering::Less {
                return Err(format!(
                    "状态结构顺序不满足严格递增：{:?} 与 {:?}",
                    prev, curr
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::fixed::PointI64;
    use crate::geom::segment::{Segment, Segments};

    #[test]
    fn orders_independent_of_insertion_order() {
        let mut segments = Segments::new();
        let s1 = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 0,
        });
        let s2 = segments.push(Segment {
            a: PointI64 { x: 0, y: 10 },
            b: PointI64 { x: 10, y: 10 },
            source_index: 1,
        });
        let s3 = segments.push(Segment {
            a: PointI64 { x: 0, y: 5 },
            b: PointI64 { x: 10, y: 15 },
            source_index: 2,
        });

        let sweep_x = Rational::from_int(5);
        let mut a = VecSweepStatus::new(sweep_x);
        let mut b = VecSweepStatus::new(sweep_x);

        a.insert(&segments, s1).unwrap();
        a.insert(&segments, s2).unwrap();
        a.insert(&segments, s3).unwrap();

        b.insert(&segments, s3).unwrap();
        b.insert(&segments, s1).unwrap();
        b.insert(&segments, s2).unwrap();

        assert_eq!(a.snapshot_order(), vec![s1, s2, s3]);
        assert_eq!(a.snapshot_order(), b.snapshot_order());
        a.validate_invariants(&segments).unwrap();
        b.validate_invariants(&segments).unwrap();
    }

    #[test]
    fn supports_pred_succ_and_remove() {
        let mut segments = Segments::new();
        let s1 = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 0,
        });
        let s2 = segments.push(Segment {
            a: PointI64 { x: 0, y: 10 },
            b: PointI64 { x: 10, y: 10 },
            source_index: 1,
        });
        let s3 = segments.push(Segment {
            a: PointI64 { x: 0, y: 5 },
            b: PointI64 { x: 10, y: 15 },
            source_index: 2,
        });

        let mut status = VecSweepStatus::new(Rational::from_int(5));
        status.insert(&segments, s1).unwrap();
        status.insert(&segments, s2).unwrap();
        status.insert(&segments, s3).unwrap();

        assert_eq!(status.pred(s2), Some(s1));
        assert_eq!(status.succ(s2), Some(s3));
        assert_eq!(status.pred(s1), None);
        assert_eq!(status.succ(s3), None);

        status.remove(s2).unwrap();
        assert_eq!(status.snapshot_order(), vec![s1, s3]);
        assert_eq!(status.pred(s3), Some(s1));
        status.validate_invariants(&segments).unwrap();
    }

    #[test]
    fn range_by_y_returns_stable_order() {
        let mut segments = Segments::new();
        let s1 = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 0,
        });
        let s2 = segments.push(Segment {
            a: PointI64 { x: 0, y: 10 },
            b: PointI64 { x: 10, y: 10 },
            source_index: 1,
        });
        let s3 = segments.push(Segment {
            a: PointI64 { x: 0, y: 5 },
            b: PointI64 { x: 10, y: 15 },
            source_index: 2,
        });

        let mut status = VecSweepStatus::new(Rational::from_int(5));
        status.insert(&segments, s3).unwrap();
        status.insert(&segments, s1).unwrap();
        status.insert(&segments, s2).unwrap();

        let ids = status.range_by_y(
            &segments,
            Rational::from_int(9),
            Rational::from_int(11),
        );
        assert_eq!(ids, vec![s2, s3]);
    }

    #[test]
    fn rejects_vertical_segment() {
        let mut segments = Segments::new();
        let vertical = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 0, y: 10 },
            source_index: 0,
        });

        let mut status = VecSweepStatus::new(Rational::from_int(0));
        assert_eq!(
            status.insert(&segments, vertical).unwrap_err(),
            SweepStatusError::VerticalSegmentNotAllowed
        );
    }
}

