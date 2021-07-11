use glam::i32;
use glam::IVec2;
use std::collections::VecDeque;
use std::iter::Iterator;
use strum_macros::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, EnumIter)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

impl Direction {
    fn axis(&self) -> Axis {
        match self {
            Direction::Right => Axis::Horizontal,
            Direction::Left => Axis::Horizontal,
            Direction::Up => Axis::Vertical,
            Direction::Down => Axis::Vertical,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Axis {
    Horizontal,
    Vertical,
}

fn move_position(pos: IVec2, dir: Direction, len: i32) -> IVec2 {
    let delta = match dir {
        Direction::Right => i32::ivec2(len, 0),
        Direction::Left => i32::ivec2(-len, 0),
        Direction::Up => i32::ivec2(0, -len),
        Direction::Down => i32::ivec2(0, len),
    };
    pos + delta
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ChainedLineSegment {
    pub dir: Direction,
    pub len: usize,
}

macro_rules! seg {
    ($dir:expr, $len:expr) => {
        ChainedLineSegment {
            dir: $dir,
            len: $len,
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct RectilinearLine {
    pub start: IVec2,
    pub segments: VecDeque<ChainedLineSegment>,
}

impl RectilinearLine {
    /// Returns the total length of the line, which is 1 for the head plus the
    /// lengths of all the line segments.
    pub fn len(&self) -> usize {
        self.segments
            .iter()
            .fold(1, |sum, &segment| sum + segment.len)
    }

    /// The direction of the line is the direction of the last line segment.
    /// If segments are empty, returns `None`.
    pub fn dir(&self) -> Option<Direction> {
        let segments = &self.segments;
        if segments.len() > 1 {
            Some(segments[segments.len() - 1].dir)
        } else {
            None
        }
    }

    pub fn shrink_tail(&mut self) {
        let tail = &mut self.segments[0];
        self.start += match tail.dir {
            Direction::Right => i32::ivec2(1, 0),
            Direction::Left => i32::ivec2(-1, 0),
            Direction::Up => i32::ivec2(0, -1),
            Direction::Down => i32::ivec2(0, 1),
        };

        if tail.len > 1 {
            tail.len -= 1;
        } else {
            self.segments.pop_front();
        }
    }

    pub fn extend_head(&mut self, dir: Direction) {
        let last_index = self.segments.len() - 1;
        let last_segment = &mut self.segments[last_index];

        if last_segment.dir == dir {
            last_segment.len += 1;
        } else {
            self.segments.push_back(ChainedLineSegment { dir, len: 1 });
        }
    }

    pub fn move_forward(&mut self, dir: Direction) {
        self.shrink_tail();
        self.extend_head(dir);
    }

    /// Checks if any segment overlaps itself anywhere except in the points
    /// where the line segments join together to make up the line
    pub fn is_self_overlapping(&self) -> bool {
        enum Overlap {
            Corner,
            Line,
        }

        // Checks if the point x is within the closed range [x0, x1]
        let intersects = |x: i32, x0: i32, x1: i32| {
            if [x0, x1].contains(&x) {
                Some(Overlap::Corner)
            } else if ((x0 + 1)..x1).contains(&x) {
                Some(Overlap::Line)
            } else {
                None
            }
        };

        // checks if two lines overlap
        let lines_overlap = |(a_x0, a_x1): (i32, i32), (b_x0, b_x1): (i32, i32)| {
            // can't overlap with self
            if (a_x0, a_x1) == (b_x0, b_x1) {
                return false;
            }

            // let a_x0 = seg_a.pos.x;
            // let a_x1 = seg_a.pos.x + seg_a.len as i32;
            // let b_x0 = seg_b.pos.x;
            // let b_x1 = seg_b.pos.x + seg_b.len as i32;

            /* Check if intersects from first side */
            if let Some(_) = intersects(b_x0, a_x0, a_x1) {
                return true;
            }

            /* Check if intersects from other side */
            if let Some(_) = intersects(b_x1, a_x0, a_x1) {
                return true;
            }

            false
        };

        /* Check vertical segments against horizontal */
        for v_seg in self.vertical_segments() {
            for h_seg in self.horizontal_segments() {
                /* Horizontal segment */
                let h_x0 = h_seg.pos.x;
                let h_x1 = h_seg.pos.x + h_seg.len as i32;
                let h_y = h_seg.pos.y;

                /* Vertical segment */
                let v_y0 = v_seg.pos.y;
                let v_y1 = v_seg.pos.y + v_seg.len as i32;
                let v_x = v_seg.pos.x;

                let overlaps_horizontally = match intersects(v_x, h_x0, h_x1) {
                    Some(Overlap::Corner) => ![v_y0, v_y1].contains(&h_y),
                    Some(Overlap::Line) => true,
                    None => false,
                };

                let overlaps_vertically = match intersects(h_y, v_y0, v_y1) {
                    Some(Overlap::Corner) => ![h_x0, h_x1].contains(&v_x),
                    Some(Overlap::Line) => true,
                    None => false,
                };

                if overlaps_horizontally && overlaps_vertically {
                    return true;
                }
            }
        }

        /* Check horizontal segments against horizontal */
        for (n, seg_a) in self.horizontal_segments().enumerate() {
            let remaining_segments = self.horizontal_segments().skip(n);
            for seg_b in remaining_segments {
                if seg_a.pos.y == seg_b.pos.y {
                    if lines_overlap(
                        (seg_a.pos.x, seg_a.pos.x + seg_a.len as i32),
                        (seg_b.pos.x, seg_b.pos.x + seg_b.len as i32),
                    ) {
                        return true;
                    }
                }
            }
        }

        /* Check vertical segments against vertical */
        for (n, seg_a) in self.vertical_segments().enumerate() {
            let remaining_segments = self.vertical_segments().skip(n);
            for seg_b in remaining_segments {
                if seg_a.pos.x == seg_b.pos.x {
                    if lines_overlap(
                        (seg_a.pos.y, seg_a.pos.y + seg_a.len as i32),
                        (seg_b.pos.y, seg_b.pos.y + seg_b.len as i32),
                    ) {
                        return true;
                    }
                }
            }
        }

        // couldn't find any overlapping segments
        false
    }

    /// Iterate over each horizontal line segment
    fn horizontal_segments(&self) -> StraightLineSegmentIter {
        StraightLineSegmentIter {
            segments: &self.segments,
            index: 0,
            pos: self.start,
            axis: Axis::Horizontal,
        }
    }

    /// Iterate over each vertical line segment
    fn vertical_segments(&self) -> StraightLineSegmentIter {
        StraightLineSegmentIter {
            segments: &self.segments,
            index: 0,
            pos: self.start,
            axis: Axis::Vertical,
        }
    }
}

/// A line segment that extends out from its position either horizontally to
/// the right or vertically down depending on the `axis` value.
#[derive(Debug, Copy, Clone, PartialEq)]
struct StraightLineSegment {
    pos: IVec2,
    len: usize,
    axis: Axis,
}

/// Iterator used for either iterating over all horizontal or all vertical
/// segments in a RectilinearLine
struct StraightLineSegmentIter<'a> {
    segments: &'a VecDeque<ChainedLineSegment>,
    index: usize,
    pos: IVec2,
    axis: Axis,
}

impl<'a> Iterator for StraightLineSegmentIter<'a> {
    type Item = StraightLineSegment;
    fn next(&mut self) -> Option<Self::Item> {
        // check if iterator exhausted
        if self.index == self.segments.len() {
            return None;
        }

        // search for next segment parallel to our axis
        for index in self.index..self.segments.len() {
            let segment = &self.segments[index];
            let cur_pos = self.pos;
            let seg_len = segment.len as i32;

            // move position along line
            self.pos = move_position(self.pos, segment.dir, seg_len);

            // update index to next segment
            self.index += 1;

            // if segment axis matches ours, return it
            if segment.dir.axis() == self.axis {
                let seg_pos = match segment.dir {
                    Direction::Right => cur_pos,
                    Direction::Left => cur_pos - seg_len * i32::ivec2(1, 0),
                    Direction::Up => cur_pos - seg_len * i32::ivec2(0, 1),
                    Direction::Down => cur_pos,
                };

                return Some(StraightLineSegment {
                    pos: seg_pos,
                    len: segment.len,
                    axis: self.axis,
                });
            }
            // else, keep looking
        }

        // we didn't find any segments, return None
        None
    }
}

#[cfg(test)]
mod line_tests {
    use super::*;
    use strum::IntoEnumIterator;

    /// Creates a straight line with total length `len` in the direction `dir`
    fn straight_line(dir: Direction, len: usize) -> RectilinearLine {
        RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::from(vec![seg!(dir, len - 1)]),
        }
    }

    /// An empty line in the origin
    fn empty_line() -> RectilinearLine {
        RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::new(),
        }
    }

    #[test]
    fn length_of_empty_line_is_one() {
        assert_eq!(empty_line().len(), 1);
    }

    #[test]
    fn length_of_line_is_sum_of_segments() {
        let line = RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::from(vec![seg!(Direction::Right, 1), seg!(Direction::Up, 2)]),
        };
        assert_eq!(line.len(), 4); // 1 + 1 + 2
    }

    #[test]
    fn direction_of_empty_line_is_none() {
        assert_eq!(empty_line().dir(), None);
    }

    #[test]
    fn direction_of_line_is_direction_of_last_segment() {
        // check each case
        for dir in Direction::iter() {
            let line = RectilinearLine {
                start: i32::ivec2(0, 0),
                segments: VecDeque::from(vec![seg!(Direction::Left, 2), seg!(dir, 2)]),
            };
            assert_eq!(line.dir(), Some(dir));
        }
    }

    #[test]
    fn shrinking_moves_end_position_in_direction_of_first_segment() {
        // cases to check (right, left, up, down)
        let poss = vec![(1, 0), (-1, 0), (0, -1), (0, 1)];

        // check each case
        for (pos, dir) in poss.into_iter().zip(Direction::iter()) {
            let mut line = straight_line(dir, 2);
            line.shrink_tail();

            assert_eq!(
                line.start,
                i32::ivec2(pos.0, pos.1),
                "shrink failed for direction \"{:?}\"",
                dir
            );
        }
    }

    #[test]
    fn shrinking_shortens_length_of_first_segment_if_length_two_or_more() {
        for seg_len in 2..100 {
            let mut line = straight_line(Direction::Right, 1 + seg_len);
            line.shrink_tail();
            assert_eq!(
                line.segments[0].len,
                seg_len - 1,
                "shrink failed for tail segment length={}",
                seg_len
            )
        }
    }

    #[test]
    fn shrinking_removes_first_segment_if_length_less_than_two() {
        for len in 0..2 {
            let mut line = RectilinearLine {
                start: i32::ivec2(0, 0),
                segments: VecDeque::from(vec![
                    seg!(Direction::Right, len),
                    seg!(Direction::Down, 5),
                    seg!(Direction::Left, 3),
                ]),
            };
            let expected_segments =
                VecDeque::from(vec![seg!(Direction::Down, 5), seg!(Direction::Left, 3)]);

            line.shrink_tail();

            assert_eq!(
                line.segments, expected_segments,
                "shrink failed to remove tail segment when length={}",
                len
            )
        }
    }

    #[test]
    fn extending_line_in_line_direction_increases_last_segment_length() {
        for dir in Direction::iter() {
            for len in 0..100 {
                let mut line = RectilinearLine {
                    start: i32::ivec2(0, 0),
                    segments: VecDeque::from(vec![seg!(Direction::Right, 2), seg!(dir, len)]),
                };

                line.extend_head(dir);

                let segments = &line.segments;
                assert_eq!(segments[segments.len() - 1].len, len + 1)
            }
        }
    }

    #[test]
    fn extending_line_in_new_direction_adds_new_segment_in_that_direction() {
        for (dir1, dir2) in Direction::iter().zip(Direction::iter()) {
            if dir1 == dir2 {
                // precondition is that directions differ
                continue;
            }

            let mut line = RectilinearLine {
                start: i32::ivec2(0, 0),
                segments: VecDeque::from(vec![seg!(dir1, 2)]),
            };

            line.extend_head(dir2);

            assert_eq!(line.segments[line.segments.len() - 1], seg!(dir2, 1));
        }
    }

    #[test]
    fn extending_and_shrinking_keeps_length_unchanged() {
        let len = 3;
        let mut line = straight_line(Direction::Right, 3);
        line.extend_head(Direction::Right);
        line.shrink_tail();
        assert_eq!(line.len(), len)
    }

    #[test]
    fn line_without_overlapping_segments_is_not_self_overlapping() {
        //      ^--->
        // o->  |
        //   |  |
        //   v-->
        let line = RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::from(vec![
                seg!(Direction::Right, 2),
                seg!(Direction::Down, 2),
                seg!(Direction::Right, 3),
                seg!(Direction::Up, 3),
                seg!(Direction::Right, 4),
            ]),
        };
        assert_eq!(line.is_self_overlapping(), false);
    }

    #[test]
    fn line_with_overlapping_segments_is_self_overlapping() {
        //     <--^
        //     |  |
        //  o--+-->
        //     |
        //     v
        let line = RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::from(vec![
                seg!(Direction::Right, 6),
                seg!(Direction::Up, 2),
                seg!(Direction::Left, 2),
                seg!(Direction::Down, 4),
            ]),
        };
        assert_eq!(line.is_self_overlapping(), true);
    }

    #[test]
    fn line_tip_can_self_overlap_to_the_right_with_body() {
        //    o
        //    |
        // ^-->
        // |  |
        // <--v
        let line = RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::from(vec![
                seg!(Direction::Down, 4),
                seg!(Direction::Left, 3),
                seg!(Direction::Up, 2),
                seg!(Direction::Right, 3),
            ]),
        };
        assert_eq!(line.is_self_overlapping(), true);
    }

