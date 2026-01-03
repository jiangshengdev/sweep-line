use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use sweep_line::geom::fixed::{Coord, PointI64, SCALE};
use sweep_line::geom::segment::{Segment, Segments};
use sweep_line::limits::Limits;
use sweep_line::run::run_phase1;
use sweep_line::session::session_v1_to_json_string_limited;
use sweep_line::sweep::bo::enumerate_point_intersections_with_trace;

const INDEX_SCHEMA: &str = "session-index.v1";

// 性能验证（L 档）常量：方便后期集中修改（与 plans/trace-visualizer.md 保持一致）。
const PERF_GRID_N: usize = 100;
const PERF_SPIDER_SPOKES: usize = 64;
const PERF_SPIDER_RINGS: usize = 40;

const PERF_SPIDER_OUTER_RADIUS: Coord = (SCALE / 100) * 95;
const PERF_CIRCLE_RADIUS_GRID: i64 = 4096;

fn main() {
    let args = match Args::parse() {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("错误：{msg}");
            eprintln!();
            eprintln!("{}", Args::usage());
            std::process::exit(2);
        }
    };

    if let Err(msg) = run(args) {
        eprintln!("错误：{msg}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<(), String> {
    let out_dir = args.out_dir;
    let curated_dir = out_dir.join("curated");
    let perf_dir = out_dir.join("perf");
    let random_dir = out_dir.join("random");
    fs::create_dir_all(&curated_dir).map_err(|e| format!("创建目录失败：{}（{}）", curated_dir.display(), e))?;
    fs::create_dir_all(&perf_dir).map_err(|e| format!("创建目录失败：{}（{}）", perf_dir.display(), e))?;
    fs::create_dir_all(&random_dir).map_err(|e| format!("创建目录失败：{}（{}）", random_dir.display(), e))?;

    let mut items: Vec<IndexItem> = Vec::new();

    items.push(write_curated_basic_cross(&curated_dir)?);
    items.push(write_curated_rational_intersection(&curated_dir)?);
    items.push(write_curated_endpoint_touch(&curated_dir)?);
    items.push(write_curated_preprocess_warnings(&curated_dir)?);

    items.push(write_perf_grid_orthogonal(&perf_dir, PERF_GRID_N)?);
    items.push(write_perf_grid_diagonal_45(&perf_dir, PERF_GRID_N)?);
    items.push(write_perf_spider_web(
        &perf_dir,
        PERF_SPIDER_SPOKES,
        PERF_SPIDER_RINGS,
    )?);

    for i in 0..args.random_count {
        let seed = mix_seed(args.seed, i as u64);
        let item = write_random_case(&random_dir, i, seed, args.segments_per_case)?;
        items.push(item);
    }

    let index_json = index_to_json_string(&items);
    let index_path = out_dir.join("index.json");
    fs::write(&index_path, index_json)
        .map_err(|e| format!("写入 index.json 失败：{}（{}）", index_path.display(), e))?;

    eprintln!(
        "已生成：{}（curated={}，perf={}，random={}），索引：{}",
        out_dir.display(),
        4,
        3,
        args.random_count,
        index_path.display()
    );

    Ok(())
}

#[derive(Clone, Debug)]
struct Args {
    out_dir: PathBuf,
    random_count: usize,
    segments_per_case: usize,
    seed: u64,
}

impl Args {
    fn parse() -> Result<Self, String> {
        let mut out_dir = PathBuf::from("viewer/generated");
        let mut random_count: usize = 30;
        let mut segments_per_case: usize = 24;
        let mut seed: u64 = 1;

        let mut it = env::args().skip(1);
        while let Some(arg) = it.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    return Err("".to_string());
                }
                "--out" => {
                    let Some(v) = it.next() else {
                        return Err("--out 缺少参数".to_string());
                    };
                    out_dir = PathBuf::from(v);
                }
                "--random" => {
                    let Some(v) = it.next() else {
                        return Err("--random 缺少参数".to_string());
                    };
                    random_count = v
                        .parse::<usize>()
                        .map_err(|_| "--random 必须是非负整数".to_string())?;
                }
                "--segments" => {
                    let Some(v) = it.next() else {
                        return Err("--segments 缺少参数".to_string());
                    };
                    segments_per_case = v
                        .parse::<usize>()
                        .map_err(|_| "--segments 必须是非负整数".to_string())?;
                }
                "--seed" => {
                    let Some(v) = it.next() else {
                        return Err("--seed 缺少参数".to_string());
                    };
                    seed = v.parse::<u64>().map_err(|_| "--seed 必须是 u64".to_string())?;
                }
                _ => {
                    return Err(format!("未知参数：{arg}"));
                }
            }
        }

        Ok(Self {
            out_dir,
            random_count,
            segments_per_case,
            seed,
        })
    }

    fn usage() -> &'static str {
        "用法：cargo run --bin generate-viewer-sessions -- [--out <dir>] [--random N] [--segments N] [--seed U64]\n\
\n\
说明：\n\
- 输出 `session.v1` 示例到 <dir>（默认：viewer/generated）。\n\
- 同时生成 <dir>/index.json（schema: session-index.v1），供 viewer 自动加载列表。\n\
\n\
示例：\n\
- 默认生成：cargo run --bin generate-viewer-sessions\n\
- 生成 100 个随机用例，每个 40 条线段：cargo run --bin generate-viewer-sessions -- --random 100 --segments 40\n"
    }
}

