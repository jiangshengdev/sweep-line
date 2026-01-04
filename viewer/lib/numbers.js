// @ts-check

/**
 * 将数字限制在 [min, max] 区间。
 * @param {number} value
 * @param {number} min
 * @param {number} max
 * @returns {number}
 */
export function clampNumber(value, min, max) {
  return Math.min(max, Math.max(min, value));
}

/**
 * 四舍五入到 0.5 精度。
 * @param {number} value
 * @returns {number}
 */
export function roundToHalf(value) {
  return Math.round(value * 2) / 2;
}