    #[test]
    fn line_tip_can_self_overlap_to_the_left_with_body() {
        // o
        // |
        // <--^
        // |  |
        // v-->
        let line = RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::from(vec![
                seg!(Direction::Down, 4),
                seg!(Direction::Right, 3),
                seg!(Direction::Up, 2),
                seg!(Direction::Left, 3),
            ]),
        };
        assert_eq!(line.is_self_overlapping(), true);
    }

    #[test]
    fn horizontal_segment_can_overlap_another_to_the_left() {
        //
        // o---X---^
        //     |   |
        //     V--->
        //
        let line = RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::from(vec![
                seg![Direction::Right, 4],
                seg![Direction::Down, 2],
                seg![Direction::Right, 4],
                seg![Direction::Up, 2],
                seg![Direction::Left, 4],
            ]),
        };
        assert_eq!(line.is_self_overlapping(), true);
    }

    #[test]
    fn horizontal_segment_can_overlap_another_to_the_right() {
        //
        // ^---X---o
        // |   |
        // <---v
        //
        let line = RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::from(vec![
                seg![Direction::Left, 4],
                seg![Direction::Down, 2],
                seg![Direction::Left, 4],
                seg![Direction::Up, 2],
                seg![Direction::Right, 4],
            ]),
        };
        assert_eq!(line.is_self_overlapping(), true);
    }

    #[test]
    fn vertical_segment_can_overlap_another_downwards() {
        //
        // <---^
        // |   |
        // X--->
        // |
        // o
        //
        let line = RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::from(vec![
                seg![Direction::Up, 2],
                seg![Direction::Right, 4],
                seg![Direction::Up, 2],
                seg![Direction::Left, 4],
                seg![Direction::Down, 2],
            ]),
        };
        assert_eq!(line.is_self_overlapping(), true);
    }

    #[test]
    fn vertical_segment_can_overlap_another_upwards() {
        //
        // o
        // |
        // X--->
        // |   |
        // <---v
        //
        let line = RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::from(vec![
                seg![Direction::Down, 2],
                seg![Direction::Right, 4],
                seg![Direction::Down, 2],
                seg![Direction::Left, 4],
                seg![Direction::Up, 2],
            ]),
        };
        assert_eq!(line.is_self_overlapping(), true);
    }

    #[test]
    // This test is just to document the assumption made that a rectilinear
    // line conists of segments with alternating axis alignment.
    fn joining_two_segments_same_direction_considered_overlapping() {
        //
        // o-->-->
        //
        let h_line = RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::from(vec![seg![Direction::Right, 3], seg![Direction::Right, 3]]),
        };
        assert_eq!(h_line.is_self_overlapping(), true);
    }
}

