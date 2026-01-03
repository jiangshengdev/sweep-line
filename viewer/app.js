class UserError extends Error {}

const elements = {
  fileInput: document.getElementById("file-input"),
  resetView: document.getElementById("reset-view"),
  reloadIndex: document.getElementById("reload-index"),
  openSessionPicker: document.getElementById("open-session-picker"),
  sessionPicker: document.getElementById("session-picker"),
  sessionPickerClose: document.getElementById("session-picker-close"),
  sessionPickerSearch: document.getElementById("session-picker-search"),
  sessionPickerList: document.getElementById("session-picker-list"),
  sessionPickerEmpty: document.getElementById("session-picker-empty"),
  sessionListViewList: document.getElementById("session-list-view-list"),
  sessionListViewGrid: document.getElementById("session-list-view-grid"),
  prevStep: document.getElementById("prev-step"),
  playPause: document.getElementById("play-pause"),
  nextStep: document.getElementById("next-step"),
  speed: document.getElementById("speed"),
  themeMode: document.getElementById("theme-mode"),
  showCumulativeIntersections: document.getElementById("show-cumulative-intersections"),
  cumulativeIntersectionSize: document.getElementById("cumulative-intersection-size"),
  cumulativeIntersectionSizeValue: document.getElementById("cumulative-intersection-size-value"),
  currentIntersectionSize: document.getElementById("current-intersection-size"),
  currentIntersectionSizeValue: document.getElementById("current-intersection-size-value"),
  boldActiveSegments: document.getElementById("bold-active-segments"),
  stepSlider: document.getElementById("step-slider"),
  stepLabel: document.getElementById("step-label"),
  status: document.getElementById("status"),
  dropHint: document.getElementById("drop-hint"),
  viewport: document.getElementById("viewport"),
  sessionList: document.getElementById("session-list"),
  sessionListEmpty: document.getElementById("session-list-empty"),
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
  sessionSource: null,
  currentStep: 0,
  playing: false,
  playTimerId: null,
  speedFactor: 1,
  settings: {
    themeMode: "system",
    showCumulativeIntersections: true,
    intersectionRadiusCumulative: 2,
    intersectionRadiusCurrent: 3.5,
    boldActiveSegments: false,
    sessionListViewMode: "list",
  },
  ui: {
    sessionPickerOpen: false,
    sessionPickerQuery: "",
    sessionPickerReturnFocus: null,
  },
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
  index: {
    items: [],
  },
};

function setStatus(message) {
  elements.status.textContent = message;
}

const storageKeys = {
  themeMode: "traceViewer.themeMode",
  showCumulativeIntersections: "traceViewer.showCumulativeIntersections",
  intersectionRadiusCumulative: "traceViewer.intersectionRadiusCumulative",
  intersectionRadiusCurrent: "traceViewer.intersectionRadiusCurrent",
  boldActiveSegments: "traceViewer.boldActiveSegments",
  sessionListViewMode: "traceViewer.sessionListViewMode",
};

