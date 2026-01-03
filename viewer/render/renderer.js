// @ts-check

/**
 * @typedef {{ x: number, y: number }} Vec2
 */

/**
 * @param {unknown} value
 * @returns {boolean}
 */
function isFiniteNumber(value) {
  return typeof value === "number" && Number.isFinite(value);
}

/**
 * @param {any} seg
 * @param {number} scale
 */
function segmentToWorldPoints(seg, scale) {
  return {
    ax: seg.a.x / scale,
    ay: seg.a.y / scale,
    bx: seg.b.x / scale,
    by: seg.b.y / scale,
  };
}

/**
 * @param {any} point
 * @param {number} scale
 */
function pointRatToWorld(point, scale) {
  return {
    x: point.x.value / scale,
    y: point.y.value / scale,
  };
}

/**
 * @param {any} session
 */
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

/**
 * @returns {{ accent: string, text: string, muted: string, ok: string, danger: string, canvasOutline: string }}
 */
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

/**
 * @param {string[]} events
 * @returns {number[]}
 */
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

/**
 * @param {string[]} notes
 * @returns {Map<number, { yMinFixed: number, yMaxFixed: number }>}
 */
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

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {Vec2} a
 * @param {Vec2} b
 */
function strokeLine(ctx, a, b) {
  ctx.beginPath();
  ctx.moveTo(a.x, a.y);
  ctx.lineTo(b.x, b.y);
  ctx.stroke();
}

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {number} worldX
 * @param {number} yMinWorld
 * @param {number} yMaxWorld
 * @param {{ text: string }} palette
 * @param {(x: number, y: number) => Vec2} worldToCanvas
 * @param {number} dpr
 */
function drawVerticalCaps(ctx, worldX, yMinWorld, yMaxWorld, palette, worldToCanvas, dpr) {
  const cap = 6 * dpr;
  const pMin = worldToCanvas(worldX, yMinWorld);
  const pMax = worldToCanvas(worldX, yMaxWorld);
  ctx.save();
  ctx.strokeStyle = palette.text;
  ctx.globalAlpha = 0.7;
  ctx.lineWidth = 1.5 * dpr;
  ctx.beginPath();
  ctx.moveTo(pMin.x - cap, pMin.y);
  ctx.lineTo(pMin.x + cap, pMin.y);
  ctx.moveTo(pMax.x - cap, pMax.y);
  ctx.lineTo(pMax.x + cap, pMax.y);
  ctx.stroke();
  ctx.restore();
}

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {number} worldX
 * @param {number} worldY
 * @param {number} radiusCss
 * @param {string} fillStyle
 * @param {string | null} strokeStyle
 * @param {number | null | undefined} strokeWidthCss
 * @param {(x: number, y: number) => Vec2} worldToCanvas
 * @param {number} dpr
 */
function drawPoint(ctx, worldX, worldY, radiusCss, fillStyle, strokeStyle, strokeWidthCss, worldToCanvas, dpr) {
  const p = worldToCanvas(worldX, worldY);
  const radius = radiusCss * dpr;
  ctx.save();
  ctx.fillStyle = fillStyle;
  if (strokeStyle) {
    ctx.strokeStyle = strokeStyle;
    ctx.lineWidth = (strokeWidthCss ?? 1) * dpr;
  }
  ctx.beginPath();
  ctx.arc(p.x, p.y, radius, 0, Math.PI * 2);
  ctx.fill();
  if (strokeStyle) {
    ctx.stroke();
  }
  ctx.restore();
}

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {number} worldX
 * @param {number} worldY
 * @param {number} sizeCss
 * @param {string} strokeStyle
 * @param {number | null | undefined} lineWidthCss
 * @param {(x: number, y: number) => Vec2} worldToCanvas
 * @param {number} dpr
 */
function drawCrosshair(ctx, worldX, worldY, sizeCss, strokeStyle, lineWidthCss, worldToCanvas, dpr) {
  const p = worldToCanvas(worldX, worldY);
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

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {any[]} intersections
 * @param {number} scale
 * @param {boolean} isCurrentStep
 * @param {any} appState
 * @param {{ ok: string, danger: string, canvasOutline: string }} palette
 * @param {(x: number, y: number) => Vec2} worldToCanvas
 * @param {number} dpr
 */
function drawIntersections(ctx, intersections, scale, isCurrentStep, appState, palette, worldToCanvas, dpr) {
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
    drawPoint(ctx, p.x, p.y, radius, color, palette.canvasOutline, strokeWidth, worldToCanvas, dpr);
  }
  ctx.restore();
}

