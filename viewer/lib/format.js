// @ts-check

import { roundToHalf } from "./numbers.js";

/**
 * 格式化滑条展示值：保留 0.5 精度，并去掉尾部的 ".0"。
 * @param {unknown} value
 * @returns {string}
 */
export function formatSizeValue(value) {
  const asNumber = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(asNumber)) {
    return "-";
  }
  const rounded = roundToHalf(asNumber);
  const text = String(rounded);
  return text.endsWith(".0") ? text.slice(0, -2) : text;
}
