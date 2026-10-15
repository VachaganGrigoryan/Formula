//! Sorting algorithms that record every step they take.
//!
//! Each algorithm runs once on a copy of the data and logs an [`Op`] for each
//! comparison, swap, write and bookkeeping change. The visualizer then replays
//! that log at any speed, forward or backward, like scrubbing a video. The
//! algorithms follow the textbook versions step for step.

/// One recorded step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    /// Two elements are compared.
    Compare(usize, usize),
    /// The held insertion key is compared with the element at this index.
    CompareKey(usize),
    /// Two elements trade places.
    Swap(usize, usize),
    /// A value is written into the array (merge, insertion shifts, copy-back).
    Write(usize, u32),
    /// This index now holds its final value.
    Sorted(usize),
    /// Quick sort's pivot index.
    Pivot(Option<usize>),
    /// Selection sort's smallest value found so far.
    Min(Option<usize>),
    /// Insertion sort's key, lifted out of the array, and the hole it would fill.
    Key(Option<(u32, usize)>),
    /// Insertion sort's sorted partition, `lo..hi`.
    Range {
        lo: usize,
        hi: usize,
        mid: Option<usize>,
    },
    ClearRange,
    /// A recursive call starts on `lo..hi` (merge sort records its split point).
    Enter {
        lo: usize,
        hi: usize,
        mid: Option<usize>,
    },
    /// The innermost recursive call returns.
    Exit,
    /// Heap sort: indices below this form the heap.
    HeapSize(usize),
    /// Counting sort: allocate counts for values `0..=max` and an output array.
    StartCounting {
        max: u32,
    },
    /// Counting sort: tally the value at this index.
    Count(usize),
    /// Counting sort: `count[i] += count[i − 1]`.
    Prefix(usize),
    /// Counting sort: place the value at `from` into `output[to]` and decrement its count.
    Place {
        from: usize,
        to: usize,
    },
    /// Counting sort: the output array has been copied back and is gone.
    FinishCounting,
}

/// Wraps the array being sorted and records every operation on it.
pub struct Recorder {
    pub a: Vec<u32>,
    pub ops: Vec<Op>,
    /// When false, operations are carried out but not logged (for timing).
    recording: bool,
}

impl Recorder {
    pub fn new(a: &[u32]) -> Self {
        Self {
            a: a.to_vec(),
            ops: Vec::new(),
            recording: true,
        }
    }

    /// A recorder that sorts without keeping a log.
    pub fn silent(a: &[u32]) -> Self {
        Self {
            recording: false,
            ..Self::new(a)
        }
    }

    fn push(&mut self, op: Op) {
        if self.recording {
            self.ops.push(op);
        }
    }

    /// `a[i] > a[j]`, recorded as a comparison.
    fn greater(&mut self, i: usize, j: usize) -> bool {
        self.push(Op::Compare(i, j));
        self.a[i] > self.a[j]
    }

    fn swap(&mut self, i: usize, j: usize) {
        self.push(Op::Swap(i, j));
        self.a.swap(i, j);
    }

    fn write(&mut self, i: usize, v: u32) {
        self.push(Op::Write(i, v));
        self.a[i] = v;
    }

    fn sorted(&mut self, i: usize) {
        self.push(Op::Sorted(i));
    }