function safeStorageGetItem(key) {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function safeStorageSetItem(key, value) {
  try {
    window.localStorage.setItem(key, value);
  } catch {
    // localStorage 不可用时保持静默：不影响主要功能
  }
}

function clampNumber(value, min, max) {
  return Math.min(max, Math.max(min, value));
}

function roundToHalf(value) {
  return Math.round(value * 2) / 2;
}

function formatSizeValue(value) {
  if (!Number.isFinite(value)) {
    return "-";
  }
  const rounded = roundToHalf(value);
  const text = String(rounded);
  return text.endsWith(".0") ? text.slice(0, -2) : text;
}

function loadSettingsFromStorage() {
  const themeMode = safeStorageGetItem(storageKeys.themeMode);
  if (themeMode === "system" || themeMode === "light" || themeMode === "dark") {
    appState.settings.themeMode = themeMode;
  }

  const sessionListViewMode = safeStorageGetItem(storageKeys.sessionListViewMode);
  if (sessionListViewMode === "list" || sessionListViewMode === "grid") {
    appState.settings.sessionListViewMode = sessionListViewMode;
  }

  const show = safeStorageGetItem(storageKeys.showCumulativeIntersections);
  if (show === "true") {
    appState.settings.showCumulativeIntersections = true;
  } else if (show === "false") {
    appState.settings.showCumulativeIntersections = false;
  }

  const cumulativeSizeText = safeStorageGetItem(storageKeys.intersectionRadiusCumulative);
  if (cumulativeSizeText !== null) {
    const cumulativeSize = Number(cumulativeSizeText);
    if (Number.isFinite(cumulativeSize)) {
      appState.settings.intersectionRadiusCumulative = roundToHalf(
        clampNumber(cumulativeSize, 0.5, 6),
      );
    }
  }

  const currentSizeText = safeStorageGetItem(storageKeys.intersectionRadiusCurrent);
  if (currentSizeText !== null) {
    const currentSize = Number(currentSizeText);
    if (Number.isFinite(currentSize)) {
      appState.settings.intersectionRadiusCurrent = roundToHalf(
        clampNumber(currentSize, 0.5, 10),
      );
    }
  }

  const boldActive = safeStorageGetItem(storageKeys.boldActiveSegments);
  if (boldActive === "true") {
    appState.settings.boldActiveSegments = true;
  } else if (boldActive === "false") {
    appState.settings.boldActiveSegments = false;
  }
}

function applyThemeMode(themeMode) {
  const root = document.documentElement;
  if (themeMode === "system") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", themeMode);
  }
}

function applySessionListViewMode(viewMode) {
  if (!elements.sessionList) {
    return;
  }
  elements.sessionList.classList.toggle("session-list--grid", viewMode === "grid");
}

function applySettingsToUi() {
  if (elements.themeMode) {
    elements.themeMode.value = appState.settings.themeMode;
  }
  if (elements.sessionListViewList && elements.sessionListViewGrid) {
    const listActive = appState.settings.sessionListViewMode === "list";
    elements.sessionListViewList.classList.toggle("segmented__button--active", listActive);
    elements.sessionListViewList.setAttribute("aria-pressed", String(listActive));

    const gridActive = appState.settings.sessionListViewMode === "grid";
    elements.sessionListViewGrid.classList.toggle("segmented__button--active", gridActive);
    elements.sessionListViewGrid.setAttribute("aria-pressed", String(gridActive));
  }
  if (elements.showCumulativeIntersections) {
    elements.showCumulativeIntersections.checked = appState.settings.showCumulativeIntersections;
  }
  if (elements.cumulativeIntersectionSize) {
    elements.cumulativeIntersectionSize.value = String(appState.settings.intersectionRadiusCumulative);
  }
  if (elements.cumulativeIntersectionSizeValue) {
    elements.cumulativeIntersectionSizeValue.textContent = formatSizeValue(
      appState.settings.intersectionRadiusCumulative,
    );
  }
  if (elements.currentIntersectionSize) {
    elements.currentIntersectionSize.value = String(appState.settings.intersectionRadiusCurrent);
  }
  if (elements.currentIntersectionSizeValue) {
    elements.currentIntersectionSizeValue.textContent = formatSizeValue(
      appState.settings.intersectionRadiusCurrent,
    );
  }
  if (elements.boldActiveSegments) {
    elements.boldActiveSegments.checked = appState.settings.boldActiveSegments;
  }

  applySessionListViewMode(appState.settings.sessionListViewMode);
}

function setThemeMode(nextMode) {
  if (nextMode !== "system" && nextMode !== "light" && nextMode !== "dark") {
    return;
  }
  if (appState.settings.themeMode === nextMode) {
    return;
  }
  appState.settings.themeMode = nextMode;
  safeStorageSetItem(storageKeys.themeMode, nextMode);
  applyThemeMode(nextMode);
  applySettingsToUi();
  appState.render.dirtyStatic = true;
  appState.render.dirtyDynamic = true;
  requestRender();
}

function setShowCumulativeIntersections(enabled) {
  const next = Boolean(enabled);
  if (appState.settings.showCumulativeIntersections === next) {
    return;
  }
  appState.settings.showCumulativeIntersections = next;
  safeStorageSetItem(storageKeys.showCumulativeIntersections, String(next));
  applySettingsToUi();
  appState.render.dirtyDynamic = true;
  requestRender();
}

