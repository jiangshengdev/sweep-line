use std::collections::BTreeMap;

use crate::geom::point::PointRat;
use crate::geom::segment::SegmentId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    SegmentStart { segment: SegmentId },
    SegmentEnd { segment: SegmentId },
    Intersection { a: SegmentId, b: SegmentId },
}

impl Event {
    pub fn intersection(a: SegmentId, b: SegmentId) -> Self {
        let (a, b) = if a <= b { (a, b) } else { (b, a) };
        Event::Intersection { a, b }
    }

    fn priority(&self) -> u8 {
        match self {
            // 以 `x+ε` 语义为主的批处理：先移除，再处理交点重排，最后插入。
            Event::SegmentEnd { .. } => 0,
            Event::Intersection { .. } => 1,
            Event::SegmentStart { .. } => 2,
        }
    }

    fn ids_for_ordering(&self) -> (SegmentId, SegmentId) {
        match *self {
            Event::SegmentStart { segment } | Event::SegmentEnd { segment } => (segment, segment),
            Event::Intersection { a, b } => (a, b),
        }
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let ord = self.priority().cmp(&other.priority());
        if ord != core::cmp::Ordering::Equal {
            return ord;
        }

        let (a1, b1) = self.ids_for_ordering();
        let (a2, b2) = other.ids_for_ordering();
        match a1.cmp(&a2) {
            core::cmp::Ordering::Equal => b1.cmp(&b2),
            o => o,
        }
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Default)]
pub struct EventQueue {
    by_point: BTreeMap<PointRat, Vec<Event>>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.by_point.is_empty()
    }

    pub fn len_points(&self) -> usize {
        self.by_point.len()
    }

    pub fn push(&mut self, point: PointRat, event: Event) {
        self.by_point.entry(point).or_default().push(event);
    }

    pub fn pop_next_batch(&mut self) -> Option<(PointRat, Vec<Event>)> {
        let (point, mut events) = self.by_point.pop_first()?;
        events.sort();
        Some((point, events))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::fixed::PointI64;
    use crate::geom::point::PointRat;
    use crate::rational::Rational;

    #[test]
    fn batches_by_point_and_orders_points() {
        let mut q = EventQueue::new();
        let p1 = PointRat::from_i64(PointI64 { x: 0, y: 0 });
        let p2 = PointRat::from_i64(PointI64 { x: 1, y: 0 });

        q.push(
            p2,
            Event::SegmentStart {
                segment: SegmentId(2),
            },
        );
        q.push(
            p1,
            Event::SegmentStart {
                segment: SegmentId(1),
            },
        );
        q.push(
            p2,
            Event::SegmentEnd {
                segment: SegmentId(0),
            },
        );

        let (p, e) = q.pop_next_batch().unwrap();
        assert_eq!(p, p1);
        assert_eq!(
            e,
            vec![Event::SegmentStart {
                segment: SegmentId(1)
            }]
        );

        let (p, e) = q.pop_next_batch().unwrap();
        assert_eq!(p, p2);
        assert_eq!(
            e,
            vec![
                Event::SegmentEnd {
                    segment: SegmentId(0)
                },
                Event::SegmentStart {
                    segment: SegmentId(2)
                }
            ]
        );
        assert!(q.is_empty());
    }

    #[test]
    fn orders_events_deterministically_inside_batch() {
        let mut q1 = EventQueue::new();
        let mut q2 = EventQueue::new();
        let p = PointRat {
            x: Rational::new(1, 3),
            y: Rational::new(-2, 7),
        };

        let e1 = Event::SegmentStart {
            segment: SegmentId(10),
        };
        let e2 = Event::SegmentEnd {
            segment: SegmentId(2),
        };
        let e3 = Event::intersection(SegmentId(7), SegmentId(3));

        q1.push(p, e1);
        q1.push(p, e2);
        q1.push(p, e3);

        q2.push(p, e3);
        q2.push(p, e1);
        q2.push(p, e2);

        let (_p, b1) = q1.pop_next_batch().unwrap();
        let (_p, b2) = q2.pop_next_batch().unwrap();
        assert_eq!(b1, b2);

        assert_eq!(
            b1,
            vec![
                Event::SegmentEnd {
                    segment: SegmentId(2)
                },
                Event::Intersection {
                    a: SegmentId(3),
                    b: SegmentId(7)
                },
                Event::SegmentStart {
                    segment: SegmentId(10)
                }
            ]
        );
    }
}
