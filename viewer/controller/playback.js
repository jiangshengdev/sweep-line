// @ts-check

/**
 * @param {{ elements: any, appState: any, panels: any, renderer: any }} deps
 */
export function createPlaybackController({ elements, appState, panels, renderer }) {
  function stopPlay() {
    if (appState.playTimerId !== null) {
      window.clearInterval(appState.playTimerId);
      appState.playTimerId = null;
    }
    appState.playing = false;
    elements.playPause.textContent = "播放";
  }

  function setCurrentStep(index) {
    if (!appState.session) {
      return;
    }
    if (appState.session.trace.steps.length === 0) {
      return;
    }
    const max = appState.session.trace.steps.length - 1;
    const nextIndex = Math.max(0, Math.min(max, index));
    if (nextIndex === appState.currentStep) {
      return;
    }
    appState.currentStep = nextIndex;
    panels.updateStepControls();
    panels.refreshUiForStep();
    renderer.invalidateDynamic();
  }

  function startPlay() {
    if (!appState.session || appState.session.trace.steps.length === 0) {
      return;
    }
    stopPlay();
    appState.playing = true;
    elements.playPause.textContent = "暂停";
    const baseFps = 10;
    const intervalMs = Math.max(10, Math.floor(1000 / (baseFps * appState.speedFactor)));
    appState.playTimerId = window.setInterval(() => {
      const last = appState.session.trace.steps.length - 1;
      if (appState.currentStep >= last) {
        stopPlay();
        return;
      }
      setCurrentStep(appState.currentStep + 1);
    }, intervalMs);
  }

  function togglePlay() {
    if (!appState.session) {
      return;
    }
    if (appState.playing) {
      stopPlay();
    } else {
      startPlay();
    }
  }

  /**
   * @param {unknown} value
   */
  function setSpeedFactor(value) {
    appState.speedFactor = Number(value) || 1;
    if (appState.playing) {
      startPlay();
    }
  }

  /**
   * 处理播放相关快捷键。
   * @param {KeyboardEvent} event
   * @returns {boolean} 是否已处理
   */
  function handleKeydown(event) {
    if (event.code === "Space") {
      event.preventDefault();
      togglePlay();
      return true;
    }
    if (event.code === "ArrowLeft") {
      event.preventDefault();
      stopPlay();
      setCurrentStep(appState.currentStep - 1);
      return true;
    }
    if (event.code === "ArrowRight") {
      event.preventDefault();
      stopPlay();
      setCurrentStep(appState.currentStep + 1);
      return true;
    }
    return false;
  }

  return {
    stopPlay,
    startPlay,
    togglePlay,
    setCurrentStep,
    setSpeedFactor,
    handleKeydown,
  };
}