function setIntersectionRadiusCumulative(value) {
  const next = roundToHalf(clampNumber(Number(value) || 0, 0.5, 6));
  if (appState.settings.intersectionRadiusCumulative === next) {
    return;
  }
  appState.settings.intersectionRadiusCumulative = next;
  safeStorageSetItem(storageKeys.intersectionRadiusCumulative, String(next));
  applySettingsToUi();
  appState.render.dirtyDynamic = true;
  requestRender();
}

function setIntersectionRadiusCurrent(value) {
  const next = roundToHalf(clampNumber(Number(value) || 0, 0.5, 10));
  if (appState.settings.intersectionRadiusCurrent === next) {
    return;
  }
  appState.settings.intersectionRadiusCurrent = next;
  safeStorageSetItem(storageKeys.intersectionRadiusCurrent, String(next));
  applySettingsToUi();
  appState.render.dirtyDynamic = true;
  requestRender();
}

function setBoldActiveSegments(enabled) {
  const next = Boolean(enabled);
  if (appState.settings.boldActiveSegments === next) {
    return;
  }
  appState.settings.boldActiveSegments = next;
  safeStorageSetItem(storageKeys.boldActiveSegments, String(next));
  applySettingsToUi();
  appState.render.dirtyDynamic = true;
  requestRender();
}

function setSessionPickerVisible(visible) {
  if (!elements.sessionPicker) {
    return;
  }
  const next = Boolean(visible);
  elements.sessionPicker.classList.toggle("hidden", !next);
  elements.sessionPicker.setAttribute("aria-hidden", String(!next));
  appState.ui.sessionPickerOpen = next;
}

function openSessionPicker() {
  if (!elements.sessionPicker) {
    return;
  }
  if (appState.ui.sessionPickerOpen) {
    return;
  }
  appState.ui.sessionPickerReturnFocus =
    document.activeElement && document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;

  appState.ui.sessionPickerQuery = "";
  if (elements.sessionPickerSearch) {
    elements.sessionPickerSearch.value = "";
  }

  setSessionPickerVisible(true);
  renderSessionPickerList();
  if (elements.sessionPickerSearch) {
    elements.sessionPickerSearch.focus();
    elements.sessionPickerSearch.select?.();
  }
}

function closeSessionPicker() {
  if (!elements.sessionPicker) {
    return;
  }
  if (!appState.ui.sessionPickerOpen) {
    return;
  }
  setSessionPickerVisible(false);
  if (appState.ui.sessionPickerReturnFocus) {
    appState.ui.sessionPickerReturnFocus.focus?.();
  }
  appState.ui.sessionPickerReturnFocus = null;
}

function toggleSessionPicker() {
  if (appState.ui.sessionPickerOpen) {
    closeSessionPicker();
  } else {
    openSessionPicker();
  }
}

function setSessionListViewMode(nextMode) {
  if (nextMode !== "list" && nextMode !== "grid") {
    return;
  }
  if (appState.settings.sessionListViewMode === nextMode) {
    return;
  }
  appState.settings.sessionListViewMode = nextMode;
  safeStorageSetItem(storageKeys.sessionListViewMode, nextMode);
  applySettingsToUi();
}

function getCanvasPalette() {
  const styles = getComputedStyle(document.documentElement);
  const read = (name, fallback) => {
    const value = styles.getPropertyValue(name).trim();
    return value || fallback;
  };
  return {
    accent: read("--accent", "#77b3ff"),
    text: read("--text", "#e7edf6"),
    muted: read("--muted", "#9aa4b2"),
    ok: read("--ok", "#6ee7b7"),
    danger: read("--danger", "#ff6b6b"),
    canvasOutline: read("--canvas-outline", "rgba(255, 255, 255, 0.62)"),
  };
}

function setDropHintVisible(visible) {
  elements.dropHint.classList.toggle("hidden", !visible);
}

function clearChildren(node) {
  while (node.firstChild) {
    node.removeChild(node.firstChild);
  }
}

function folderKeyForSessionIndexItem(item) {
  const rawPath = String(item?.path || "").replace(/\\/g, "/");
  const parts = rawPath.split("/").filter(Boolean);
  if (parts.length < 3) {
    return "";
  }
  return parts[1];
}