#[derive(Clone, Debug)]
struct IndexItem {
    id: String,
    title: String,
    path: String,
    tags: Vec<String>,
    segments: usize,
    steps: usize,
    warnings: usize,
}

fn write_curated_basic_cross(curated_dir: &Path) -> Result<IndexItem, String> {
    let mut segments = Segments::new();
    push_segment(
        &mut segments,
        PointI64 { x: -SCALE, y: 0 },
        PointI64 { x: SCALE, y: 0 },
        0,
    );
    push_segment(
        &mut segments,
        PointI64 { x: 0, y: -SCALE },
        PointI64 { x: 0, y: SCALE },
        1,
    );

    let (_hits, trace) = enumerate_point_intersections_with_trace(&segments)
        .map_err(|e| format!("运行算法失败（basic-cross）：{e}"))?;

    let json = session_v1_to_json_string_limited(&segments, &trace, Limits::default())
        .map_err(|e| format!("生成 session JSON 失败（basic-cross）：{e}"))?;
    let rel_path = "generated/curated/basic-cross.json".to_string();
    let out_path = curated_dir.join("basic-cross.json");
    fs::write(&out_path, json)
        .map_err(|e| format!("写入失败：{}（{}）", out_path.display(), e))?;

    Ok(IndexItem {
        id: "basic-cross".to_string(),
        title: "基本：垂直命中（VerticalFlush）".to_string(),
        path: rel_path,
        tags: vec!["curated".to_string(), "vertical".to_string()],
        segments: segments.len(),
        steps: trace.steps.len(),
        warnings: trace.warnings.len(),
    })
}

fn write_curated_rational_intersection(curated_dir: &Path) -> Result<IndexItem, String> {
    let mut segments = Segments::new();
    push_segment(
        &mut segments,
        PointI64 { x: -SCALE, y: 0 },
        PointI64 { x: SCALE, y: 0 },
        0,
    );
    push_segment(
        &mut segments,
        PointI64 { x: 0, y: SCALE / 2 },
        PointI64 { x: SCALE, y: -SCALE },
        1,
    );

    let (_hits, trace) = enumerate_point_intersections_with_trace(&segments)
        .map_err(|e| format!("运行算法失败（rational-intersection）：{e}"))?;

    let json = session_v1_to_json_string_limited(&segments, &trace, Limits::default())
        .map_err(|e| format!("生成 session JSON 失败（rational-intersection）：{e}"))?;
    let rel_path = "generated/curated/rational-intersection.json".to_string();
    let out_path = curated_dir.join("rational-intersection.json");
    fs::write(&out_path, json)
        .map_err(|e| format!("写入失败：{}（{}）", out_path.display(), e))?;

    Ok(IndexItem {
        id: "rational-intersection".to_string(),
        title: "基本：有理数交点（x=1/3）".to_string(),
        path: rel_path,
        tags: vec!["curated".to_string(), "rational".to_string()],
        segments: segments.len(),
        steps: trace.steps.len(),
        warnings: trace.warnings.len(),
    })
}