#[cfg(test)]
mod iter_tests {
    use super::*;

    /// Horizontal straight line segment
    macro_rules! hseg {
        (($x:expr, $y:expr), $len:expr) => {
            StraightLineSegment {
                pos: i32::ivec2($x, $y),
                len: $len,
                axis: Axis::Horizontal,
            }
        };
    }

    /// Vertical straight line segment
    macro_rules! vseg {
        (($x:expr, $y:expr), $len:expr) => {
            StraightLineSegment {
                pos: i32::ivec2($x, $y),
                len: $len,
                axis: Axis::Vertical,
            }
        };
    }

    #[test]
    fn segment_iterator_on_empty_segments_returns_none() {
        let line = RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::from(vec![ /* nothing */ ]),
        };
        assert_eq!(line.horizontal_segments().next(), None);
    }

    #[test]
    fn axis_aligned_segments_can_be_iterated_over() {
        // <-----^
        //       |
        // o->   |
        //   |   |
        //   |   |
        //   v--->
        let line = RectilinearLine {
            start: i32::ivec2(0, 0),
            segments: VecDeque::from(vec![
                seg!(Direction::Right, 2),
                seg!(Direction::Down, 3),
                seg!(Direction::Right, 4),
                seg!(Direction::Up, 5),
                seg!(Direction::Left, 6),
            ]),
        };

        let h_segments = vec![hseg!((0, 0), 2), hseg!((2, 3), 4), hseg!((0, -2), 6)];
        assert_eq!(line.horizontal_segments().collect::<Vec<_>>(), h_segments);

        let v_segments = vec![vseg!((2, 0), 3), vseg!((6, -2), 5)];
        assert_eq!(line.vertical_segments().collect::<Vec<_>>(), v_segments);
    }
}
