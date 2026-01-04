// @ts-check

import { formatSizeValue } from "../lib/format.js";
import { clampNumber, roundToHalf } from "../lib/numbers.js";
import { safeStorageGetItem, safeStorageSetItem } from "../lib/storage.js";

const storageKeys = {
  themeMode: "traceViewer.themeMode",
  showCumulativeIntersections: "traceViewer.showCumulativeIntersections",
  intersectionRadiusCumulative: "traceViewer.intersectionRadiusCumulative",
  intersectionRadiusCurrent: "traceViewer.intersectionRadiusCurrent",
  boldActiveSegments: "traceViewer.boldActiveSegments",
  sessionListViewMode: "traceViewer.sessionListViewMode",
};

/**
 * @param {"system" | "light" | "dark"} themeMode
 */
function applyThemeMode(themeMode) {
  const root = document.documentElement;
  if (themeMode === "system") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", themeMode);
  }
}

/**
 * @param {any} elements
 * @param {"list" | "grid"} viewMode
 */
function applySessionListViewMode(elements, viewMode) {
  if (!elements.sessionList) {
    return;
  }
  elements.sessionList.classList.toggle(
    "session-list--grid",
    viewMode === "grid",
  );
}

/**
 * @param {{ elements: any, appState: any, renderer: any }} deps
 */
export function createSettingsController({ elements, appState, renderer }) {
  function loadSettingsFromStorage() {
    const themeMode = safeStorageGetItem(storageKeys.themeMode);
    if (
      themeMode === "system" ||
      themeMode === "light" ||
      themeMode === "dark"
    ) {
      appState.settings.themeMode = themeMode;
    }

    const sessionListViewMode = safeStorageGetItem(
      storageKeys.sessionListViewMode,
    );
    if (sessionListViewMode === "list" || sessionListViewMode === "grid") {
      appState.settings.sessionListViewMode = sessionListViewMode;
    }

    const show = safeStorageGetItem(storageKeys.showCumulativeIntersections);
    if (show === "true") {
      appState.settings.showCumulativeIntersections = true;
    } else if (show === "false") {
      appState.settings.showCumulativeIntersections = false;
    }

    const cumulativeSizeText = safeStorageGetItem(
      storageKeys.intersectionRadiusCumulative,
    );
    if (cumulativeSizeText !== null) {
      const cumulativeSize = Number(cumulativeSizeText);
      if (Number.isFinite(cumulativeSize)) {
        appState.settings.intersectionRadiusCumulative = roundToHalf(
          clampNumber(cumulativeSize, 0.5, 6),
        );
      }
    }

    const currentSizeText = safeStorageGetItem(
      storageKeys.intersectionRadiusCurrent,
    );
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

  function applySettingsToUi() {
    if (elements.themeMode) {
      elements.themeMode.value = appState.settings.themeMode;
    }
    if (elements.sessionListViewList && elements.sessionListViewGrid) {
      const listActive = appState.settings.sessionListViewMode === "list";
      elements.sessionListViewList.classList.toggle(
        "segmented__button--active",
        listActive,
      );
      elements.sessionListViewList.setAttribute(
        "aria-pressed",
        String(listActive),
      );

      const gridActive = appState.settings.sessionListViewMode === "grid";
      elements.sessionListViewGrid.classList.toggle(
        "segmented__button--active",
        gridActive,
      );
      elements.sessionListViewGrid.setAttribute(
        "aria-pressed",
        String(gridActive),
      );
    }
    if (elements.showCumulativeIntersections) {
      elements.showCumulativeIntersections.checked =
        appState.settings.showCumulativeIntersections;
    }
    if (elements.cumulativeIntersectionSize) {
      elements.cumulativeIntersectionSize.value = String(
        appState.settings.intersectionRadiusCumulative,
      );
    }
    if (elements.cumulativeIntersectionSizeValue) {
      elements.cumulativeIntersectionSizeValue.textContent = formatSizeValue(
        appState.settings.intersectionRadiusCumulative,
      );
    }
    if (elements.currentIntersectionSize) {
      elements.currentIntersectionSize.value = String(
        appState.settings.intersectionRadiusCurrent,
      );
    }
    if (elements.currentIntersectionSizeValue) {
      elements.currentIntersectionSizeValue.textContent = formatSizeValue(
        appState.settings.intersectionRadiusCurrent,
      );
    }
    if (elements.boldActiveSegments) {
      elements.boldActiveSegments.checked =
        appState.settings.boldActiveSegments;
    }

    applySessionListViewMode(elements, appState.settings.sessionListViewMode);
  }

  /**
   * @param {string} nextMode
   */
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
    renderer.invalidateAll();
  }

  /**
   * @param {unknown} enabled
   */
  function setShowCumulativeIntersections(enabled) {
    const next = Boolean(enabled);
    if (appState.settings.showCumulativeIntersections === next) {
      return;
    }
    appState.settings.showCumulativeIntersections = next;
    safeStorageSetItem(storageKeys.showCumulativeIntersections, String(next));
    applySettingsToUi();
    renderer.invalidateDynamic();
  }

  /**
   * @param {unknown} value
   */
  function setIntersectionRadiusCumulative(value) {
    const next = roundToHalf(clampNumber(Number(value) || 0, 0.5, 6));
    if (appState.settings.intersectionRadiusCumulative === next) {
      return;
    }
    appState.settings.intersectionRadiusCumulative = next;
    safeStorageSetItem(storageKeys.intersectionRadiusCumulative, String(next));
    applySettingsToUi();
    renderer.invalidateDynamic();
  }

  /**
   * @param {unknown} value
   */
  function setIntersectionRadiusCurrent(value) {
    const next = roundToHalf(clampNumber(Number(value) || 0, 0.5, 10));
    if (appState.settings.intersectionRadiusCurrent === next) {
      return;
    }
    appState.settings.intersectionRadiusCurrent = next;
    safeStorageSetItem(storageKeys.intersectionRadiusCurrent, String(next));
    applySettingsToUi();
    renderer.invalidateDynamic();
  }

  /**
   * @param {unknown} enabled
   */
  function setBoldActiveSegments(enabled) {
    const next = Boolean(enabled);
    if (appState.settings.boldActiveSegments === next) {
      return;
    }
    appState.settings.boldActiveSegments = next;
    safeStorageSetItem(storageKeys.boldActiveSegments, String(next));
    applySettingsToUi();
    renderer.invalidateDynamic();
  }

  /**
   * @param {string} nextMode
   */
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

  function init() {
    loadSettingsFromStorage();
    applyThemeMode(appState.settings.themeMode);
    applySettingsToUi();

    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", () => {
        if (appState.settings.themeMode !== "system") {
          return;
        }
        renderer.invalidateAll();
      });
  }

  return {
    init,
    setThemeMode,
    setShowCumulativeIntersections,
    setIntersectionRadiusCumulative,
    setIntersectionRadiusCurrent,
    setBoldActiveSegments,
    setSessionListViewMode,
  };
}