    fn sorted_range(&mut self, lo: usize, hi: usize) {
        for i in lo..hi {
            self.sorted(i);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    Bubble,
    Selection,
    Insertion,
    Merge,
    Quick,
    Quick3,
    Heap,
    Counting,
}

impl Algorithm {
    pub const ALL: [Algorithm; 8] = [
        Algorithm::Bubble,
        Algorithm::Selection,
        Algorithm::Insertion,
        Algorithm::Merge,
        Algorithm::Quick,
        Algorithm::Quick3,
        Algorithm::Heap,
        Algorithm::Counting,
    ];

    /// Sort a copy of `data`, returning the recorded steps and the sorted result.
    pub fn record(self, data: &[u32]) -> Recorder {
        let mut r = Recorder::new(data);
        self.run(&mut r);
        r
    }

    /// Average wall-clock microseconds to sort `data`, without recording.
    /// Repeats short sorts so the measurement is not just timer noise.
    pub fn time_us(self, data: &[u32]) -> f64 {
        let start = web_time::Instant::now();
        let mut runs = 0u32;
        while runs < 3 || (start.elapsed().as_secs_f64() < 0.004 && runs < 500) {
            let mut r = Recorder::silent(data);
            self.run(&mut r);
            std::hint::black_box(&r.a);
            runs += 1;
        }
        start.elapsed().as_secs_f64() * 1e6 / runs as f64
    }

    fn run(self, r: &mut Recorder) {
        let r = &mut *r;
        match self {
            Algorithm::Bubble => bubble(r),
            Algorithm::Selection => selection(r),
            Algorithm::Insertion => insertion(r),
            Algorithm::Merge => {
                let n = r.a.len();
                merge_sort(r, 0, n);
                r.push(Op::ClearRange);
            }
            Algorithm::Quick => {
                let n = r.a.len();
                if n > 0 {
                    quick_sort(r, 0, n as isize - 1);
                }
                r.push(Op::ClearRange);
            }
            Algorithm::Quick3 => {
                let n = r.a.len();
                if n > 0 {
                    quick_sort_3way(r, 0, n - 1);
                }
            }
            Algorithm::Heap => heap_sort(r),
            Algorithm::Counting => counting(r),
        }
    }
}

fn bubble(r: &mut Recorder) {
    let n = r.a.len();
    for i in 0..n {
        let mut swapped = false;
        for j in 0..n - i - 1 {
            if r.greater(j, j + 1) {
                r.swap(j, j + 1);
                swapped = true;
            }
        }
        // The largest remaining value has bubbled to the end of this pass.
        r.sorted(n - i - 1);
        if !swapped {
            // No swaps: everything left is already in order.
            r.sorted_range(0, n - i - 1);
            break;
        }
    }
}

fn selection(r: &mut Recorder) {
    let n = r.a.len();
    for i in 0..n {
        let mut min = i;
        r.push(Op::Min(Some(min)));
        for j in i + 1..n {
            if r.greater(min, j) {
                min = j;
                r.push(Op::Min(Some(min)));
            }
        }
        if min != i {
            r.swap(i, min);
        }
        r.push(Op::Min(None));
        r.sorted(i);
    }
}

fn insertion(r: &mut Recorder) {
    let n = r.a.len();
    for i in 1..n {
        let key = r.a[i];
        let mut j = i as isize - 1;
        // Left of i is the sorted partition; the key is the next unsorted value.
        r.push(Op::Range {
            lo: 0,
            hi: i + 1,
            mid: None,
        });
        r.push(Op::Key(Some((key, i))));
        while j >= 0 {
            let ju = j as usize;
            r.push(Op::CompareKey(ju));
            if key >= r.a[ju] {
                break;
            }
            // Shift the bigger value right; the hole moves left.
            let v = r.a[ju];
            r.write(ju + 1, v);
            r.push(Op::Key(Some((key, ju))));
            j -= 1;
        }
        r.write((j + 1) as usize, key);
        r.push(Op::Key(None));
    }
    r.push(Op::ClearRange);
    r.sorted_range(0, n);
}

/// Sort `lo..hi`.
fn merge_sort(r: &mut Recorder, lo: usize, hi: usize) {
    if hi - lo <= 1 {
        if lo == 0 && hi == r.a.len() {
            r.sorted_range(lo, hi);
        }
        return;
    }
    let mid = lo + (hi - lo) / 2;
    r.push(Op::Enter {
        lo,
        hi,
        mid: Some(mid),
    });
    merge_sort(r, lo, mid);
    merge_sort(r, mid, hi);

    // The halves are copied aside (as L and R), then merged back into lo..hi.
    let left: Vec<u32> = r.a[lo..mid].to_vec();
    let right: Vec<u32> = r.a[mid..hi].to_vec();
    let top = lo == 0 && hi == r.a.len();
    let (mut i, mut j, mut k) = (0, 0, lo);
    while i < left.len() && j < right.len() {
        // Compare the two heads, at the positions they started from.
        r.push(Op::Compare(lo + i, mid + j));
        if left[i] <= right[j] {
            r.write(k, left[i]);
            i += 1;
        } else {
            r.write(k, right[j]);
            j += 1;
        }
        if top {
            r.sorted(k);
        }
        k += 1;
    }
    for &v in left[i..].iter().chain(&right[j..]) {
        r.write(k, v);
        if top {
            r.sorted(k);
        }
        k += 1;
    }
    r.push(Op::Exit);
}

/// Sort `low..=high` (Lomuto partition, last element as pivot).
fn quick_sort(r: &mut Recorder, low: isize, high: isize) {
    if low < high {
        r.push(Op::Enter {
            lo: low as usize,
            hi: high as usize + 1,
            mid: None,
        });
        let pi = partition(r, low as usize, high as usize);
        r.sorted(pi);
        quick_sort(r, low, pi as isize - 1);
        quick_sort(r, pi as isize + 1, high);
        r.push(Op::Exit);
    } else if low == high {
        r.sorted(low as usize);
    }
}

fn partition(r: &mut Recorder, low: usize, high: usize) -> usize {
    r.push(Op::Pivot(Some(high)));
    let mut i = low;
    for j in low..high {
        // arr[j] <= pivot  is  !(arr[j] > pivot)
        if !r.greater(j, high) {
            if i != j {
                r.swap(i, j);
            }
            i += 1;
        }
    }
    if i != high {
        r.swap(i, high);
    }
    r.push(Op::Pivot(None));
    i
}

/// Sort `lo..=hi` with Dijkstra's three-way partition: values below the pivot
/// go left, above go right, and every copy of the pivot value lands in the
/// middle at once. On lists with many duplicates that middle band is big, so
/// far less work is left for the recursive calls.
fn quick_sort_3way(r: &mut Recorder, lo: usize, hi: usize) {
    if lo >= hi {
        if lo == hi {
            r.sorted(lo);
        }
        return;
    }
    r.push(Op::Enter {
        lo,
        hi: hi + 1,
        mid: None,
    });
    let pivot = r.a[lo];
    let (mut lt, mut i, mut gt) = (lo, lo + 1, hi);
    r.push(Op::Pivot(Some(lt)));
    while i <= gt {
        // One three-way comparison against the pivot, which always sits at lt.
        r.push(Op::Compare(i, lt));
        if r.a[i] < pivot {
            r.swap(lt, i);
            lt += 1;
            i += 1;
            r.push(Op::Pivot(Some(lt)));
        } else if r.a[i] > pivot {
            if i != gt {
                r.swap(i, gt);
            }
            gt -= 1;
        } else {
            i += 1;
        }
    }
    r.push(Op::Pivot(None));
    // lt..=gt now all equal the pivot: all in their final places.
    r.sorted_range(lt, gt + 1);
    if lt > lo {
        quick_sort_3way(r, lo, lt - 1);
    }
    quick_sort_3way(r, gt + 1, hi);
    r.push(Op::Exit);
}

fn heapify(r: &mut Recorder, n: usize, i: usize) {
    let mut largest = i;
    let (left, right) = (2 * i + 1, 2 * i + 2);
    if left < n && r.greater(left, largest) {
        largest = left;
    }
    if right < n && r.greater(right, largest) {
        largest = right;
    }
    if largest != i {
        r.swap(i, largest);
        heapify(r, n, largest);
    }
}

fn heap_sort(r: &mut Recorder) {
    let n = r.a.len();
    r.push(Op::HeapSize(n));
    for i in (0..n / 2).rev() {
        heapify(r, n, i);
    }
    for i in (1..n).rev() {
        // The root is the maximum: move it behind the heap.
        r.swap(0, i);
        r.sorted(i);
        r.push(Op::HeapSize(i));
        heapify(r, i, 0);
    }
    if n > 0 {
        r.sorted(0);
    }
    r.push(Op::HeapSize(0));
}

fn counting(r: &mut Recorder) {
    let n = r.a.len();
    let Some(&max) = r.a.iter().max() else { return };
    r.push(Op::StartCounting { max });
    let mut count = vec![0usize; max as usize + 1];
    for i in 0..n {
        r.push(Op::Count(i));
        count[r.a[i] as usize] += 1;
    }
    for v in 1..count.len() {
        r.push(Op::Prefix(v));
        count[v] += count[v - 1];
    }
    // Walk the input backwards so equal values keep their order (stability).
    let mut output = vec![0u32; n];
    for i in (0..n).rev() {
        let v = r.a[i] as usize;
        count[v] -= 1;
        r.push(Op::Place {
            from: i,
            to: count[v],
        });
        output[count[v]] = r.a[i];
    }
    for (i, &v) in output.iter().enumerate() {
        r.write(i, v);
        r.sorted(i);
    }
    r.push(Op::FinishCounting);
}

/// Everything the visualizer draws, rebuilt by replaying ops.
#[derive(Clone, Debug, Default)]
pub struct State {
    pub a: Vec<u32>,
    pub sorted: Vec<bool>,
    pub pivot: Option<usize>,
    pub min: Option<usize>,
    pub key: Option<(u32, usize)>,
    pub range: Option<(usize, usize, Option<usize>)>,
    /// Open recursive calls, outermost first.
    pub stack: Vec<(usize, usize, Option<usize>)>,
    pub heap_size: Option<usize>,
    pub counts: Option<Vec<u32>>,
    pub output: Option<Vec<Option<u32>>>,
    pub comparisons: usize,
    pub swaps: usize,
    pub writes: usize,
    /// The op applied most recently, for highlighting.
    pub last: Option<Op>,
}

impl State {
    pub fn new(a: &[u32]) -> Self {
        Self {
            a: a.to_vec(),
            sorted: vec![false; a.len()],
            ..Default::default()
        }
    }

    pub fn apply(&mut self, op: Op) {
        match op {
            Op::Compare(..) | Op::CompareKey(_) => self.comparisons += 1,
            Op::Swap(i, j) => {
                self.a.swap(i, j);
                self.swaps += 1;
            }
            Op::Write(i, v) => {
                self.a[i] = v;
                self.writes += 1;
            }
            Op::Sorted(i) => self.sorted[i] = true,
            Op::Pivot(p) => self.pivot = p,
            Op::Min(m) => self.min = m,
            Op::Key(k) => self.key = k,
            Op::Range { lo, hi, mid } => self.range = Some((lo, hi, mid)),
            Op::ClearRange => self.range = None,
            Op::Enter { lo, hi, mid } => self.stack.push((lo, hi, mid)),
            Op::Exit => {
                self.stack.pop();
            }
            Op::HeapSize(n) => self.heap_size = (n > 0).then_some(n),
            Op::StartCounting { max } => {
                self.counts = Some(vec![0; max as usize + 1]);
                self.output = Some(vec![None; self.a.len()]);
            }
            Op::Count(i) => {
                if let Some(c) = &mut self.counts {
                    c[self.a[i] as usize] += 1;
                }
            }
            Op::Prefix(v) => {
                if let Some(c) = &mut self.counts {
                    c[v] += c[v - 1];
                }
            }
            Op::Place { from, to } => {
                let v = self.a[from];
                if let Some(c) = &mut self.counts {
                    c[v as usize] -= 1;
                }
                if let Some(out) = &mut self.output {
                    out[to] = Some(v);
                }
                self.writes += 1;
            }
            Op::FinishCounting => {
                self.counts = None;
                self.output = None;
            }
        }
        self.last = Some(op);
    }
}

impl crate::formulas::replay::Replay for State {
    type Op = Op;
    fn apply(&mut self, op: Op) {
        State::apply(self, op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replay(data: &[u32], ops: &[Op]) -> State {
        let mut s = State::new(data);
        for &op in ops {
            s.apply(op);
        }
        s
    }

    const CASES: [&[u32]; 8] = [
        &[],
        &[1],
        &[2, 1],
        &[5, 3, 8, 1, 9, 2, 7, 4, 6],
        &[1, 2, 3, 4, 5, 6],
        &[6, 5, 4, 3, 2, 1],
        &[3, 3, 1, 3, 1, 2, 2],
        &[10, 1, 10, 1, 10, 1, 5, 5, 0, 7, 7, 3],
    ];

    #[test]
    fn every_algorithm_sorts_and_replays_to_the_same_result() {
        for alg in Algorithm::ALL {
            for data in CASES {
                let r = alg.record(data);
                let mut expected = data.to_vec();
                expected.sort();
                assert_eq!(r.a, expected, "{alg:?} on {data:?}");
                let s = replay(data, &r.ops);
                assert_eq!(s.a, expected, "{alg:?} replay on {data:?}");
                assert!(
                    s.sorted.iter().all(|&b| b),
                    "{alg:?} left unmarked bars on {data:?}"
                );
                assert!(s.counts.is_none() && s.output.is_none());
                assert!(
                    s.stack.is_empty() && s.range.is_none(),
                    "{alg:?} left state behind"
                );
            }
        }
    }

    #[test]
    fn indices_stay_in_bounds() {
        let data = [9, 2, 7, 2, 5, 1, 8, 3, 3, 6, 0, 4];
        for alg in Algorithm::ALL {
            for op in alg.record(&data).ops {
                let ok = match op {
                    Op::Compare(i, j) | Op::Swap(i, j) => i < data.len() && j < data.len(),
                    Op::CompareKey(i) | Op::Write(i, _) | Op::Sorted(i) | Op::Count(i) => {
                        i < data.len()
                    }
                    Op::Range { lo, hi, .. } | Op::Enter { lo, hi, .. } => {
                        lo < hi && hi <= data.len()
                    }
                    Op::Place { from, to } => from < data.len() && to < data.len(),
                    _ => true,
                };
                assert!(ok, "{alg:?}: {op:?}");
            }
        }
    }

    #[test]
    fn best_cases_are_linear() {
        let sorted: Vec<u32> = (1..=50).collect();
        for alg in [Algorithm::Bubble, Algorithm::Insertion] {
            let s = replay(&sorted, &alg.record(&sorted).ops);
            assert_eq!(s.comparisons, 49, "{alg:?}");
            assert_eq!(
                s.swaps + s.writes,
                if alg == Algorithm::Insertion { 49 } else { 0 }
            );
        }
    }

    #[test]
    fn selection_always_compares_n_squared_over_two() {
        let data: Vec<u32> = (1..=20).rev().collect();
        let s = replay(&data, &Algorithm::Selection.record(&data).ops);
        assert_eq!(s.comparisons, 20 * 19 / 2);
        assert!(s.swaps <= 19);
    }

    #[test]
    fn efficient_sorts_beat_quadratic_on_random_data() {
        let mut rng = crate::rng::Rng::new(11);
        let data: Vec<u32> = (0..200).map(|_| rng.index(200) as u32 + 1).collect();
        let bubble = replay(&data, &Algorithm::Bubble.record(&data).ops).comparisons;
        for alg in [Algorithm::Merge, Algorithm::Quick, Algorithm::Heap] {
            let c = replay(&data, &alg.record(&data).ops).comparisons;
            assert!(c * 4 < bubble, "{alg:?}: {c} vs bubble {bubble}");
        }
        // Counting sort never compares two elements.
        assert_eq!(
            replay(&data, &Algorithm::Counting.record(&data).ops).comparisons,
            0
        );
    }
}

#[cfg(test)]
mod stack_tests {
    use super::*;

    #[test]
    fn merge_sort_recursion_depth_is_logarithmic() {
        let data: Vec<u32> = (0..64).rev().collect();
        let mut s = State::new(&data);
        let mut deepest = 0;
        for op in Algorithm::Merge.record(&data).ops {
            s.apply(op);
            deepest = deepest.max(s.stack.len());
        }
        assert_eq!(deepest, 6, "64 = 2^6 splits six levels deep");
    }

    #[test]
    fn quick_sort_degrades_on_sorted_input() {
        let data: Vec<u32> = (0..30).collect();
        let mut s = State::new(&data);
        let mut deepest = 0;
        for op in Algorithm::Quick.record(&data).ops {
            s.apply(op);
            deepest = deepest.max(s.stack.len());
        }
        // The last-element pivot peels off one value per call.
        assert_eq!(deepest, 29);
    }
}

#[cfg(test)]
mod quick3_tests {
    use super::*;

    fn comparisons(alg: Algorithm, data: &[u32]) -> usize {
        let mut s = State::new(data);
        for op in alg.record(data).ops {
            s.apply(op);
        }
        s.comparisons
    }

    #[test]
    fn three_way_wins_on_duplicates() {
        let data: Vec<u32> = (0..200).map(|i| [10, 20, 30, 40][i % 4]).collect();
        let two_way = comparisons(Algorithm::Quick, &data);
        let three_way = comparisons(Algorithm::Quick3, &data);
        assert!(
            three_way * 5 < two_way,
            "3-way {three_way} vs 2-way {two_way}"
        );
    }

    #[test]
    fn silent_recorder_sorts_without_logging() {
        for alg in Algorithm::ALL {
            let mut r = Recorder::silent(&[5, 1, 4, 2, 3]);
            alg.run(&mut r);
            assert_eq!(r.a, vec![1, 2, 3, 4, 5], "{alg:?}");
            assert!(r.ops.is_empty());
        }
    }

    #[test]
    fn timing_is_positive() {
        assert!(Algorithm::Merge.time_us(&[3, 1, 2]) > 0.0);
    }
}
