// @ts-check

import { stableColorForSegmentId } from "../lib/color.js";

export class UserError extends Error {}

/**
 * @param {unknown} value
 * @param {string} path
 * @returns {Record<string, unknown>}
 */
function parseObject(value, path) {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new UserError(`${path} 不是对象`);
  }
  return /** @type {Record<string, unknown>} */ (value);
}

/**
 * @param {unknown} value
 * @param {string} path
 * @returns {string}
 */
function parseString(value, path) {
  if (typeof value !== "string") {
    throw new UserError(`${path} 不是字符串`);
  }
  return value;
}

/**
 * @template T
 * @param {unknown} value
 * @param {string} path
 * @returns {T[]}
 */
function parseArray(value, path) {
  if (!Array.isArray(value)) {
    throw new UserError(`${path} 不是数组`);
  }
  return /** @type {T[]} */ (value);
}

/**
 * @param {unknown} value
 * @param {string} path
 * @returns {number}
 */
function parseInteger(value, path) {
  if (typeof value === "number") {
    if (!Number.isFinite(value) || !Number.isInteger(value)) {
      throw new UserError(`${path} 不是整数`);
    }
    return value;
  }
  if (typeof value === "string") {
    if (!/^-?\d+$/.test(value)) {
      throw new UserError(`${path} 不是整数文本`);
    }
    const asNumber = Number(value);
    if (!Number.isFinite(asNumber) || !Number.isSafeInteger(asNumber)) {
      throw new UserError(`${path} 超出 JS 安全整数范围`);
    }
    return asNumber;
  }
  throw new UserError(`${path} 不是整数`);
}

/**
 * @param {{ numStr: string, denStr: string }} r
 * @returns {string}
 */
function formatRationalText(r) {
  if (r.denStr === "1") {
    return r.numStr;
  }
  return `${r.numStr}/${r.denStr}`;
}

/**
 * @param {{ num: bigint, den: bigint }} r
 * @returns {number}
 */
function approxRationalToNumber(r) {
  const num = Number(r.num);
  const den = Number(r.den);
  if (!Number.isFinite(num) || !Number.isFinite(den) || den === 0) {
    return NaN;
  }
  return num / den;
}

/**
 * @param {unknown} value
 * @param {string} path
 */
function parseRational(value, path) {
  const obj = parseObject(value, path);
  const numStr = parseString(obj.num, `${path}.num`);
  const denStr = parseString(obj.den, `${path}.den`);
  let num;
  let den;
  try {
    num = BigInt(numStr);
    den = BigInt(denStr);
  } catch {
    throw new UserError(`${path} 不是合法的有理数字符串`);
  }
  if (den === 0n) {
    throw new UserError(`${path}.den 不能为 0`);
  }
  const rat = { numStr, denStr, num, den };
  return {
    ...rat,
    text: formatRationalText(rat),
    value: approxRationalToNumber(rat),
  };
}

/**
 * @param {unknown} value
 * @param {string} path
 */
function parsePointFixed(value, path) {
  const obj = parseObject(value, path);
  return {
    x: parseInteger(obj.x, `${path}.x`),
    y: parseInteger(obj.y, `${path}.y`),
  };
}

/**
 * @param {unknown} value
 * @param {string} path
 */
function parsePointRat(value, path) {
  const obj = parseObject(value, path);
  return {
    x: parseRational(obj.x, `${path}.x`),
    y: parseRational(obj.y, `${path}.y`),
  };
}

/**
 * @param {number[]} ids
 * @returns {number[]}
 */
function normalizeSegmentIdList(ids) {
  const unique = new Set();
  for (const id of ids) {
    if (!Number.isFinite(id)) {
      continue;
    }
    unique.add(id);
  }
  const out = Array.from(unique);
  out.sort((a, b) => a - b);
  return out;
}

/**
 * @param {unknown} value
 * @param {string} path
 */
function parseIntersectionV1(value, path) {
  const obj = parseObject(value, path);
  const a = parseInteger(obj.a, `${path}.a`);
  const b = parseInteger(obj.b, `${path}.b`);
  const kind = parseString(obj.kind, `${path}.kind`);
  const point = parsePointRat(obj.point, `${path}.point`);
  const segments = normalizeSegmentIdList([a, b]);
  return {
    kind,
    kindDetail: kind,
    point,
    segments,
    endpointSegments: null,
    interiorSegments: null,
  };
}

/**
 * 对 `trace.v2` 的按点聚合交点，派生 Phase 2 所需的三分类：
 * - Proper（内部-内部）
 * - EndpointEndpoint（端点-端点）
 * - EndpointInterior（端点-内部）
 *
 * 注意：同一几何点可能同时存在 EndpointEndpoint 与 EndpointInterior（例如多个端点重合且还有线段穿过）。
 * 这时 `kindDetail` 会用 `A+B` 的形式表达多标签。
 *
 * @param {number[]} endpointSegments
 * @param {number[]} interiorSegments
 * @returns {{ kind: "Proper" | "EndpointTouch", kindDetail: string }}
 */
function deriveIntersectionKind(endpointSegments, interiorSegments) {
  if (endpointSegments.length === 0) {
    return { kind: "Proper", kindDetail: "Proper" };
  }

  /** @type {string[]} */
  const tags = [];
  if (endpointSegments.length >= 2) {
    tags.push("EndpointEndpoint");
  }
  if (interiorSegments.length > 0) {
    tags.push("EndpointInterior");
  }

  // 理论上按点聚合输出至少包含 2 条线段；这里留个兜底，避免异常数据导致 UI 崩溃。
  const kindDetail = tags.length ? tags.join("+") : "EndpointTouch";
  return { kind: "EndpointTouch", kindDetail };
}

