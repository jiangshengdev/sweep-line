use core::cmp::Ordering;
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

    fn lower_bound_by_y(&self, segments: &Segments, y_min: Rational) -> Option<SegmentId>;

    fn range_by_y(&self, segments: &Segments, y_min: Rational, y_max: Rational) -> Vec<SegmentId> {
        let (y_min, y_max) = if y_min <= y_max { (y_min, y_max) } else { (y_max, y_min) };
        let mut out = Vec::new();

        let mut current = self.lower_bound_by_y(segments, y_min);
        while let Some(id) = current {
            let y = y_at_x(segments.get(id), self.sweep_x());
            if y > y_max {
                break;
            }
            out.push(id);
            current = self.succ(id);
        }

        out
    }

    fn reorder_segments(
        &mut self,
        segments: &Segments,
        ids: &[SegmentId],
    ) -> Result<(), SweepStatusError> {
        let mut ids: Vec<SegmentId> = ids.to_vec();
        ids.sort();
        ids.dedup();

        for id in &ids {
            self.remove(*id)?;
        }
        for id in &ids {
            self.insert(segments, *id)?;
        }
        Ok(())
    }

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

    fn lower_bound_by_y(&self, segments: &Segments, y_min: Rational) -> Option<SegmentId> {
        let sweep_x = self.sweep_x;
        let index = self
            .active
            .partition_point(|id| y_at_x(segments.get(*id), sweep_x) < y_min);
        self.active.get(index).copied()
    }

    fn range_by_y(&self, segments: &Segments, y_min: Rational, y_max: Rational) -> Vec<SegmentId> {
        let (y_min, y_max) = if y_min <= y_max { (y_min, y_max) } else { (y_max, y_min) };
        let sweep_x = self.sweep_x;

        let start = self
            .active
            .partition_point(|id| y_at_x(segments.get(*id), sweep_x) < y_min);
        let mut out = Vec::new();
        for id in &self.active[start..] {
            let y = y_at_x(segments.get(*id), sweep_x);
            if y > y_max {
                break;
            }
            out.push(*id);
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

#[derive(Clone, Copy, Debug, Default)]
struct TreapNode {
    left: Option<SegmentId>,
    right: Option<SegmentId>,
    parent: Option<SegmentId>,
    active: bool,
}

/// 一个确定性的 Treap 实现：节点优先级由 `SegmentId` 固定混洗得到（无 RNG）。
///
/// 说明：
/// - 这是后续高性能实现的基础版本；
/// - `range_by_y` 通过 `lower_bound_by_y + succ` 迭代实现，先保证稳定性与语义正确；
/// - 由于比较器依赖 `sweep_x`，调用方必须遵守“事件点批处理 + `x+ε` 语义”的使用约定。
#[derive(Clone, Debug)]
pub struct TreapSweepStatus {
    sweep_x: Rational,
    root: Option<SegmentId>,
    nodes: Vec<TreapNode>,
    len: usize,
}

impl TreapSweepStatus {
    pub fn new(sweep_x: Rational) -> Self {
        Self {
            sweep_x,
            root: None,
            nodes: Vec::new(),
            len: 0,
        }
    }

    fn ensure_node(&mut self, id: SegmentId) {
        if id.0 < self.nodes.len() {
            return;
        }
        self.nodes.resize(id.0 + 1, TreapNode::default());
    }

    fn is_active(&self, id: SegmentId) -> bool {
        self.nodes.get(id.0).is_some_and(|n| n.active)
    }

    fn priority(id: SegmentId) -> u64 {
        // splitmix64：简单、快、确定性强，适合作为 Treap 的固定优先级来源。
        let mut x = (id.0 as u64).wrapping_add(0x9E37_79B9_7F4A_7C15);
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^ (x >> 31)
    }

    fn priority_key(id: SegmentId) -> (u64, usize) {
        (Self::priority(id), id.0)
    }

    fn higher_priority(a: SegmentId, b: SegmentId) -> bool {
        Self::priority_key(a) > Self::priority_key(b)
    }

    fn set_parent_link(&mut self, parent: Option<SegmentId>, child: Option<SegmentId>, was_left: bool) {
        if let Some(parent) = parent {
            if was_left {
                self.nodes[parent.0].left = child;
            } else {
                self.nodes[parent.0].right = child;
            }
        } else {
            self.root = child;
        }
        if let Some(child) = child {
            self.nodes[child.0].parent = parent;
        }
    }

    fn rotate_left(&mut self, x: SegmentId) {
        let Some(y) = self.nodes[x.0].right else {
            return;
        };
        let parent = self.nodes[x.0].parent;
        let x_was_left = parent.is_some_and(|p| self.nodes[p.0].left == Some(x));

        let beta = self.nodes[y.0].left;

        // y 顶替 x 的位置
        self.set_parent_link(parent, Some(y), x_was_left);

        // x 成为 y 的左子
        self.nodes[y.0].left = Some(x);
        self.nodes[x.0].parent = Some(y);

        // beta 挂到 x 的右子
        self.nodes[x.0].right = beta;
        if let Some(beta) = beta {
            self.nodes[beta.0].parent = Some(x);
        }
    }

    fn rotate_right(&mut self, x: SegmentId) {
        let Some(y) = self.nodes[x.0].left else {
            return;
        };
        let parent = self.nodes[x.0].parent;
        let x_was_left = parent.is_some_and(|p| self.nodes[p.0].left == Some(x));

        let beta = self.nodes[y.0].right;

        // y 顶替 x 的位置
        self.set_parent_link(parent, Some(y), x_was_left);

        // x 成为 y 的右子
        self.nodes[y.0].right = Some(x);
        self.nodes[x.0].parent = Some(y);

        // beta 挂到 x 的左子
        self.nodes[x.0].left = beta;
        if let Some(beta) = beta {
            self.nodes[beta.0].parent = Some(x);
        }
    }

    fn bubble_up(&mut self, id: SegmentId) {
        while let Some(parent) = self.nodes[id.0].parent {
            if !Self::higher_priority(id, parent) {
                break;
            }

            if self.nodes[parent.0].left == Some(id) {
                self.rotate_right(parent);
            } else {
                self.rotate_left(parent);
            }
        }
    }

    fn min_node(&self, mut id: SegmentId) -> SegmentId {
        while let Some(left) = self.nodes[id.0].left {
            id = left;
        }
        id
    }

    fn max_node(&self, mut id: SegmentId) -> SegmentId {
        while let Some(right) = self.nodes[id.0].right {
            id = right;
        }
        id
    }

    fn inorder_collect(&self, out: &mut Vec<SegmentId>) {
        let mut stack: Vec<SegmentId> = Vec::new();
        let mut current = self.root;

        while current.is_some() || !stack.is_empty() {
            while let Some(id) = current {
                stack.push(id);
                current = self.nodes[id.0].left;
            }

            let id = stack.pop().expect("栈不应为空");
            out.push(id);
            current = self.nodes[id.0].right;
        }
    }
}

impl SweepStatus for TreapSweepStatus {
    fn set_sweep_x(&mut self, sweep_x: Rational) {
        self.sweep_x = sweep_x;
    }

    fn sweep_x(&self) -> Rational {
        self.sweep_x
    }

    fn len(&self) -> usize {
        self.len
    }

    fn insert(&mut self, segments: &Segments, id: SegmentId) -> Result<(), SweepStatusError> {
        if segments.get(id).is_vertical() {
            return Err(SweepStatusError::VerticalSegmentNotAllowed);
        }

        self.ensure_node(id);
        if self.nodes[id.0].active {
            return Err(SweepStatusError::DuplicateSegmentId);
        }

        self.nodes[id.0] = TreapNode {
            left: None,
            right: None,
            parent: None,
            active: true,
        };

        let Some(root) = self.root else {
            self.root = Some(id);
            self.len += 1;
            return Ok(());
        };

        let sweep_x = self.sweep_x;
        let mut current = root;
        loop {
            match cmp_segments_at_x_plus_epsilon(segments, current, id, sweep_x) {
                Ordering::Less => {
                    // current < id，往右
                    if let Some(next) = self.nodes[current.0].right {
                        current = next;
                        continue;
                    }
                    self.nodes[current.0].right = Some(id);
                    self.nodes[id.0].parent = Some(current);
                    break;
                }
                Ordering::Greater => {
                    // current > id，往左
                    if let Some(next) = self.nodes[current.0].left {
                        current = next;
                        continue;
                    }
                    self.nodes[current.0].left = Some(id);
                    self.nodes[id.0].parent = Some(current);
                    break;
                }
                Ordering::Equal => {
                    // 由于比较器兜底了 SegmentId，这里理论上不会相等。
                    return Err(SweepStatusError::DuplicateSegmentId);
                }
            }
        }

        self.bubble_up(id);
        self.len += 1;
        Ok(())
    }

    fn remove(&mut self, id: SegmentId) -> Result<(), SweepStatusError> {
        if !self.is_active(id) {
            return Err(SweepStatusError::SegmentNotFound);
        }

        // 通过旋转把目标节点下沉到最多只有一个子节点的位置。
        loop {
            let left = self.nodes[id.0].left;
            let right = self.nodes[id.0].right;
            match (left, right) {
                (Some(left), Some(right)) => {
                    if Self::higher_priority(left, right) {
                        self.rotate_right(id);
                    } else {
                        self.rotate_left(id);
                    }
                }
                _ => break,
            }
        }

        let child = self.nodes[id.0].left.or(self.nodes[id.0].right);
        let parent = self.nodes[id.0].parent;
        let was_left = parent.is_some_and(|p| self.nodes[p.0].left == Some(id));

        self.set_parent_link(parent, child, was_left);

        self.nodes[id.0] = TreapNode::default();
        self.len -= 1;
        Ok(())
    }

    fn pred(&self, id: SegmentId) -> Option<SegmentId> {
        if !self.is_active(id) {
            return None;
        }

        if let Some(left) = self.nodes[id.0].left {
            return Some(self.max_node(left));
        }

        let mut current = id;
        while let Some(parent) = self.nodes[current.0].parent {
            if self.nodes[parent.0].right == Some(current) {
                return Some(parent);
            }
            current = parent;
        }
        None
    }

    fn succ(&self, id: SegmentId) -> Option<SegmentId> {
        if !self.is_active(id) {
            return None;
        }

        if let Some(right) = self.nodes[id.0].right {
            return Some(self.min_node(right));
        }

        let mut current = id;
        while let Some(parent) = self.nodes[current.0].parent {
            if self.nodes[parent.0].left == Some(current) {
                return Some(parent);
            }
            current = parent;
        }
        None
    }

    fn lower_bound_by_y(&self, segments: &Segments, y_min: Rational) -> Option<SegmentId> {
        let mut current = self.root;
        let mut candidate = None;

        while let Some(id) = current {
            let y = y_at_x(segments.get(id), self.sweep_x);
            if y < y_min {
                current = self.nodes[id.0].right;
            } else {
                candidate = Some(id);
                current = self.nodes[id.0].left;
            }
        }

        candidate
    }

    fn snapshot_order(&self) -> Vec<SegmentId> {
        let mut out = Vec::with_capacity(self.len);
        self.inorder_collect(&mut out);
        out
    }

    fn validate_invariants(&self, segments: &Segments) -> Result<(), String> {
        if self.len == 0 {
            if self.root.is_some() {
                return Err("len=0 时 root 应为 None".to_string());
            }
            return Ok(());
        }
        let Some(root) = self.root else {
            return Err("len>0 时 root 不应为 None".to_string());
        };
        if self.nodes[root.0].parent.is_some() {
            return Err("root 的 parent 必须为 None".to_string());
        }

        let mut visited = vec![false; self.nodes.len()];
        let mut stack = vec![root];
        let mut reachable = 0_usize;

        while let Some(id) = stack.pop() {
            if !self.is_active(id) {
                return Err(format!("树中包含未激活节点：{:?}", id));
            }
            if visited[id.0] {
                return Err(format!("检测到环或重复引用：{:?}", id));
            }
            visited[id.0] = true;
            reachable += 1;

            if segments.get(id).is_vertical() {
                return Err("状态结构中不应包含垂直线段".to_string());
            }

            if let Some(left) = self.nodes[id.0].left {
                if self.nodes[left.0].parent != Some(id) {
                    return Err(format!("parent 指针不一致：{:?} <- {:?}", id, left));
                }
                if !Self::higher_priority(id, left) {
                    return Err(format!("Treap 堆性质破坏：{:?} vs {:?}", id, left));
                }
                stack.push(left);
            }
            if let Some(right) = self.nodes[id.0].right {
                if self.nodes[right.0].parent != Some(id) {
                    return Err(format!("parent 指针不一致：{:?} <- {:?}", id, right));
                }
                if !Self::higher_priority(id, right) {
                    return Err(format!("Treap 堆性质破坏：{:?} vs {:?}", id, right));
                }
                stack.push(right);
            }
        }

        if reachable != self.len {
            return Err(format!(
                "可达节点数与 len 不一致：reachable={}, len={}",
                reachable, self.len
            ));
        }

        let ordered = self.snapshot_order();
        if ordered.len() != self.len {
            return Err("snapshot_order 长度与 len 不一致".to_string());
        }
        for i in 1..ordered.len() {
            let prev = ordered[i - 1];
            let curr = ordered[i];
            let ord = cmp_segments_at_x_plus_epsilon(segments, prev, curr, self.sweep_x);
            if ord != Ordering::Less {
                return Err(format!(
                    "BST 顺序不满足严格递增：{:?} 与 {:?}",
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

    #[test]
    fn treap_orders_independent_of_insertion_order_and_matches_vec() {
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
        let mut v = VecSweepStatus::new(sweep_x);
        let mut t1 = TreapSweepStatus::new(sweep_x);
        let mut t2 = TreapSweepStatus::new(sweep_x);

        v.insert(&segments, s1).unwrap();
        v.insert(&segments, s2).unwrap();
        v.insert(&segments, s3).unwrap();

        t1.insert(&segments, s3).unwrap();
        t1.insert(&segments, s1).unwrap();
        t1.insert(&segments, s2).unwrap();

        t2.insert(&segments, s2).unwrap();
        t2.insert(&segments, s3).unwrap();
        t2.insert(&segments, s1).unwrap();

        assert_eq!(v.snapshot_order(), vec![s1, s2, s3]);
        assert_eq!(t1.snapshot_order(), v.snapshot_order());
        assert_eq!(t2.snapshot_order(), v.snapshot_order());
        t1.validate_invariants(&segments).unwrap();
        t2.validate_invariants(&segments).unwrap();
    }

    #[test]
    fn treap_supports_pred_succ_remove_and_range_by_y() {
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

        let mut status = TreapSweepStatus::new(Rational::from_int(5));
        status.insert(&segments, s3).unwrap();
        status.insert(&segments, s1).unwrap();
        status.insert(&segments, s2).unwrap();

        assert_eq!(status.snapshot_order(), vec![s1, s2, s3]);
        assert_eq!(status.pred(s2), Some(s1));
        assert_eq!(status.succ(s2), Some(s3));

        let ids = status.range_by_y(
            &segments,
            Rational::from_int(9),
            Rational::from_int(11),
        );
        assert_eq!(ids, vec![s2, s3]);

        status.remove(s2).unwrap();
        assert_eq!(status.snapshot_order(), vec![s1, s3]);
        status.validate_invariants(&segments).unwrap();
    }

    #[test]
    fn treap_rejects_vertical_segment() {
        let mut segments = Segments::new();
        let vertical = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 0, y: 10 },
            source_index: 0,
        });

        let mut status = TreapSweepStatus::new(Rational::from_int(0));
        assert_eq!(
            status.insert(&segments, vertical).unwrap_err(),
            SweepStatusError::VerticalSegmentNotAllowed
        );
    }
}