function groupSessionIndexItemsForRender(items) {
  const groups = [];
  const groupByTitle = new Map();

  for (const item of items) {
    const groupTitle = String(item?.groupTitle || "");
    let group = groupByTitle.get(groupTitle);
    if (!group) {
      group = {
        title: groupTitle,
        folders: [],
        folderMap: new Map(),
      };
      groups.push(group);
      groupByTitle.set(groupTitle, group);
    }

    const folderKey = folderKeyForSessionIndexItem(item);
    let folderGroup = group.folderMap.get(folderKey);
    if (!folderGroup) {
      folderGroup = { key: folderKey, items: [] };
      group.folders.push(folderGroup);
      group.folderMap.set(folderKey, folderGroup);
    }
    folderGroup.items.push(item);
  }

  for (const group of groups) {
    delete group.folderMap;
  }

  return groups;
}

function normalizeSearchQuery(query) {
  return String(query || "").trim().toLowerCase();
}

function sessionIndexItemMatchesQuery(item, normalizedQuery) {
  if (!normalizedQuery) {
    return true;
  }
  const haystacks = [
    item.title,
    item.id,
    item.groupTitle,
    ...(Array.isArray(item.tags) ? item.tags : []),
  ];
  for (const raw of haystacks) {
    if (!raw) {
      continue;
    }
    if (String(raw).toLowerCase().includes(normalizedQuery)) {
      return true;
    }
  }
  return false;
}

function filterSessionIndexItems(items, query) {
  const normalized = normalizeSearchQuery(query);
  if (!normalized) {
    return items;
  }
  return items.filter((item) => sessionIndexItemMatchesQuery(item, normalized));
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

  const palette = getCanvasPalette();
  const step = session.trace.steps[appState.currentStep];
  if (!step) {
    return;
  }

  const sweepXWorld = step.sweepX.value / session.scale;
  if (Number.isFinite(sweepXWorld)) {
    const sweepA = worldToCanvas(sweepXWorld, -1e9);
    const sweepB = worldToCanvas(sweepXWorld, 1e9);
    ctx.save();
    ctx.strokeStyle = palette.accent;
    ctx.lineWidth = 1.5 * appState.viewport.dpr;
    ctx.globalAlpha = 0.75;
    const dash = 6 * appState.viewport.dpr;
    ctx.setLineDash([dash, dash]);
    ctx.beginPath();
    ctx.moveTo(sweepA.x, sweepA.y);
    ctx.lineTo(sweepB.x, sweepB.y);
    ctx.stroke();
    ctx.restore();
  }

  const activeSet = new Set(step.active);
  ctx.save();
  const baseActiveWidthCss = 1.25;
  const activeWidthCss = appState.settings.boldActiveSegments ? 3 : baseActiveWidthCss;
  ctx.lineWidth = activeWidthCss * appState.viewport.dpr;
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
        drawVerticalCaps(ctx, p.ax, yMinWorld, yMaxWorld, palette);
      }
    }
    ctx.restore();
  }

  if (appState.settings.showCumulativeIntersections) {
    const currentIndex = appState.currentStep;
    const end = currentIndex > 0 ? (session.intersectionPrefixCounts[currentIndex - 1] ?? 0) : 0;
    const intersectionsToDraw = session.intersectionsFlat.slice(0, end);
    drawIntersections(ctx, intersectionsToDraw, session.scale, false, palette);
  }
  drawIntersections(ctx, step.intersections, session.scale, true, palette);

  if (step.point) {
    const p = pointRatToWorld(step.point, session.scale);
    ctx.save();
    ctx.globalAlpha = 0.9;
    const sizeCss = Math.max(12, appState.settings.intersectionRadiusCurrent * 4 + 6);
    drawCrosshair(ctx, p.x, p.y, sizeCss, palette.accent, 1.5);
    ctx.restore();
  }
}