/**
 * @param {unknown} value
 * @param {string} path
 */
function parseIntersectionV2(value, path) {
  const obj = parseObject(value, path);
  const point = parsePointRat(obj.point, `${path}.point`);
  const endpointSegments = parseArray(obj.endpoint_segments, `${path}.endpoint_segments`).map(
    (v, i) => parseInteger(v, `${path}.endpoint_segments[${i}]`),
  );
  const interiorSegments = parseArray(obj.interior_segments, `${path}.interior_segments`).map(
    (v, i) => parseInteger(v, `${path}.interior_segments[${i}]`),
  );

  const endpointNorm = normalizeSegmentIdList(endpointSegments);
  const interiorNorm = normalizeSegmentIdList(interiorSegments);
  const segments = normalizeSegmentIdList([...endpointNorm, ...interiorNorm]);
  const { kind, kindDetail } = deriveIntersectionKind(endpointNorm, interiorNorm);

  return {
    kind,
    kindDetail,
    point,
    segments,
    endpointSegments: endpointNorm,
    interiorSegments: interiorNorm,
  };
}

/**
 * @param {unknown} value
 * @param {string} path
 * @param {"trace.v1" | "trace.v2"} traceSchema
 */
function parseStep(value, path, traceSchema) {
  const obj = parseObject(value, path);
  const kind = parseString(obj.kind, `${path}.kind`);
  if (kind !== "PointBatch" && kind !== "VerticalFlush") {
    throw new UserError(`${path}.kind 不是 PointBatch/VerticalFlush`);
  }
  const sweepX = parseRational(obj.sweep_x, `${path}.sweep_x`);
  let point = null;
  if (obj.point !== null) {
    point = parsePointRat(obj.point, `${path}.point`);
  }
  const events = parseArray(obj.events, `${path}.events`).map((v, i) =>
    parseString(v, `${path}.events[${i}]`),
  );
  const active = parseArray(obj.active, `${path}.active`).map((v, i) =>
    parseInteger(v, `${path}.active[${i}]`),
  );
  const intersections = parseArray(obj.intersections, `${path}.intersections`).map((v, i) => {
    const itemPath = `${path}.intersections[${i}]`;
    if (traceSchema === "trace.v1") {
      return parseIntersectionV1(v, itemPath);
    }
    if (traceSchema === "trace.v2") {
      return parseIntersectionV2(v, itemPath);
    }
    throw new UserError(`不支持的 trace schema：${traceSchema}`);
  });
  const notes = parseArray(obj.notes, `${path}.notes`).map((v, i) =>
    parseString(v, `${path}.notes[${i}]`),
  );
  return {
    kind,
    sweepX,
    point,
    events,
    active,
    intersections,
    notes,
  };
}

/**
 * @param {unknown} value
 * @param {string} path
 */
function parseTrace(value, path) {
  const obj = parseObject(value, path);
  const schema = parseString(obj.schema, `${path}.schema`);
  if (schema !== "trace.v1" && schema !== "trace.v2") {
    throw new UserError(`${path}.schema 不是 trace.v1/trace.v2`);
  }
  const warnings = parseArray(obj.warnings, `${path}.warnings`).map((v, i) =>
    parseString(v, `${path}.warnings[${i}]`),
  );
  const steps = parseArray(obj.steps, `${path}.steps`).map((v, i) =>
    parseStep(v, `${path}.steps[${i}]`, schema),
  );
  return { schema, warnings, steps };
}

/**
 * @param {unknown} value
 * @param {string} path
 */
function parseSegments(value, path) {
  const items = parseArray(value, path).map((v, i) => parseObject(v, `${path}[${i}]`));
  const segmentsById = [];
  const worldSegments = [];
  for (let i = 0; i < items.length; i++) {
    const itemPath = `${path}[${i}]`;
    const id = parseInteger(items[i].id, `${itemPath}.id`);
    const sourceIndex = parseInteger(items[i].source_index, `${itemPath}.source_index`);
    const a = parsePointFixed(items[i].a, `${itemPath}.a`);
    const b = parsePointFixed(items[i].b, `${itemPath}.b`);
    if (segmentsById[id]) {
      throw new UserError(`${itemPath}.id 重复：${id}`);
    }
    const seg = { id, sourceIndex, a, b, color: stableColorForSegmentId(id) };
    segmentsById[id] = seg;
    worldSegments.push(seg);
  }
  worldSegments.sort((l, r) => l.id - r.id);
  return { segmentsById, segments: worldSegments };
}

/**
 * 解析 `session.v1/session.v2` JSON。
 * @param {unknown} value
 */
export function parseSession(value) {
  const obj = parseObject(value, "$");
  const schema = parseString(obj.schema, "$.schema");
  if (schema !== "session.v1" && schema !== "session.v2") {
    throw new UserError("不是 session.v1/session.v2 文件");
  }
  const fixed = parseObject(obj.fixed, "$.fixed");
  const scaleStr = parseString(fixed.scale, "$.fixed.scale");
  let scaleBig;
  try {
    scaleBig = BigInt(scaleStr);
  } catch {
    throw new UserError("fixed.scale 无效：不是整数");
  }
  if (scaleBig <= 0n) {
    throw new UserError("fixed.scale 无效：必须为正整数");
  }
  const scale = Number(scaleBig);
  if (!Number.isFinite(scale) || scale <= 0) {
    throw new UserError("fixed.scale 无效：超出 JS 可表示范围");
  }

  const segments = parseSegments(obj.segments, "$.segments");
  const trace = parseTrace(obj.trace, "$.trace");
  return {
    schema,
    scaleStr,
    scale,
    segmentsById: segments.segmentsById,
    segments: segments.segments,
    trace,
  };
}