fn write_curated_endpoint_touch(curated_dir: &Path) -> Result<IndexItem, String> {
    let mut segments = Segments::new();
    push_segment(
        &mut segments,
        PointI64 { x: -SCALE / 2, y: 0 },
        PointI64 { x: 0, y: 0 },
        0,
    );
    push_segment(
        &mut segments,
        PointI64 { x: 0, y: 0 },
        PointI64 { x: SCALE / 2, y: SCALE / 2 },
        1,
    );

    let (_hits, trace) = enumerate_point_intersections_with_trace(&segments)
        .map_err(|e| format!("运行算法失败（endpoint-touch）：{e}"))?;

    let json = session_v1_to_json_string_limited(&segments, &trace, Limits::default())
        .map_err(|e| format!("生成 session JSON 失败（endpoint-touch）：{e}"))?;
    let rel_path = "generated/curated/endpoint-touch.json".to_string();
    let out_path = curated_dir.join("endpoint-touch.json");
    fs::write(&out_path, json)
        .map_err(|e| format!("写入失败：{}（{}）", out_path.display(), e))?;

    Ok(IndexItem {
        id: "endpoint-touch".to_string(),
        title: "退化：端点接触（EndpointTouch）".to_string(),
        path: rel_path,
        tags: vec!["curated".to_string(), "degenerate".to_string()],
        segments: segments.len(),
        steps: trace.steps.len(),
        warnings: trace.warnings.len(),
    })
}

fn write_curated_preprocess_warnings(curated_dir: &Path) -> Result<IndexItem, String> {
    let input = [
        sweep_line::InputSegmentF64 {
            ax: 2.0,
            ay: 0.0,
            bx: 0.0,
            by: 0.0,
        },
        sweep_line::InputSegmentF64 {
            ax: 0.0,
            ay: 0.0,
            bx: 0.0,
            by: 0.0,
        },
        sweep_line::InputSegmentF64 {
            ax: -0.5,
            ay: -0.5,
            bx: 0.5,
            by: 0.5,
        },
    ];
    let out = run_phase1(&input).map_err(|e| format!("运行 phase1 失败（warnings）：{e}"))?;
    let json = session_v1_to_json_string_limited(&out.preprocess.segments, &out.trace, Limits::default())
        .map_err(|e| format!("生成 session JSON 失败（warnings）：{e}"))?;

    let rel_path = "generated/curated/preprocess-warnings.json".to_string();
    let out_path = curated_dir.join("preprocess-warnings.json");
    fs::write(&out_path, json)
        .map_err(|e| format!("写入失败：{}（{}）", out_path.display(), e))?;

    Ok(IndexItem {
        id: "preprocess-warnings".to_string(),
        title: "预处理告警：越界/零长度丢弃".to_string(),
        path: rel_path,
        tags: vec!["curated".to_string(), "warnings".to_string()],
        segments: out.preprocess.segments.len(),
        steps: out.trace.steps.len(),
        warnings: out.trace.warnings.len(),
    })
}

fn write_perf_grid_orthogonal(perf_dir: &Path, n: usize) -> Result<IndexItem, String> {
    let segments = build_perf_grid_orthogonal(n);
    let (_hits, trace) = enumerate_point_intersections_with_trace(&segments)
        .map_err(|e| format!("运行算法失败（perf-grid-orthogonal）：{e}"))?;

    let json = session_v1_to_json_string_limited(&segments, &trace, Limits::default())
        .map_err(|e| format!("生成 session JSON 失败（perf-grid-orthogonal）：{e}"))?;
    let file_name = "perf-grid-orthogonal.json";
    let rel_path = format!("generated/perf/{file_name}");
    let out_path = perf_dir.join(file_name);
    fs::write(&out_path, json).map_err(|e| format!("写入失败：{}（{}）", out_path.display(), e))?;

    Ok(IndexItem {
        id: "perf-grid-orthogonal".to_string(),
        title: format!("性能：正交网格（N={n}）"),
        path: rel_path,
        tags: vec![
            "perf".to_string(),
            "grid".to_string(),
            "orthogonal".to_string(),
            "vertical".to_string(),
        ],
        segments: segments.len(),
        steps: trace.steps.len(),
        warnings: trace.warnings.len(),
    })
}

