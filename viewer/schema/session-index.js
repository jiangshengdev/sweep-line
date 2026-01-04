// @ts-check

import { UserError } from "./session.js";

/**
 * @param {unknown} value
 * @param {string} path
 * @returns {Record<string, unknown>}
 */
function parseObject(value, path) {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new UserError(`${path} 不是对象`);
  }
  return /** @type {Record<string, unknown>} */ (value);
}

/**
 * @param {unknown} value
 * @param {string} path
 * @returns {string}
 */
function parseString(value, path) {
  if (typeof value !== "string") {
    throw new UserError(`${path} 不是字符串`);
  }
  return value;
}

/**
 * @template T
 * @param {unknown} value
 * @param {string} path
 * @returns {T[]}
 */
function parseArray(value, path) {
  if (!Array.isArray(value)) {
    throw new UserError(`${path} 不是数组`);
  }
  return /** @type {T[]} */ (value);
}

/**
 * @param {unknown} value
 * @param {string} path
 * @returns {number}
 */
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

/**
 * 解析 `session-index.v1` JSON。
 * @param {unknown} value
 * @param {string} path
 */
export function parseSessionIndex(value, path) {
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
