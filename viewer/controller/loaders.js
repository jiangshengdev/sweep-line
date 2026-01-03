// @ts-check

import { parseSession, UserError } from "../schema/session.js";
import { parseSessionIndex } from "../schema/session-index.js";

/**
 * @param {any} session
 */
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

/**
 * @param {{ appState: any, setStatus: (message: string) => void }} deps
 */
export function createLoaders({ appState, setStatus }) {
  /**
   * @param {unknown} error
   */
  function handleError(error) {
    if (error instanceof UserError) {
      setStatus(`错误：${error.message}`);
      console.error(error);
      return;
    }
    setStatus("发生未知错误，请在控制台查看详情");
    console.error(error);
  }

  /**
   * @param {unknown} json
   * @param {{
   *   sourceForSelection: string | null,
   *   labelForUi: string,
   *   onBeforeApply?: () => void,
   *   onAfterApply?: () => void,
   *   setDropHintVisible: (visible: boolean) => void,
   *   resetView: () => void,
   *   refreshUiForSession: (label: string) => void,
   *   updateSessionListSelection: () => void,
   * }} opts
   */
  function loadSession(json, opts) {
    opts.onBeforeApply?.();
    const session = parseSession(json);
    prepareSessionForPlayback(session);
    appState.session = session;
    appState.sessionSource = opts.sourceForSelection;
    appState.currentStep = 0;
    opts.setDropHintVisible(false);
    opts.resetView();
    opts.refreshUiForSession(opts.labelForUi);
    opts.updateSessionListSelection();
    setStatus(`已加载：${opts.labelForUi}`);
    opts.onAfterApply?.();
  }

  /**
   * @param {File} file
   * @param {Omit<Parameters<typeof loadSession>[1], "labelForUi" | "sourceForSelection">} opts
   */
  async function loadFromFile(file, opts) {
    let json;
    try {
      const text = await file.text();
      json = JSON.parse(text);
    } catch {
      throw new UserError("JSON 解析失败：不是合法 JSON");
    }
    loadSession(json, {
      ...opts,
      sourceForSelection: null,
      labelForUi: file.name,
    });
  }

  /**
   * @param {string} url
   * @param {Omit<Parameters<typeof loadSession>[1], \"labelForUi\" | \"sourceForSelection\">} opts
   */
  async function loadFromUrl(url, opts) {
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

    loadSession(json, {
      ...opts,
      sourceForSelection: url,
      labelForUi: url,
    });
  }

  /**
   * @param {string} url
   * @param {string} groupTitle
   */
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

  /**
   * @param {{ renderSessionList: () => void, renderSessionPicker: () => void }} opts
   */
  async function loadIndexAndRenderList({ renderSessionList, renderSessionPicker }) {
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
    renderSessionPicker();
  }

  return {
    handleError,
    loadFromFile,
    loadFromUrl,
    loadIndexAndRenderList,
  };
}