fn write_perf_grid_diagonal_45(perf_dir: &Path, n: usize) -> Result<IndexItem, String> {
    let segments = build_perf_grid_diagonal_45(n);
    let (_hits, trace) = enumerate_point_intersections_with_trace(&segments)
        .map_err(|e| format!("运行算法失败（perf-grid-diagonal-45）：{e}"))?;

    let json = session_v1_to_json_string_limited(&segments, &trace, Limits::default())
        .map_err(|e| format!("生成 session JSON 失败（perf-grid-diagonal-45）：{e}"))?;
    let file_name = "perf-grid-diagonal-45.json";
    let rel_path = format!("generated/perf/{file_name}");
    let out_path = perf_dir.join(file_name);
    fs::write(&out_path, json).map_err(|e| format!("写入失败：{}（{}）", out_path.display(), e))?;

    Ok(IndexItem {
        id: "perf-grid-diagonal-45".to_string(),
        title: format!("性能：45° 网格（N={n}）"),
        path: rel_path,
        tags: vec![
            "perf".to_string(),
            "grid".to_string(),
            "diagonal".to_string(),
            "rational".to_string(),
        ],
        segments: segments.len(),
        steps: trace.steps.len(),
        warnings: trace.warnings.len(),
    })
}

fn write_perf_spider_web(perf_dir: &Path, spokes: usize, rings: usize) -> Result<IndexItem, String> {
    let segments = build_perf_spider_web(spokes, rings)?;
    let (_hits, trace) = enumerate_point_intersections_with_trace(&segments)
        .map_err(|e| format!("运行算法失败（perf-spider-web）：{e}"))?;

    let json = session_v1_to_json_string_limited(&segments, &trace, Limits::default())
        .map_err(|e| format!("生成 session JSON 失败（perf-spider-web）：{e}"))?;
    let file_name = "perf-spider-web.json";
    let rel_path = format!("generated/perf/{file_name}");
    let out_path = perf_dir.join(file_name);
    fs::write(&out_path, json).map_err(|e| format!("写入失败：{}（{}）", out_path.display(), e))?;

    Ok(IndexItem {
        id: "perf-spider-web".to_string(),
        title: format!("性能：蜘蛛网（spokes={spokes}, rings={rings}）"),
        path: rel_path,
        tags: vec!["perf".to_string(), "spider".to_string()],
        segments: segments.len(),
        steps: trace.steps.len(),
        warnings: trace.warnings.len(),
    })
}

fn write_random_case(
    random_dir: &Path,
    index: usize,
    seed: u64,
    segments_per_case: usize,
) -> Result<IndexItem, String> {
    let segments = build_random_segments(seed, segments_per_case);

    let (_hits, trace) = enumerate_point_intersections_with_trace(&segments)
        .map_err(|e| format!("运行算法失败（random-{index:04}）：{e}"))?;

    let json = session_v1_to_json_string_limited(&segments, &trace, Limits::default())
        .map_err(|e| format!("生成 session JSON 失败（random-{index:04}）：{e}"))?;
    let file_name = format!("random-{index:04}.json");
    let rel_path = format!("generated/random/{file_name}");
    let out_path = random_dir.join(&file_name);
    fs::write(&out_path, json)
        .map_err(|e| format!("写入失败：{}（{}）", out_path.display(), e))?;

    Ok(IndexItem {
        id: format!("random-{index:04}"),
        title: format!("随机：{segments_per_case} 条线段（seed={seed}）"),
        path: rel_path,
        tags: vec!["random".to_string()],
        segments: segments.len(),
        steps: trace.steps.len(),
        warnings: trace.warnings.len(),
    })
}

fn build_random_segments(seed: u64, segments_per_case: usize) -> Segments {
    let mut rng = XorShift64::new(seed);
    let mut segments = Segments::new();

    let mut generated = 0;
    while generated < segments_per_case {
        let a = random_point_on_grid(&mut rng, 60);
        let b = random_point_on_grid(&mut rng, 60);
        if a == b {
            continue;
        }
        push_segment(&mut segments, a, b, generated);
        generated += 1;
    }

    segments
}

fn build_perf_grid_orthogonal(n: usize) -> Segments {
    let mut segments = Segments::new();
    let coords = linspace_inner(-SCALE, SCALE, n);

    let mut source_index = 0;
    for &y in &coords {
        push_segment(
            &mut segments,
            PointI64 { x: -SCALE, y },
            PointI64 { x: SCALE, y },
            source_index,
        );
        source_index += 1;
    }
    for &x in &coords {
        push_segment(
            &mut segments,
            PointI64 { x, y: -SCALE },
            PointI64 { x, y: SCALE },
            source_index,
        );
        source_index += 1;
    }

    segments
}

