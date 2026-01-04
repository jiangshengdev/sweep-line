// @ts-check

import { renderSessionListInto } from "./session-list.js";

/**
 * @param {unknown} query
 * @returns {string}
 */
function normalizeSearchQuery(query) {
  return String(query || "")
    .trim()
    .toLowerCase();
}

/**
 * @param {any} item
 * @param {string} normalizedQuery
 * @returns {boolean}
 */
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

/**
 * @param {any[]} items
 * @param {string} query
 */
function filterSessionIndexItems(items, query) {
  const normalized = normalizeSearchQuery(query);
  if (!normalized) {
    return items;
  }
  return items.filter((item) => sessionIndexItemMatchesQuery(item, normalized));
}

/**
 * @param {{
 *   elements: any,
 *   appState: any,
 *   onSelect: (item: any) => void,
 * }} params
 */
export function renderSessionPickerList({ elements, appState, onSelect }) {
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
    onSelect,
  });
}