/**
 * @param {{ elements: any, appState: any }} deps
 */
export function createRenderer({ elements, appState }) {
  /**
   * @param {number} x
   * @param {number} y
   * @returns {Vec2}
   */
  function worldToCanvas(x, y) {
    const { widthCss, heightCss, dpr } = appState.viewport;
    const { cx, cy, zoom } = appState.camera;
    const px = (x - cx) * zoom + widthCss / 2;
    const py = (-(y - cy) * zoom) + heightCss / 2;
    return { x: px * dpr, y: py * dpr };
  }

  /**
   * 将 viewport 内的 CSS 像素坐标转换为 world 坐标。
   * @param {number} localX
   * @param {number} localY
   */
  function screenToWorld(localX, localY) {
    const { widthCss, heightCss } = appState.viewport;
    const { cx, cy, zoom } = appState.camera;
    const x = (localX - widthCss / 2) / zoom + cx;
    const y = -((localY - heightCss / 2) / zoom) + cy;
    return { x, y };
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
      strokeLine(ctx, a, b);
    }
    ctx.restore();
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
    if (isFiniteNumber(sweepXWorld)) {
      const sweepA = worldToCanvas(sweepXWorld, -1e9);
      const sweepB = worldToCanvas(sweepXWorld, 1e9);
      ctx.save();
      ctx.strokeStyle = palette.accent;
      ctx.lineWidth = 1.5 * appState.viewport.dpr;
      ctx.globalAlpha = 0.75;
      const dash = 6 * appState.viewport.dpr;
      ctx.setLineDash([dash, dash]);
      strokeLine(ctx, sweepA, sweepB);
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
      strokeLine(ctx, a, b);
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
        strokeLine(ctx, a, b);

        const range = ranges.get(id);
        if (range && isFiniteNumber(p.ax)) {
          const yMinWorld = range.yMinFixed / session.scale;
          const yMaxWorld = range.yMaxFixed / session.scale;
          drawVerticalCaps(
            ctx,
            p.ax,
            yMinWorld,
            yMaxWorld,
            palette,
            worldToCanvas,
            appState.viewport.dpr,
          );
        }
      }
      ctx.restore();
    }

    if (appState.settings.showCumulativeIntersections) {
      const currentIndex = appState.currentStep;
      const end =
        currentIndex > 0 ? (session.intersectionPrefixCounts[currentIndex - 1] ?? 0) : 0;
      const intersectionsToDraw = session.intersectionsFlat.slice(0, end);
      drawIntersections(
        ctx,
        intersectionsToDraw,
        session.scale,
        false,
        appState,
        palette,
        worldToCanvas,
        appState.viewport.dpr,
      );
    }
    drawIntersections(
      ctx,
      step.intersections,
      session.scale,
      true,
      appState,
      palette,
      worldToCanvas,
      appState.viewport.dpr,
    );

    if (step.point) {
      const p = pointRatToWorld(step.point, session.scale);
      ctx.save();
      ctx.globalAlpha = 0.9;
      const sizeCss = Math.max(12, appState.settings.intersectionRadiusCurrent * 4 + 6);
      drawCrosshair(
        ctx,
        p.x,
        p.y,
        sizeCss,
        palette.accent,
        1.5,
        worldToCanvas,
        appState.viewport.dpr,
      );
      ctx.restore();
    }
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

  function invalidateStatic() {
    appState.render.dirtyStatic = true;
    requestRender();
  }

  function invalidateDynamic() {
    appState.render.dirtyDynamic = true;
    requestRender();
  }

  function invalidateAll() {
    appState.render.dirtyStatic = true;
    appState.render.dirtyDynamic = true;
    requestRender();
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
    invalidateAll();
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
    invalidateAll();
  }

  return {
    invalidateStatic,
    invalidateDynamic,
    invalidateAll,
    requestRender,
    renderIfDirty,
    resizeCanvases,
    resetView,
    screenToWorld,
  };
}