fn build_perf_grid_diagonal_45(n: usize) -> Segments {
    let mut segments = Segments::new();

    // 为了避免“交点恰好落在边界端点”引发退化事件，45° 网格用更小的边界框。
    let bound = PERF_SPIDER_OUTER_RADIUS;
    let intercepts = linspace_inner(-bound, bound, n);

    let mut source_index = 0;
    for &b in &intercepts {
        let (a, c) = clip_line_slope_plus1(bound, b);
        push_segment(&mut segments, a, c, source_index);
        source_index += 1;
    }
    for &c in &intercepts {
        let (a, b) = clip_line_slope_minus1(bound, c);
        push_segment(&mut segments, a, b, source_index);
        source_index += 1;
    }

    segments
}

fn build_perf_spider_web(spokes: usize, rings: usize) -> Result<Segments, String> {
    if spokes < 4 {
        return Err("spokes 至少为 4".to_string());
    }
    if rings < 1 {
        return Err("rings 至少为 1".to_string());
    }

    let dirs = select_circle_directions_evenly(spokes * 2, PERF_CIRCLE_RADIUS_GRID)?;
    let spoke_dirs: Vec<Vec2> = dirs.iter().step_by(2).copied().collect();
    let ring_dirs: Vec<Vec2> = dirs.iter().skip(1).step_by(2).copied().collect();
    debug_assert_eq!(spoke_dirs.len(), spokes);
    debug_assert_eq!(ring_dirs.len(), spokes);

    let outer = PERF_SPIDER_OUTER_RADIUS;
    let denom = (rings as i128) + 1;
    let spoke_inner = ((outer as i128) / (denom * 2)) as Coord;

    let mut segments = Segments::new();
    let mut source_index = 0;

    // spokes：长线段（内半径 -> 外半径），避免大量端点重合在原点。
    for &d in &spoke_dirs {
        let a = scale_dir_to_point(d, spoke_inner);
        let b = scale_dir_to_point(d, outer);
        if a == b {
            continue;
        }
        push_segment(&mut segments, a, b, source_index);
        source_index += 1;
    }

    // rings：每圈用多段折线闭合（顶点角度与 spokes 错开，尽量让 spoke 与 ring 的交点落在边上而非顶点）。
    for k in 1..=rings {
        let radius = ((outer as i128) * (k as i128) / denom) as Coord;
        let mut points: Vec<PointI64> = Vec::with_capacity(spokes);
        for &d in &ring_dirs {
            points.push(scale_dir_to_point(d, radius));
        }
        for i in 0..spokes {
            let a = points[i];
            let b = points[(i + 1) % spokes];
            if a == b {
                continue;
            }
            push_segment(&mut segments, a, b, source_index);
            source_index += 1;
        }
    }

    Ok(segments)
}

fn push_segment(segments: &mut Segments, a: PointI64, b: PointI64, source_index: usize) {
    let (a, b) = canonicalize_endpoints(a, b);
    segments.push(Segment { a, b, source_index });
}

fn canonicalize_endpoints(mut a: PointI64, mut b: PointI64) -> (PointI64, PointI64) {
    if b < a {
        core::mem::swap(&mut a, &mut b);
    }
    (a, b)
}

fn random_point_on_grid(rng: &mut XorShift64, steps: i64) -> PointI64 {
    // 为了更稳定地生成“可视化可读”的用例，这里只在规则网格上取点，减少极端退化。
    // steps=60 表示将 [-SCALE, SCALE] 等分为 60 个刻度（含正负与 0）。
    debug_assert!(steps > 0);
    let idx_x = rng.next_u64() % ((steps as u64) * 2 + 1);
    let idx_y = rng.next_u64() % ((steps as u64) * 2 + 1);
    let x = grid_value(idx_x as i64, steps);
    let y = grid_value(idx_y as i64, steps);
    PointI64 { x, y }
}

fn grid_value(idx: i64, steps: i64) -> Coord {
    let max = steps;
    let signed = idx - max;
    let value = (signed as i128) * (SCALE as i128) / (max as i128);
    value as Coord
}

