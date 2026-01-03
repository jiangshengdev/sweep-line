import { createLoaders } from "./controller/loaders.js";
import { createPlaybackController } from "./controller/playback.js";
import { createSettingsController } from "./controller/settings.js";
import { createRenderer } from "./render/renderer.js";
import { createPanels } from "./ui/panels.js";
import { renderSessionPickerList } from "./ui/session-picker.js";
import { renderSessionListInto } from "./ui/session-list.js";

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

const renderer = createRenderer({ elements, appState });
const panels = createPanels({ elements, appState });

function setStatus(message) {
  elements.status.textContent = message;
}

const settings = createSettingsController({ elements, appState, renderer });
const playback = createPlaybackController({ elements, appState, panels, renderer });
const loaders = createLoaders({ appState, setStatus });

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
  renderSessionPicker();
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

function setDropHintVisible(visible) {
  elements.dropHint.classList.toggle("hidden", !visible);
}

function getSessionLoadOptions() {
  return {
    onBeforeApply: () => playback.stopPlay(),
    setDropHintVisible,
    resetView: () => renderer.resetView(),
    refreshUiForSession: (label) => panels.refreshUiForSession(label),
    updateSessionListSelection: () => panels.updateSessionListSelection(),
  };
}

function renderSessionList() {
  renderSessionListInto({
    list: elements.sessionList,
    empty: elements.sessionListEmpty,
    items: appState.index.items,
    currentSource: appState.sessionSource,
    onSelect: (item) => {
      loaders.loadFromUrl(item.path, getSessionLoadOptions()).catch(loaders.handleError);
    },
  });
}

function renderSessionPicker() {
  renderSessionPickerList({
    elements,
    appState,
    onSelect: (item) => {
      loaders
        .loadFromUrl(item.path, getSessionLoadOptions())
        .then(() => {
          closeSessionPicker();
        })
        .catch(loaders.handleError);
    },
  });
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
    renderSessionPicker();
  });

  elements.sessionPicker?.addEventListener("pointerdown", (event) => {
    if (event.target !== elements.sessionPicker) {
      return;
    }
    closeSessionPicker();
  });

  elements.themeMode?.addEventListener("change", () => {
    settings.setThemeMode(elements.themeMode.value);
  });

  elements.showCumulativeIntersections?.addEventListener("change", () => {
    settings.setShowCumulativeIntersections(elements.showCumulativeIntersections.checked);
  });

  elements.cumulativeIntersectionSize?.addEventListener("input", () => {
    settings.setIntersectionRadiusCumulative(elements.cumulativeIntersectionSize.value);
  });

  elements.currentIntersectionSize?.addEventListener("input", () => {
    settings.setIntersectionRadiusCurrent(elements.currentIntersectionSize.value);
  });

  elements.boldActiveSegments?.addEventListener("change", () => {
    settings.setBoldActiveSegments(elements.boldActiveSegments.checked);
  });

  elements.sessionListViewList?.addEventListener("click", () => {
    settings.setSessionListViewMode("list");
  });

  elements.sessionListViewGrid?.addEventListener("click", () => {
    settings.setSessionListViewMode("grid");
  });

  elements.reloadIndex.addEventListener("click", async () => {
    try {
      await loaders.loadIndexAndRenderList({ renderSessionList, renderSessionPicker });
      setStatus("已刷新示例列表");
    } catch (error) {
      loaders.handleError(error);
    }
  });

  elements.fileInput.addEventListener("change", async (event) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) {
      return;
    }
    try {
      await loaders.loadFromFile(file, getSessionLoadOptions());
    } catch (error) {
      loaders.handleError(error);
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
      await loaders.loadFromFile(file, getSessionLoadOptions());
    } catch (error) {
      loaders.handleError(error);
    }
  });

  elements.resetView.addEventListener("click", () => {
    renderer.resetView();
  });

  elements.prevStep.addEventListener("click", () => playback.setCurrentStep(appState.currentStep - 1));
  elements.nextStep.addEventListener("click", () => playback.setCurrentStep(appState.currentStep + 1));
  elements.playPause.addEventListener("click", () => playback.togglePlay());

  elements.speed.addEventListener("change", () => {
    playback.setSpeedFactor(elements.speed.value);
  });

  elements.stepSlider.addEventListener("input", () => {
    playback.stopPlay();
    playback.setCurrentStep(Number(elements.stepSlider.value));
  });

  window.addEventListener("resize", () => renderer.resizeCanvases());
  renderer.resizeCanvases();

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
    renderer.invalidateAll();
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
      const before = renderer.screenToWorld(localX, localY);
      appState.camera.zoom = Math.max(10, Math.min(5000, appState.camera.zoom * scaleFactor));
      const after = renderer.screenToWorld(localX, localY);
      appState.camera.cx += before.x - after.x;
      appState.camera.cy += before.y - after.y;
      renderer.invalidateAll();
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
    playback.handleKeydown(event);
  });
}

settings.init();
installEventHandlers();
loaders
  .loadIndexAndRenderList({ renderSessionList, renderSessionPicker })
  .catch(loaders.handleError);
