// @ts-check

import { appendKvLines, appendListItems, clearChildren } from "../lib/dom.js";

/**
 * @param {number[] | null | undefined} ids
 * @param {number} limit
 * @returns {string}
 */
function formatIdList(ids, limit) {
  if (!Array.isArray(ids) || ids.length === 0) {
    return "[]";
  }
  const shown = ids.slice(0, limit);
  const suffix = ids.length > limit ? `,...${ids.length - limit} more` : "";
  return `[${shown.join(",")}${suffix}]`;
}

/**
 * @param {number[] | null | undefined} ids
 * @param {number} limit
 * @returns {string}
 */
function formatOptionalIdList(ids, limit) {
  if (ids == null) {
    return "（未知）";
  }
  return formatIdList(ids, limit);
}

/**
 * @param {HTMLElement | null} container
 * @param {string | null} current
 */
function updateSessionListSelectionIn(container, current) {
  if (!container) {
    return;
  }
  const buttons = container.querySelectorAll("button.session-list__item");
  for (const btn of buttons) {
    btn.classList.toggle("session-list__item--active", current && btn.dataset.path === current);
  }
}

/**
 * @param {{ elements: any, appState: any }} deps
 */
export function createPanels({ elements, appState }) {
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

    const pointText = step.point ? `(${step.point.x.text}, ${step.point.y.text})` : "null";

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
      const tdSegments = document.createElement("td");
      tdSegments.textContent = formatIdList(it.segments, 16);
      const tdKind = document.createElement("td");
      tdKind.textContent = it.kindDetail ?? it.kind;
      const tdPoint = document.createElement("td");
      tdPoint.textContent = `(${it.point.x.text}, ${it.point.y.text})`;
      const tdEndpoint = document.createElement("td");
      tdEndpoint.textContent = formatOptionalIdList(it.endpointSegments, 16);
      const tdInterior = document.createElement("td");
      tdInterior.textContent = formatOptionalIdList(it.interiorSegments, 16);
      tr.appendChild(tdSegments);
      tr.appendChild(tdKind);
      tr.appendChild(tdPoint);
      tr.appendChild(tdEndpoint);
      tr.appendChild(tdInterior);
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

  function updateSessionListSelection() {
    const current = appState.sessionSource;
    updateSessionListSelectionIn(elements.sessionList, current);
    updateSessionListSelectionIn(elements.sessionPickerList, current);
  }

  return {
    refreshUiForSession,
    refreshUiForStep,
    updateStepControls,
    updateSessionListSelection,
  };
}
