(() => {
  const saveHandlers = new Set();
  const loadHandlers = new Set();

  const registerSaveHandler = (handler) => {
    saveHandlers.add(handler);
  };

  const registerLoadHandler = (handler) => {
    loadHandlers.add(handler);
  };

  const getStateKey = () => {
    const docId = window.d2f?.document?.id ?? '';
    const rawFilename = window.location.pathname.split('/').pop() || 'index.html';
    const filename = decodeURIComponent(rawFilename || 'index.html');
    return `d2f_state_${docId ? `${docId}_` : ''}${filename}`;
  };

  const loadState = () => {
    const key = getStateKey();
    try {
      const raw = localStorage.getItem(key);
      if (!raw) return;

      const parsed = JSON.parse(raw);
      const isRecord = window.d2f?.utils?.isRecord;
      if (typeof isRecord === 'function') {
        if (!isRecord(parsed)) return;
      } else if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
        return;
      }

      const state = parsed;

      loadHandlers.forEach((handler) => {
        try {
          handler(state);
        } catch (e) {
          console.warn('Failed to execute load handler', e);
        }
      });
    } catch (e) {
      console.warn(`Failed to load state from localStorage [key: ${key}]`, e);
    }
  };

  const saveState = () => {
    const combinedState = {};

    saveHandlers.forEach((handler) => {
      try {
        const providerState = handler();
        if (providerState && typeof providerState === 'object') {
          Object.assign(combinedState, providerState);
        }
      } catch (e) {
        console.warn('Failed to collect state from handler', e);
      }
    });

    const key = getStateKey();
    try {
      localStorage.setItem(key, JSON.stringify(combinedState));
    } catch (e) {
      console.warn(`Failed to save state to localStorage [key: ${key}]`, e);
    }
  };

  window.d2f = window.d2f || {};
  window.d2f.storage = {
    registerSaveHandler,
    registerLoadHandler,
    loadState,
    saveState,
  };

  if (typeof window !== 'undefined') {
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', () => {
        loadState();
      });
    } else {
      loadState();
    }
  }
})();
