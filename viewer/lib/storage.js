// @ts-check

/**
 * 安全读取 localStorage：在隐私模式/禁用/配额问题等情况下返回 null。
 * @param {string} key
 * @returns {string | null}
 */
export function safeStorageGetItem(key) {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

/**
 * 安全写入 localStorage：失败时保持静默（不影响主要功能）。
 * @param {string} key
 * @param {string} value
 */
export function safeStorageSetItem(key, value) {
  try {
    window.localStorage.setItem(key, value);
  } catch {
    // localStorage 不可用时保持静默：不影响主要功能
  }
}

