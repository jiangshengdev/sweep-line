// @ts-check

/**
 * 移除节点的所有子节点。
 * @param {Element} node
 */
export function clearChildren(node) {
  while (node.firstChild) {
    node.removeChild(node.firstChild);
  }
}

/**
 * 向列表容器追加若干行文本（以 <li> 渲染）。
 * @param {HTMLElement} list
 * @param {string[]} items
 */
export function appendListItems(list, items) {
  const fragment = document.createDocumentFragment();
  for (const item of items) {
    const li = document.createElement("li");
    li.textContent = item;
    fragment.appendChild(li);
  }
  list.appendChild(fragment);
}

/**
 * 渲染 key/value 多行文本（以 <div> 列表渲染）。
 * @param {HTMLElement} container
 * @param {Array<[string, string]>} lines
 */
export function appendKvLines(container, lines) {
  clearChildren(container);
  const fragment = document.createDocumentFragment();
  for (const [key, value] of lines) {
    const div = document.createElement("div");
    div.textContent = `${key}: ${value}`;
    fragment.appendChild(div);
  }
  container.appendChild(fragment);
}

