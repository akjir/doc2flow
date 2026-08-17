(() => {
  const debounce = (func, wait) => {
    let timeout;
    return (...args) => {
      if (timeout !== undefined) {
        clearTimeout(timeout);
      }
      timeout = setTimeout(() => func(...args), wait);
    };
  };

  const isRecord = (val) => typeof val === 'object' && val !== null && !Array.isArray(val);

  window.d2f = window.d2f || {};
  window.d2f.utils = {
    debounce,
    isRecord,
  };
})();
