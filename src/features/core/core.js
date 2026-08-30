(() => {
  const resetHandlers = new Set();

  const registerResetHandler = (handler) => {
    resetHandlers.add(handler);
  };

  const resetAll = () => {
    const resetBtn = document.getElementById('item-button-reset')
      || document.querySelector('.item-button-reset');
    const confirmMsg = resetBtn?.dataset?.confirm
      || resetBtn?.getAttribute('data-confirm')
      || '';

    if (confirmMsg && !confirm(confirmMsg)) return;

    resetHandlers.forEach((handler) => {
      try {
        handler();
      } catch (e) {
        console.warn('Failed to execute reset handler', e);
      }
    });

    window.d2f?.storage?.saveState?.();
  };

  window.d2f = window.d2f || {};
  window.d2f.core = {
    registerResetHandler,
    resetAll,
  };

  const init = () => {
    document.addEventListener('click', (e) => {
      const { target } = e;
      if (!(target instanceof Element)) return;

      const btn = target.closest('#item-button-reset, .item-button-reset');
      if (btn) {
        resetAll();
      }
    });
  };

  if (typeof window !== 'undefined') {
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', init);
    } else {
      init();
    }
  }
})();
