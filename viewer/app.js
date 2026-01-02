class UserError extends Error {}

const elements = {
  fileInput: document.getElementById("file-input"),
  resetView: document.getElementById("reset-view"),
  prevStep: document.getElementById("prev-step"),
  playPause: document.getElementById("play-pause"),
  nextStep: document.getElementById("next-step"),
  speed: document.getElementById("speed"),
  stepSlider: document.getElementById("step-slider"),
  stepLabel: document.getElementById("step-label"),
  status: document.getElementById("status"),
  dropHint: document.getElementById("drop-hint"),
  viewport: document.getElementById("viewport"),
  staticCanvas: document.getElementById("static-canvas"),
  dynamicCanvas: document.getElementById("dynamic-canvas"),
  sessionMeta: document.getElementById("session-meta"),
  warnings: document.getElementById("warnings"),
  stepMeta: document.getElementById("step-meta"),
  events: document.getElementById("events"),
  notes: document.getElementById("notes"),
  active: document.getElementById("active"),
  intersections: document.getElementById("intersections"),
};

const appState = {
  session: null,
  currentStep: 0,
  playing: false,
  playTimerId: null,
  speedFactor: 1,
  viewport: {
    widthCss: 1,
    heightCss: 1,
    dpr: 1,
  },
  camera: {
    cx: 0,
    cy: 0,
    zoom: 1,
  },
  dragging: {
    active: false,
    lastX: 0,
    lastY: 0,
    pointerId: null,
  },
  render: {
    scheduled: false,
    dirtyStatic: true,
    dirtyDynamic: true,
  },
};

function setStatus(message) {
  elements.status.textContent = message;
}

function setDropHintVisible(visible) {
  elements.dropHint.classList.toggle("hidden", !visible);
}

function clearChildren(node) {
  while (node.firstChild) {
    node.removeChild(node.firstChild);
  }
}

function appendListItems(list, items) {
  const fragment = document.createDocumentFragment();
  for (const item of items) {
    const li = document.createElement("li");
    li.textContent = item;
    fragment.appendChild(li);
  }
  list.appendChild(fragment);
}

function appendKvLines(container, lines) {
  clearChildren(container);
  const fragment = document.createDocumentFragment();
  for (const [key, value] of lines) {
    const div = document.createElement("div");
    div.textContent = `${key}: ${value}`;
    fragment.appendChild(div);
  }
  container.appendChild(fragment);
}

function stableColorForSegmentId(id) {
  const hue = (id * 47) % 360;
  return `hsl(${hue}deg 85% 68%)`;
}

function parseVerticalIdsFromEvents(events) {
  const ids = [];
  for (const event of events) {
    const m = /^Vertical\((\d+)\)$/.exec(event);
    if (!m) {
      continue;
    }
    ids.push(Number(m[1]));
  }
  return ids;
}

function parseVerticalRangesFromNotes(notes) {
  const map = new Map();
  for (const note of notes) {
    const m = /^VerticalRange\((\d+)\): y=\[(-?\d+),(-?\d+)\]$/.exec(note);
    if (!m) {
      continue;
    }
    map.set(Number(m[1]), {
      yMinFixed: Number(m[2]),
      yMaxFixed: Number(m[3]),
    });
  }
  return map;
}

function formatRationalText(r) {
  if (r.denStr === "1") {
    return r.numStr;
  }
  return `${r.numStr}/${r.denStr}`;
}

function approxRationalToNumber(r) {
  const num = Number(r.num);
  const den = Number(r.den);
  if (!Number.isFinite(num) || !Number.isFinite(den) || den === 0) {
    return NaN;
  }
  return num / den;
}

function parseObject(value, path) {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new UserError(`${path} 不是对象`);
  }
  return value;
}

function parseString(value, path) {
  if (typeof value !== "string") {
    throw new UserError(`${path} 不是字符串`);
  }
  return value;
}

function parseArray(value, path) {
  if (!Array.isArray(value)) {
    throw new UserError(`${path} 不是数组`);
  }
  return value;
}

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
  rat.text = formatRationalText(rat);
  rat.value = approxRationalToNumber(rat);
  return rat;
}

function parsePointFixed(value, path) {
  const obj = parseObject(value, path);
  return {
    x: parseInteger(obj.x, `${path}.x`),
    y: parseInteger(obj.y, `${path}.y`),
  };
}