fn linspace_inner(min: Coord, max: Coord, count: usize) -> Vec<Coord> {
    // 生成 count 个“去掉端点”的等分坐标：避免恰好取到边界端点导致交点落在端点。
    debug_assert!(min < max);
    let count_i = count as i128;
    let min_i = min as i128;
    let max_i = max as i128;
    let span = max_i - min_i;

    let mut out: Vec<Coord> = Vec::with_capacity(count);
    for i in 0..count {
        let v = min_i + ((i as i128 + 1) * span) / (count_i + 1);
        out.push(v as Coord);
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Vec2 {
    x: i64,
    y: i64,
}

fn scale_dir_to_point(dir: Vec2, radius: Coord) -> PointI64 {
    let x = ((dir.x as i128) * (radius as i128) / (PERF_CIRCLE_RADIUS_GRID as i128)) as Coord;
    let y = ((dir.y as i128) * (radius as i128) / (PERF_CIRCLE_RADIUS_GRID as i128)) as Coord;
    PointI64 { x, y }
}

fn select_circle_directions_evenly(count: usize, radius: i64) -> Result<Vec<Vec2>, String> {
    if count == 0 {
        return Err("count 不能为 0".to_string());
    }
    if radius <= 0 {
        return Err("radius 必须为正数".to_string());
    }

    let dirs = build_circle_directions(radius);
    if dirs.len() < count {
        return Err(format!(
            "圆周方向数量不足：需要 {count}，实际 {}",
            dirs.len()
        ));
    }

    let mut out: Vec<Vec2> = Vec::with_capacity(count);
    for i in 0..count {
        let idx = (i * dirs.len()) / count;
        out.push(dirs[idx]);
    }

    // 如果等分取样碰到重复（极端情况下可能发生），用线性扫描补齐。
    let mut seen = std::collections::BTreeSet::<(i64, i64)>::new();
    let mut unique: Vec<Vec2> = Vec::with_capacity(count);
    for v in out.into_iter() {
        if seen.insert((v.x, v.y)) {
            unique.push(v);
        }
    }
    if unique.len() == count {
        return Ok(unique);
    }

    for v in dirs {
        if unique.len() == count {
            break;
        }
        if seen.insert((v.x, v.y)) {
            unique.push(v);
        }
    }

    if unique.len() != count {
        return Err(format!("无法补齐方向向量：需要 {count}，实际 {}", unique.len()));
    }
    Ok(unique)
}

fn build_circle_directions(radius: i64) -> Vec<Vec2> {
    let r2 = (radius as u128) * (radius as u128);
    let mut raw: Vec<Vec2> = Vec::new();
    for x in 0..=radius {
        let rem = r2 - (x as u128) * (x as u128);
        let y = isqrt_u128(rem) as i64;
        push_circle_sym_points(&mut raw, x, y);
    }

    raw.retain(|v| !(v.x == 0 && v.y == 0));
    raw.sort_by(|a, b| angle_cmp(*a, *b));
    raw.dedup_by(|a, b| a.x == b.x && a.y == b.y);
    raw
}

fn push_circle_sym_points(out: &mut Vec<Vec2>, x: i64, y: i64) {
    let candidates = [
        (x, y),
        (y, x),
        (-x, y),
        (-y, x),
        (x, -y),
        (y, -x),
        (-x, -y),
        (-y, -x),
    ];
    for (x, y) in candidates {
        out.push(Vec2 { x, y });
    }
}

fn angle_cmp(a: Vec2, b: Vec2) -> core::cmp::Ordering {
    let ha = is_upper_half(a);
    let hb = is_upper_half(b);
    if ha != hb {
        // upper half-plane 优先（从 +x 轴开始逆时针）
        return hb.cmp(&ha);
    }

    let cross = (a.x as i128) * (b.y as i128) - (a.y as i128) * (b.x as i128);
    if cross != 0 {
        return if cross > 0 {
            core::cmp::Ordering::Less
        } else {
            core::cmp::Ordering::Greater
        };
    }

    // 同方向：用坐标稳定排序（避免平台差异）。
    match a.x.cmp(&b.x) {
        core::cmp::Ordering::Equal => a.y.cmp(&b.y),
        o => o,
    }
}

fn is_upper_half(v: Vec2) -> bool {
    v.y > 0 || (v.y == 0 && v.x >= 0)
}

fn isqrt_u128(n: u128) -> u128 {
    if n < 2 {
        return n;
    }

    // Newton-Raphson：确定性、无依赖。
    let mut x0 = n;
    let mut x1 = (x0 + 1) / 2;
    while x1 < x0 {
        x0 = x1;
        x1 = (x1 + n / x1) / 2;
    }
    x0
}

fn clip_line_slope_plus1(bound: Coord, intercept: Coord) -> (PointI64, PointI64) {
    // y = x + intercept
    let mut points: Vec<PointI64> = Vec::with_capacity(4);

    let y = -bound + intercept;
    if y >= -bound && y <= bound {
        points.push(PointI64 { x: -bound, y });
    }
    let y = bound + intercept;
    if y >= -bound && y <= bound {
        points.push(PointI64 { x: bound, y });
    }

    let x = -bound - intercept;
    if x >= -bound && x <= bound {
        points.push(PointI64 { x, y: -bound });
    }
    let x = bound - intercept;
    if x >= -bound && x <= bound {
        points.push(PointI64 { x, y: bound });
    }

    points.sort();
    points.dedup();
    debug_assert_eq!(points.len(), 2, "clip_line_slope_plus1 应得到 2 个端点");
    (points[0], points[1])
}

fn clip_line_slope_minus1(bound: Coord, intercept: Coord) -> (PointI64, PointI64) {
    // y = -x + intercept
    let mut points: Vec<PointI64> = Vec::with_capacity(4);

    let y = bound + intercept;
    if y >= -bound && y <= bound {
        points.push(PointI64 { x: -bound, y });
    }
    let y = -bound + intercept;
    if y >= -bound && y <= bound {
        points.push(PointI64 { x: bound, y });
    }

    let x = intercept + bound;
    if x >= -bound && x <= bound {
        points.push(PointI64 { x, y: -bound });
    }
    let x = intercept - bound;
    if x >= -bound && x <= bound {
        points.push(PointI64 { x, y: bound });
    }

    points.sort();
    points.dedup();
    debug_assert_eq!(points.len(), 2, "clip_line_slope_minus1 应得到 2 个端点");
    (points[0], points[1])
}

fn mix_seed(base: u64, salt: u64) -> u64 {
    base ^ salt.wrapping_mul(0x9e37_79b9_7f4a_7c15)
}

/// 一个简单的确定性 RNG（无依赖），用于生成可复现示例。
#[derive(Clone, Copy, Debug)]
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        let seed = if seed == 0 { 0x4d59_5df4_d0f3_3173 } else { seed };
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

fn index_to_json_string(items: &[IndexItem]) -> String {
    let mut out = String::new();
    out.push('{');
    write_kv_str(&mut out, "schema", INDEX_SCHEMA);
    out.push(',');
    out.push('"');
    out.push_str("items");
    out.push('"');
    out.push(':');
    out.push('[');
    for (i, item) in items.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        write_index_item(&mut out, item);
    }
    out.push(']');
    out.push('}');
    out
}

fn write_index_item(out: &mut String, item: &IndexItem) {
    out.push('{');
    write_kv_str(out, "id", &item.id);
    out.push(',');
    write_kv_str(out, "title", &item.title);
    out.push(',');
    write_kv_str(out, "path", &item.path);
    out.push(',');
    write_kv_string_array(out, "tags", &item.tags);
    out.push(',');
    write_kv_usize(out, "segments", item.segments);
    out.push(',');
    write_kv_usize(out, "steps", item.steps);
    out.push(',');
    write_kv_usize(out, "warnings", item.warnings);
    out.push('}');
}

fn write_kv_usize(out: &mut String, key: &str, value: usize) {
    out.push('"');
    out.push_str(key);
    out.push('"');
    out.push(':');
    out.push_str(&value.to_string());
}

fn write_kv_str(out: &mut String, key: &str, value: &str) {
    out.push('"');
    out.push_str(key);
    out.push('"');
    out.push(':');
    write_json_string(out, value);
}

fn write_kv_string_array(out: &mut String, key: &str, value: &[String]) {
    out.push('"');
    out.push_str(key);
    out.push('"');
    out.push(':');
    out.push('[');
    for (i, item) in value.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        write_json_string(out, item);
    }
    out.push(']');
}

