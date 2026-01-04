// @ts-check

import { clearChildren } from "../lib/dom.js";

/**
 * @param {any} item
 * @returns {string}
 */
function folderKeyForSessionIndexItem(item) {
  const rawPath = String(item?.path || "").replace(/\\/g, "/");
  const parts = rawPath.split("/").filter(Boolean);
  if (parts.length < 3) {
    return "";
  }
  return parts[1];
}

/**
 * 将索引项按来源（groupTitle）与二级目录分组，便于列表展示。
 * @param {any[]} items
 */
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

/**
 * @param {{
 *   list: HTMLElement | null,
 *   empty: HTMLElement | null,
 *   items: any[],
 *   currentSource: string | null,
 *   onSelect: (item: any) => void,
 * }} params
 */
export function renderSessionListInto({
  list,
  empty,
  items,
  currentSource,
  onSelect,
}) {
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
      folderTitle.className =
        "session-list__group session-list__group--folder mono";
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
