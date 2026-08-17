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

  const translate = (key) => {
    if (typeof key !== 'string' && typeof key !== 'number') {
      return '';
    }
    const dict = window.d2f?.language?.dictionary || window.d2f?.lang?.dictionary;
    if (dict && typeof dict[key] === 'string') {
      return dict[key];
    }
    return `{{${key}}}`;
  };

  window.d2f = window.d2f || {};
  window.d2f.utils = {
    debounce,
    isRecord,
    translate,
  };
})();
