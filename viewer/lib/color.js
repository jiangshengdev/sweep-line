// @ts-check

/**
 * 基于线段 id 生成稳定颜色（便于回放与对照）。
 * @param {number} id
 * @returns {string}
 */
export function stableColorForSegmentId(id) {
  const hue = (id * 47) % 360;
  return `hsl(${hue}deg 85% 68%)`;
}