function drawVerticalCaps(ctx, worldX, yMinWorld, yMaxWorld, palette) {
  const cap = 6 * appState.viewport.dpr;
  const pMin = worldToCanvas(worldX, yMinWorld);
  const pMax = worldToCanvas(worldX, yMaxWorld);
  ctx.save();
  ctx.strokeStyle = palette.text;
  ctx.globalAlpha = 0.7;
  ctx.lineWidth = 1.5 * appState.viewport.dpr;
  ctx.beginPath();
  ctx.moveTo(pMin.x - cap, pMin.y);
  ctx.lineTo(pMin.x + cap, pMin.y);
  ctx.moveTo(pMax.x - cap, pMax.y);
  ctx.lineTo(pMax.x + cap, pMax.y);
  ctx.stroke();
  ctx.restore();
}

function drawIntersections(ctx, intersections, scale, isCurrentStep, palette) {
  const radius = isCurrentStep
    ? appState.settings.intersectionRadiusCurrent
    : appState.settings.intersectionRadiusCumulative;
  const baseStrokeWidth = isCurrentStep ? 1.5 : 1.25;
  const strokeWidth = Math.min(baseStrokeWidth, radius);
  const alpha = isCurrentStep ? 1.0 : 0.85;
  ctx.save();
  ctx.globalAlpha = alpha;
  for (const it of intersections) {
    const p = pointRatToWorld(it.point, scale);
    if (!Number.isFinite(p.x) || !Number.isFinite(p.y)) {
      continue;
    }
    const color = it.kind === "Proper" ? palette.ok : palette.danger;
    drawPoint(ctx, p.x, p.y, radius, color, palette.canvasOutline, strokeWidth);
  }
  ctx.restore();
}

function drawPoint(ctx, worldX, worldY, radiusCss, fillStyle, strokeStyle, strokeWidthCss) {
  const p = worldToCanvas(worldX, worldY);
  const radius = radiusCss * appState.viewport.dpr;
  ctx.save();
  ctx.fillStyle = fillStyle;
  if (strokeStyle) {
    ctx.strokeStyle = strokeStyle;
    ctx.lineWidth = (strokeWidthCss ?? 1) * appState.viewport.dpr;
  }
  ctx.beginPath();
  ctx.arc(p.x, p.y, radius, 0, Math.PI * 2);
  ctx.fill();
  if (strokeStyle) {
    ctx.stroke();
  }
  ctx.restore();
}