fn write_json_string(out: &mut String, value: &str) {
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str("\\u");
                let code = c as u32;
                out.push(hex_nibble((code >> 12) & 0xF));
                out.push(hex_nibble((code >> 8) & 0xF));
                out.push(hex_nibble((code >> 4) & 0xF));
                out.push(hex_nibble(code & 0xF));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn hex_nibble(v: u32) -> char {
    debug_assert!(v < 16);
    match v {
        0..=9 => (b'0' + v as u8) as char,
        _ => (b'a' + (v as u8 - 10)) as char,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_json_is_deterministic() {
        let items = vec![
            IndexItem {
                id: "a".to_string(),
                title: "A".to_string(),
                path: "generated/a.json".to_string(),
                tags: vec!["x".to_string(), "y".to_string()],
                segments: 1,
                steps: 2,
                warnings: 3,
            },
            IndexItem {
                id: "b".to_string(),
                title: "B".to_string(),
                path: "generated/b.json".to_string(),
                tags: vec![],
                segments: 0,
                steps: 0,
                warnings: 0,
            },
        ];
        let a = index_to_json_string(&items);
        let b = index_to_json_string(&items);
        assert_eq!(a, b);
        assert_eq!(
            a,
            "{\"schema\":\"session-index.v1\",\"items\":[{\"id\":\"a\",\"title\":\"A\",\"path\":\"generated/a.json\",\"tags\":[\"x\",\"y\"],\"segments\":1,\"steps\":2,\"warnings\":3},{\"id\":\"b\",\"title\":\"B\",\"path\":\"generated/b.json\",\"tags\":[],\"segments\":0,\"steps\":0,\"warnings\":0}]}"
        );
    }

    #[test]
    fn random_case_is_deterministic_for_same_seed() {
        let seed = mix_seed(1, 0);
        let segments = build_random_segments(seed, 12);
        let (_hits_a, trace_a) = enumerate_point_intersections_with_trace(&segments).unwrap();
        let json_a =
            session_v1_to_json_string_limited(&segments, &trace_a, Limits::default()).unwrap();

        let segments_b = build_random_segments(seed, 12);
        let (_hits_b, trace_b) = enumerate_point_intersections_with_trace(&segments_b).unwrap();
        let json_b =
            session_v1_to_json_string_limited(&segments_b, &trace_b, Limits::default()).unwrap();

        assert_eq!(json_a, json_b);
        assert!(json_a.starts_with("{\"schema\":\"session.v1\""));
    }

    #[test]
    fn perf_grid_orthogonal_is_deterministic() {
        let segments_a = build_perf_grid_orthogonal(20);
        let (_hits_a, trace_a) = enumerate_point_intersections_with_trace(&segments_a).unwrap();
        let json_a =
            session_v1_to_json_string_limited(&segments_a, &trace_a, Limits::default()).unwrap();

        let segments_b = build_perf_grid_orthogonal(20);
        let (_hits_b, trace_b) = enumerate_point_intersections_with_trace(&segments_b).unwrap();
        let json_b =
            session_v1_to_json_string_limited(&segments_b, &trace_b, Limits::default()).unwrap();

        assert_eq!(json_a, json_b);
        assert!(json_a.starts_with("{\"schema\":\"session.v1\""));
    }

    #[test]
    fn perf_grid_diagonal_45_is_deterministic() {
        let segments_a = build_perf_grid_diagonal_45(20);
        let (_hits_a, trace_a) = enumerate_point_intersections_with_trace(&segments_a).unwrap();
        let json_a =
            session_v1_to_json_string_limited(&segments_a, &trace_a, Limits::default()).unwrap();

        let segments_b = build_perf_grid_diagonal_45(20);
        let (_hits_b, trace_b) = enumerate_point_intersections_with_trace(&segments_b).unwrap();
        let json_b =
            session_v1_to_json_string_limited(&segments_b, &trace_b, Limits::default()).unwrap();

        assert_eq!(json_a, json_b);
        assert!(json_a.starts_with("{\"schema\":\"session.v1\""));
    }

    #[test]
    fn perf_spider_web_is_deterministic() {
        let segments_a = build_perf_spider_web(16, 6).unwrap();
        let (_hits_a, trace_a) = enumerate_point_intersections_with_trace(&segments_a).unwrap();
        let json_a =
            session_v1_to_json_string_limited(&segments_a, &trace_a, Limits::default()).unwrap();

        let segments_b = build_perf_spider_web(16, 6).unwrap();
        let (_hits_b, trace_b) = enumerate_point_intersections_with_trace(&segments_b).unwrap();
        let json_b =
            session_v1_to_json_string_limited(&segments_b, &trace_b, Limits::default()).unwrap();

        assert_eq!(json_a, json_b);
        assert!(json_a.starts_with("{\"schema\":\"session.v1\""));
    }
}