function parsePointRat(value, path) {
  const obj = parseObject(value, path);
  return {
    x: parseRational(obj.x, `${path}.x`),
    y: parseRational(obj.y, `${path}.y`),
  };
}

function parseIntersection(value, path) {
  const obj = parseObject(value, path);
  const a = parseInteger(obj.a, `${path}.a`);
  const b = parseInteger(obj.b, `${path}.b`);
  const kind = parseString(obj.kind, `${path}.kind`);
  const point = parsePointRat(obj.point, `${path}.point`);
  return {
    a,
    b,
    kind,
    point,
  };
}

function parseStep(value, path) {
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
  const intersections = parseArray(obj.intersections, `${path}.intersections`).map((v, i) =>
    parseIntersection(v, `${path}.intersections[${i}]`),
  );
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

function parseTrace(value, path) {
  const obj = parseObject(value, path);
  const schema = parseString(obj.schema, `${path}.schema`);
  if (schema !== "trace.v1") {
    throw new UserError(`${path}.schema 不是 trace.v1`);
  }
  const warnings = parseArray(obj.warnings, `${path}.warnings`).map((v, i) =>
    parseString(v, `${path}.warnings[${i}]`),
  );
  const steps = parseArray(obj.steps, `${path}.steps`).map((v, i) =>
    parseStep(v, `${path}.steps[${i}]`),
  );
  return { schema, warnings, steps };
}

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

function parseSession(value) {
  const obj = parseObject(value, "$");
  const schema = parseString(obj.schema, "$.schema");
  if (schema !== "session.v1") {
    throw new UserError("不是 session.v1 文件");
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

function segmentToWorldPoints(seg, scale) {
  return {
    ax: seg.a.x / scale,
    ay: seg.a.y / scale,
    bx: seg.b.x / scale,
    by: seg.b.y / scale,
  };
}

function pointRatToWorld(point, scale) {
  return {
    x: point.x.value / scale,
    y: point.y.value / scale,
  };
}

function computeBounds(session) {
  let minX = Infinity;
  let maxX = -Infinity;
  let minY = Infinity;
  let maxY = -Infinity;
  for (const seg of session.segments) {
    const p = segmentToWorldPoints(seg, session.scale);
    minX = Math.min(minX, p.ax, p.bx);
    maxX = Math.max(maxX, p.ax, p.bx);
    minY = Math.min(minY, p.ay, p.by);
    maxY = Math.max(maxY, p.ay, p.by);
  }
  if (!Number.isFinite(minX)) {
    minX = -1;
    maxX = 1;
    minY = -1;
    maxY = 1;
  }
  return { minX, maxX, minY, maxY };
}

function resizeCanvases() {
  const dpr = window.devicePixelRatio || 1;
  const rect = elements.viewport.getBoundingClientRect();
  const widthCss = Math.max(1, Math.floor(rect.width));
  const heightCss = Math.max(1, Math.floor(rect.height));
  appState.viewport = { widthCss, heightCss, dpr };
  for (const canvas of [elements.staticCanvas, elements.dynamicCanvas]) {
    canvas.width = Math.floor(widthCss * dpr);
    canvas.height = Math.floor(heightCss * dpr);
    canvas.style.width = `${widthCss}px`;
    canvas.style.height = `${heightCss}px`;
  }
  appState.render.dirtyStatic = true;
  appState.render.dirtyDynamic = true;
  requestRender();
}

function screenToWorld(localX, localY) {
  const { widthCss, heightCss } = appState.viewport;
  const { cx, cy, zoom } = appState.camera;
  const x = (localX - widthCss / 2) / zoom + cx;
  const y = -((localY - heightCss / 2) / zoom) + cy;
  return { x, y };
}

function resetView() {
  const session = appState.session;
  const width = appState.viewport.widthCss;
  const height = appState.viewport.heightCss;
  const bounds = session ? computeBounds(session) : { minX: -1, maxX: 1, minY: -1, maxY: 1 };

  const w = Math.max(1e-9, bounds.maxX - bounds.minX);
  const h = Math.max(1e-9, bounds.maxY - bounds.minY);
  const zoom = 0.9 * Math.min(width / w, height / h);
  appState.camera = {
    cx: (bounds.minX + bounds.maxX) / 2,
    cy: (bounds.minY + bounds.maxY) / 2,
    zoom: Math.max(10, zoom),
  };
  appState.render.dirtyStatic = true;
  appState.render.dirtyDynamic = true;
  requestRender();
}

function requestRender() {
  if (appState.render.scheduled) {
    return;
  }
  appState.render.scheduled = true;
  window.requestAnimationFrame(() => {
    appState.render.scheduled = false;
    renderIfDirty();
  });
}

function renderIfDirty() {
  if (!appState.session) {
    clearCanvas(elements.staticCanvas);
    clearCanvas(elements.dynamicCanvas);
    return;
  }
  if (appState.render.dirtyStatic) {
    renderStaticLayer();
    appState.render.dirtyStatic = false;
  }
  if (appState.render.dirtyDynamic) {
    renderDynamicLayer();
    appState.render.dirtyDynamic = false;
  }
}

function clearCanvas(canvas) {
  const ctx = canvas.getContext("2d");
  ctx.clearRect(0, 0, canvas.width, canvas.height);
}

function renderStaticLayer() {
  const session = appState.session;
  const canvas = elements.staticCanvas;
  const ctx = canvas.getContext("2d");
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  ctx.save();
  ctx.lineWidth = 1.25 * appState.viewport.dpr;
  ctx.globalAlpha = 0.35;
  for (const seg of session.segments) {
    const p = segmentToWorldPoints(seg, session.scale);
    const a = worldToCanvas(p.ax, p.ay);
    const b = worldToCanvas(p.bx, p.by);
    ctx.strokeStyle = seg.color;
    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.lineTo(b.x, b.y);
    ctx.stroke();
  }
  ctx.restore();
}

function worldToCanvas(x, y) {
  const { widthCss, heightCss, dpr } = appState.viewport;
  const { cx, cy, zoom } = appState.camera;
  const px = (x - cx) * zoom + widthCss / 2;
  const py = (-(y - cy) * zoom) + heightCss / 2;
  return { x: px * dpr, y: py * dpr };
}

function renderDynamicLayer() {
  const session = appState.session;
  const canvas = elements.dynamicCanvas;
  const ctx = canvas.getContext("2d");
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  const step = session.trace.steps[appState.currentStep];
  if (!step) {
    return;
  }

  const sweepXWorld = step.sweepX.value / session.scale;
  if (Number.isFinite(sweepXWorld)) {
    const sweepA = worldToCanvas(sweepXWorld, -1e9);
    const sweepB = worldToCanvas(sweepXWorld, 1e9);
    ctx.save();
    ctx.strokeStyle = "rgba(119, 179, 255, 0.75)";
    ctx.lineWidth = 1.5 * appState.viewport.dpr;
    ctx.setLineDash([6, 6]);
    ctx.beginPath();
    ctx.moveTo(sweepA.x, sweepA.y);
    ctx.lineTo(sweepB.x, sweepB.y);
    ctx.stroke();
    ctx.restore();
  }

  const activeSet = new Set(step.active);
  ctx.save();
  ctx.lineWidth = 3 * appState.viewport.dpr;
  ctx.globalAlpha = 0.95;
  for (const id of activeSet) {
    const seg = session.segmentsById[id];
    if (!seg) {
      continue;
    }
    const p = segmentToWorldPoints(seg, session.scale);
    const a = worldToCanvas(p.ax, p.ay);
    const b = worldToCanvas(p.bx, p.by);
    ctx.strokeStyle = seg.color;
    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.lineTo(b.x, b.y);
    ctx.stroke();
  }
  ctx.restore();

  if (step.kind === "VerticalFlush") {
    const verticalIds = parseVerticalIdsFromEvents(step.events);
    const ranges = parseVerticalRangesFromNotes(step.notes);
    ctx.save();
    ctx.globalAlpha = 1.0;
    ctx.lineWidth = 4 * appState.viewport.dpr;
    for (const id of verticalIds) {
      const seg = session.segmentsById[id];
      if (!seg) {
        continue;
      }
      const p = segmentToWorldPoints(seg, session.scale);
      const a = worldToCanvas(p.ax, p.ay);
      const b = worldToCanvas(p.bx, p.by);
      ctx.strokeStyle = seg.color;
      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.lineTo(b.x, b.y);
      ctx.stroke();

      const range = ranges.get(id);
      if (range && Number.isFinite(p.ax)) {
        const yMinWorld = range.yMinFixed / session.scale;
        const yMaxWorld = range.yMaxFixed / session.scale;
        drawVerticalCaps(ctx, p.ax, yMinWorld, yMaxWorld);
      }
    }
    ctx.restore();
  }

  const intersectionsToDraw = session.intersectionsFlat.slice(
    0,
    session.intersectionPrefixCounts[appState.currentStep] ?? 0,
  );
  drawIntersections(ctx, intersectionsToDraw, session.scale, false);
  drawIntersections(ctx, step.intersections, session.scale, true);

  if (step.point) {
    const p = pointRatToWorld(step.point, session.scale);
    drawPoint(ctx, p.x, p.y, 6, "rgba(255, 255, 255, 0.9)");
  }
}

function drawVerticalCaps(ctx, worldX, yMinWorld, yMaxWorld) {
  const cap = 6 * appState.viewport.dpr;
  const pMin = worldToCanvas(worldX, yMinWorld);
  const pMax = worldToCanvas(worldX, yMaxWorld);
  ctx.save();
  ctx.strokeStyle = "rgba(255, 255, 255, 0.7)";
  ctx.lineWidth = 1.5 * appState.viewport.dpr;
  ctx.beginPath();
  ctx.moveTo(pMin.x - cap, pMin.y);
  ctx.lineTo(pMin.x + cap, pMin.y);
  ctx.moveTo(pMax.x - cap, pMax.y);
  ctx.lineTo(pMax.x + cap, pMax.y);
  ctx.stroke();
  ctx.restore();
}

function drawIntersections(ctx, intersections, scale, isCurrentStep) {
  const radius = isCurrentStep ? 4 : 2;
  for (const it of intersections) {
    const p = pointRatToWorld(it.point, scale);
    if (!Number.isFinite(p.x) || !Number.isFinite(p.y)) {
      continue;
    }
    const color =
      it.kind === "Proper"
        ? "rgba(110, 231, 183, 0.9)"
        : "rgba(255, 107, 107, 0.9)";
    drawPoint(ctx, p.x, p.y, radius, color);
  }
}

function drawPoint(ctx, worldX, worldY, radius, fillStyle) {
  const p = worldToCanvas(worldX, worldY);
  ctx.save();
  ctx.fillStyle = fillStyle;
  ctx.beginPath();
  ctx.arc(p.x, p.y, radius, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();
}

function refreshUiForSession(fileName) {
  const session = appState.session;
  appendKvLines(elements.sessionMeta, [
    ["schema", session.schema],
    ["file", fileName ?? "-"],
    ["scale", session.scaleStr],
    ["segments", String(session.segments.length)],
    ["steps", String(session.trace.steps.length)],
  ]);

  clearChildren(elements.warnings);
  if (session.trace.warnings.length === 0) {
    appendListItems(elements.warnings, ["（无）"]);
  } else {
    appendListItems(elements.warnings, session.trace.warnings);
  }

  updateStepControls();
  if (session.trace.steps.length === 0) {
    appendKvLines(elements.stepMeta, [["index", "0/0"]]);
    clearChildren(elements.events);
    clearChildren(elements.notes);
    clearChildren(elements.active);
    clearChildren(elements.intersections);
    return;
  }
  refreshUiForStep();
}

function refreshUiForStep() {
  const session = appState.session;
  const step = session.trace.steps[appState.currentStep];
  if (!step) {
    return;
  }

  const pointText = step.point
    ? `(${step.point.x.text}, ${step.point.y.text})`
    : "null";

  appendKvLines(elements.stepMeta, [
    ["index", `${appState.currentStep + 1}/${session.trace.steps.length}`],
    ["kind", step.kind],
    ["sweep_x", step.sweepX.text],
    ["point", pointText],
  ]);

  clearChildren(elements.events);
  appendListItems(elements.events, step.events.length ? step.events : ["（空）"]);

  clearChildren(elements.notes);
  appendListItems(elements.notes, step.notes.length ? step.notes : ["（空）"]);

  clearChildren(elements.active);
  appendListItems(elements.active, step.active.map((id) => String(id)));

  clearChildren(elements.intersections);
  const fragment = document.createDocumentFragment();
  for (const it of step.intersections) {
    const tr = document.createElement("tr");
    const tdA = document.createElement("td");
    tdA.textContent = String(it.a);
    const tdB = document.createElement("td");
    tdB.textContent = String(it.b);
    const tdKind = document.createElement("td");
    tdKind.textContent = it.kind;
    const tdPoint = document.createElement("td");
    tdPoint.textContent = `(${it.point.x.text}, ${it.point.y.text})`;
    tr.appendChild(tdA);
    tr.appendChild(tdB);
    tr.appendChild(tdKind);
    tr.appendChild(tdPoint);
    fragment.appendChild(tr);
  }
  elements.intersections.appendChild(fragment);
}

function updateStepControls() {
  const session = appState.session;
  const stepCount = session.trace.steps.length;
  if (stepCount === 0) {
    elements.stepSlider.min = "0";
    elements.stepSlider.max = "0";
    elements.stepSlider.value = "0";
    elements.stepLabel.textContent = "0/0";
    elements.prevStep.disabled = true;
    elements.nextStep.disabled = true;
    elements.playPause.disabled = true;
    elements.stepSlider.disabled = true;
    return;
  }
  elements.playPause.disabled = false;
  elements.stepSlider.disabled = false;
  elements.stepSlider.min = "0";
  elements.stepSlider.max = String(stepCount - 1);
  elements.stepSlider.value = String(appState.currentStep);
  elements.stepLabel.textContent = `${appState.currentStep + 1}/${stepCount}`;
  elements.prevStep.disabled = appState.currentStep === 0;
  elements.nextStep.disabled = appState.currentStep >= stepCount - 1;
}

function setCurrentStep(index) {
  if (!appState.session) {
    return;
  }
  if (appState.session.trace.steps.length === 0) {
    return;
  }
  const max = appState.session.trace.steps.length - 1;
  const nextIndex = Math.max(0, Math.min(max, index));
  if (nextIndex === appState.currentStep) {
    return;
  }
  appState.currentStep = nextIndex;
  updateStepControls();
  refreshUiForStep();
  appState.render.dirtyDynamic = true;
  requestRender();
}

function togglePlay() {
  if (!appState.session) {
    return;
  }
  if (appState.playing) {
    stopPlay();
  } else {
    startPlay();
  }
}

function startPlay() {
  stopPlay();
  appState.playing = true;
  elements.playPause.textContent = "暂停";
  const baseFps = 10;
  const intervalMs = Math.max(10, Math.floor(1000 / (baseFps * appState.speedFactor)));
  appState.playTimerId = window.setInterval(() => {
    const last = appState.session.trace.steps.length - 1;
    if (appState.currentStep >= last) {
      stopPlay();
      return;
    }
    setCurrentStep(appState.currentStep + 1);
  }, intervalMs);
}

function stopPlay() {
  if (appState.playTimerId !== null) {
    window.clearInterval(appState.playTimerId);
    appState.playTimerId = null;
  }
  appState.playing = false;
  elements.playPause.textContent = "播放";
}

async function loadFromFile(file) {
  stopPlay();
  let json;
  try {
    const text = await file.text();
    json = JSON.parse(text);
  } catch {
    throw new UserError("JSON 解析失败：不是合法 JSON");
  }
  const session = parseSession(json);
  prepareSessionForPlayback(session);
  appState.session = session;
  appState.currentStep = 0;
  setDropHintVisible(false);
  resetView();
  refreshUiForSession(file.name);
  appState.render.dirtyStatic = true;
  appState.render.dirtyDynamic = true;
  requestRender();
  setStatus(`已加载：${file.name}`);
}

function prepareSessionForPlayback(session) {
  const prefixCounts = new Array(session.trace.steps.length);
  const flat = [];
  let count = 0;
  for (let i = 0; i < session.trace.steps.length; i++) {
    const step = session.trace.steps[i];
    for (const it of step.intersections) {
      flat.push(it);
      count++;
    }
    prefixCounts[i] = count;
  }
  session.intersectionsFlat = flat;
  session.intersectionPrefixCounts = prefixCounts;
}

function handleError(error) {
  if (error instanceof UserError) {
    setStatus(`错误：${error.message}`);
    console.error(error);
    return;
  }
  setStatus("发生未知错误，请在控制台查看详情");
  console.error(error);
}

function installEventHandlers() {
  setStatus("未加载数据：请选择或拖拽 session.json");
  setDropHintVisible(true);
  elements.prevStep.disabled = true;
  elements.nextStep.disabled = true;
  elements.playPause.disabled = true;
  elements.stepSlider.disabled = true;

  elements.fileInput.addEventListener("change", async (event) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) {
      return;
    }
    try {
      await loadFromFile(file);
    } catch (error) {
      handleError(error);
    }
  });

  elements.viewport.addEventListener("dragover", (event) => {
    event.preventDefault();
  });

  elements.viewport.addEventListener("drop", async (event) => {
    event.preventDefault();
    const file = event.dataTransfer?.files?.[0];
    if (!file) {
      return;
    }
    try {
      await loadFromFile(file);
    } catch (error) {
      handleError(error);
    }
  });

  elements.resetView.addEventListener("click", () => {
    resetView();
  });

  elements.prevStep.addEventListener("click", () => setCurrentStep(appState.currentStep - 1));
  elements.nextStep.addEventListener("click", () => setCurrentStep(appState.currentStep + 1));
  elements.playPause.addEventListener("click", () => togglePlay());

  elements.speed.addEventListener("change", () => {
    appState.speedFactor = Number(elements.speed.value) || 1;
    if (appState.playing) {
      startPlay();
    }
  });

  elements.stepSlider.addEventListener("input", () => {
    stopPlay();
    setCurrentStep(Number(elements.stepSlider.value));
  });

  window.addEventListener("resize", () => resizeCanvases());
  resizeCanvases();

  elements.viewport.addEventListener("pointerdown", (event) => {
    if (event.button !== 0) {
      return;
    }
    elements.viewport.setPointerCapture(event.pointerId);
    appState.dragging = {
      active: true,
      lastX: event.clientX,
      lastY: event.clientY,
      pointerId: event.pointerId,
    };
  });

  elements.viewport.addEventListener("pointermove", (event) => {
    if (!appState.dragging.active || event.pointerId !== appState.dragging.pointerId) {
      return;
    }
    const dx = event.clientX - appState.dragging.lastX;
    const dy = event.clientY - appState.dragging.lastY;
    appState.dragging.lastX = event.clientX;
    appState.dragging.lastY = event.clientY;
    appState.camera.cx -= dx / appState.camera.zoom;
    appState.camera.cy += dy / appState.camera.zoom;
    appState.render.dirtyStatic = true;
    appState.render.dirtyDynamic = true;
    requestRender();
  });

  function endDrag(event) {
    if (event.pointerId !== appState.dragging.pointerId) {
      return;
    }
    appState.dragging.active = false;
    appState.dragging.pointerId = null;
  }

  elements.viewport.addEventListener("pointerup", endDrag);
  elements.viewport.addEventListener("pointercancel", endDrag);

  elements.viewport.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      const scaleFactor = event.deltaY < 0 ? 1.1 : 1 / 1.1;
      const rect = elements.viewport.getBoundingClientRect();
      const localX = event.clientX - rect.left;
      const localY = event.clientY - rect.top;
      const before = screenToWorld(localX, localY);
      appState.camera.zoom = Math.max(10, Math.min(5000, appState.camera.zoom * scaleFactor));
      const after = screenToWorld(localX, localY);
      appState.camera.cx += before.x - after.x;
      appState.camera.cy += before.y - after.y;
      appState.render.dirtyStatic = true;
      appState.render.dirtyDynamic = true;
      requestRender();
    },
    { passive: false },
  );

  window.addEventListener("keydown", (event) => {
    if (event.target && ["INPUT", "SELECT", "TEXTAREA"].includes(event.target.tagName)) {
      return;
    }
    if (event.code === "Space") {
      event.preventDefault();
      togglePlay();
    } else if (event.code === "ArrowLeft") {
      event.preventDefault();
      stopPlay();
      setCurrentStep(appState.currentStep - 1);
    } else if (event.code === "ArrowRight") {
      event.preventDefault();
      stopPlay();
      setCurrentStep(appState.currentStep + 1);
    }
  });
}

installEventHandlers();