function drawCrosshair(ctx, worldX, worldY, sizeCss, strokeStyle, lineWidthCss) {
  const p = worldToCanvas(worldX, worldY);
  const dpr = appState.viewport.dpr;
  const size = sizeCss * dpr;
  const half = size / 2;
  ctx.save();
  ctx.strokeStyle = strokeStyle;
  ctx.lineWidth = (lineWidthCss ?? 1) * dpr;
  ctx.beginPath();
  ctx.moveTo(p.x - half, p.y);
  ctx.lineTo(p.x + half, p.y);
  ctx.moveTo(p.x, p.y - half);
  ctx.lineTo(p.x, p.y + half);
  ctx.stroke();
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
  appState.sessionSource = null;
  appState.currentStep = 0;
  setDropHintVisible(false);
  resetView();
  refreshUiForSession(file.name);
  appState.render.dirtyStatic = true;
  appState.render.dirtyDynamic = true;
  requestRender();
  updateSessionListSelection();
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

function parseSessionIndex(value, path) {
  const obj = parseObject(value, path);
  const schema = parseString(obj.schema, `${path}.schema`);
  if (schema !== "session-index.v1") {
    throw new UserError(`${path}.schema 不是 session-index.v1`);
  }
  const rawItems = parseArray(obj.items, `${path}.items`);
  const items = rawItems.map((v, i) => {
    const itemPath = `${path}.items[${i}]`;
    const itemObj = parseObject(v, itemPath);
    return {
      id: parseString(itemObj.id, `${itemPath}.id`),
      title: parseString(itemObj.title, `${itemPath}.title`),
      path: parseString(itemObj.path, `${itemPath}.path`),
      tags: parseArray(itemObj.tags, `${itemPath}.tags`).map((t, j) =>
        parseString(t, `${itemPath}.tags[${j}]`),
      ),
      segments: parseInteger(itemObj.segments, `${itemPath}.segments`),
      steps: parseInteger(itemObj.steps, `${itemPath}.steps`),
      warnings: parseInteger(itemObj.warnings, `${itemPath}.warnings`),
    };
  });
  return { schema, items };
}

async function tryLoadIndexFile(url, groupTitle) {
  let res;
  try {
    res = await fetch(url, { cache: "no-store" });
  } catch {
    return null;
  }
  if (!res.ok) {
    return null;
  }

  let json;
  try {
    json = await res.json();
  } catch {
    throw new UserError(`索引解析失败：${url} 不是合法 JSON`);
  }

  const index = parseSessionIndex(json, "$");
  return index.items.map((item) => ({ ...item, groupTitle }));
}

async function loadIndexAndRenderList() {
  const items = [];
  const generated = await tryLoadIndexFile("./generated/index.json", "Rust 生成");
  if (generated) {
    items.push(...generated);
  }
  const examples = await tryLoadIndexFile("./examples/index.json", "内置示例");
  if (examples) {
    items.push(...examples);
  }
  appState.index.items = items;
  renderSessionList();
  renderSessionPickerList();
}

function renderSessionListInto({ list, empty, items, currentSource, onSelect }) {
  if (!list) {
    return;
  }
  clearChildren(list);

  if (!items.length) {
    if (empty) {
      empty.hidden = false;
    }
    return;
  }
  if (empty) {
    empty.hidden = true;
  }

  const fragment = document.createDocumentFragment();
  const groups = groupSessionIndexItemsForRender(items);
  for (const group of groups) {
    if (group.title) {
      const groupLi = document.createElement("li");
      groupLi.className = "session-list__group";
      groupLi.textContent = group.title;
      fragment.appendChild(groupLi);
    }

    for (const folder of group.folders) {
      const sectionLi = document.createElement("li");
      sectionLi.className = "session-list__folder";

      const folderTitle = document.createElement("div");
      folderTitle.className = "session-list__group session-list__group--folder mono";
      folderTitle.textContent = folder.key ? `${folder.key}/` : "（根目录）";
      sectionLi.appendChild(folderTitle);

      const folderList = document.createElement("ul");
      folderList.className = "session-list session-list__folder-list";
      for (const item of folder.items) {
        const li = document.createElement("li");
        const button = document.createElement("button");
        button.type = "button";
        button.className = "session-list__item";
        button.dataset.path = item.path;
        button.classList.toggle(
          "session-list__item--active",
          currentSource && item.path === currentSource,
        );

        const title = document.createElement("div");
        title.className = "session-item__title";
        title.textContent = item.title;

        const meta = document.createElement("div");
        meta.className = "session-item__meta mono";
        meta.textContent = `segments=${item.segments} steps=${item.steps} warnings=${item.warnings}`;

        button.appendChild(title);
        button.appendChild(meta);
        button.addEventListener("click", () => {
          onSelect(item);
        });

        li.appendChild(button);
        folderList.appendChild(li);
      }

      sectionLi.appendChild(folderList);
      fragment.appendChild(sectionLi);
    }
  }

  list.appendChild(fragment);
}

function renderSessionList() {
  renderSessionListInto({
    list: elements.sessionList,
    empty: elements.sessionListEmpty,
    items: appState.index.items,
    currentSource: appState.sessionSource,
    onSelect: (item) => {
      loadFromUrl(item.path).catch(handleError);
    },
  });
}

function renderSessionPickerList() {
  if (!elements.sessionPickerList || !elements.sessionPickerEmpty) {
    return;
  }

  const allItems = appState.index.items;
  const query = appState.ui.sessionPickerQuery;
  const filtered = filterSessionIndexItems(allItems, query);

  if (!allItems.length) {
    elements.sessionPickerEmpty.textContent =
      "未找到索引：请运行 pnpm gen:sessions 生成 viewer/generated/index.json，或手动加载 session.json。";
  } else if (!filtered.length && normalizeSearchQuery(query)) {
    elements.sessionPickerEmpty.textContent = "无匹配结果";
  } else {
    elements.sessionPickerEmpty.textContent = "";
  }

  renderSessionListInto({
    list: elements.sessionPickerList,
    empty: elements.sessionPickerEmpty,
    items: filtered,
    currentSource: appState.sessionSource,
    onSelect: (item) => {
      loadFromUrl(item.path)
        .then(() => {
          closeSessionPicker();
        })
        .catch(handleError);
    },
  });
}

function updateSessionListSelectionIn(container, current) {
  if (!container) {
    return;
  }
  const buttons = container.querySelectorAll("button.session-list__item");
  for (const btn of buttons) {
    btn.classList.toggle("session-list__item--active", current && btn.dataset.path === current);
  }
}

function updateSessionListSelection() {
  const current = appState.sessionSource;
  updateSessionListSelectionIn(elements.sessionList, current);
  updateSessionListSelectionIn(elements.sessionPickerList, current);
}

async function loadFromUrl(url) {
  stopPlay();
  let res;
  try {
    res = await fetch(url, { cache: "no-store" });
  } catch {
    throw new UserError(`加载失败：无法请求 ${url}`);
  }
  if (!res.ok) {
    throw new UserError(`加载失败：${url}（HTTP ${res.status}）`);
  }

  let json;
  try {
    json = await res.json();
  } catch {
    throw new UserError(`加载失败：${url} 不是合法 JSON`);
  }

  const session = parseSession(json);
  prepareSessionForPlayback(session);
  appState.session = session;
  appState.sessionSource = url;
  appState.currentStep = 0;
  setDropHintVisible(false);
  resetView();
  refreshUiForSession(url);
  appState.render.dirtyStatic = true;
  appState.render.dirtyDynamic = true;
  requestRender();
  updateSessionListSelection();
  setStatus(`已加载：${url}`);
}

function installEventHandlers() {
  setStatus("未加载数据：请选择或拖拽 session.json");
  setDropHintVisible(true);
  elements.prevStep.disabled = true;
  elements.nextStep.disabled = true;
  elements.playPause.disabled = true;
  elements.stepSlider.disabled = true;

  elements.openSessionPicker?.addEventListener("click", () => {
    toggleSessionPicker();
  });

  elements.sessionPickerClose?.addEventListener("click", () => {
    closeSessionPicker();
  });

  elements.sessionPickerSearch?.addEventListener("input", () => {
    appState.ui.sessionPickerQuery = elements.sessionPickerSearch.value;
    renderSessionPickerList();
  });

  elements.sessionPicker?.addEventListener("pointerdown", (event) => {
    if (event.target !== elements.sessionPicker) {
      return;
    }
    closeSessionPicker();
  });

  elements.themeMode?.addEventListener("change", () => {
    setThemeMode(elements.themeMode.value);
  });

  elements.showCumulativeIntersections?.addEventListener("change", () => {
    setShowCumulativeIntersections(elements.showCumulativeIntersections.checked);
  });

  elements.cumulativeIntersectionSize?.addEventListener("input", () => {
    setIntersectionRadiusCumulative(elements.cumulativeIntersectionSize.value);
  });

  elements.currentIntersectionSize?.addEventListener("input", () => {
    setIntersectionRadiusCurrent(elements.currentIntersectionSize.value);
  });

  elements.boldActiveSegments?.addEventListener("change", () => {
    setBoldActiveSegments(elements.boldActiveSegments.checked);
  });

  elements.sessionListViewList?.addEventListener("click", () => {
    setSessionListViewMode("list");
  });

  elements.sessionListViewGrid?.addEventListener("click", () => {
    setSessionListViewMode("grid");
  });

  elements.reloadIndex.addEventListener("click", async () => {
    try {
      await loadIndexAndRenderList();
      setStatus("已刷新示例列表");
    } catch (error) {
      handleError(error);
    }
  });

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
    if (appState.ui.sessionPickerOpen) {
      if (event.code === "Escape") {
        event.preventDefault();
        closeSessionPicker();
      }
      return;
    }

    if ((event.ctrlKey || event.metaKey) && event.code === "KeyK") {
      event.preventDefault();
      openSessionPicker();
      return;
    }

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

loadSettingsFromStorage();
applyThemeMode(appState.settings.themeMode);
applySettingsToUi();

window
  .matchMedia("(prefers-color-scheme: dark)")
  .addEventListener("change", () => {
    if (appState.settings.themeMode !== "system") {
      return;
    }
    appState.render.dirtyStatic = true;
    appState.render.dirtyDynamic = true;
    requestRender();
  });

installEventHandlers();
loadIndexAndRenderList().catch(handleError);
