(() => {
  const FIELD_SELECTOR = 'input.input-field, textarea.input-field, input.code-table-input';

  // Serializes current input field and textarea values to persistent storage.
  const saveFields = () => {
    const fields = {};
    document.querySelectorAll(FIELD_SELECTOR).forEach((el, index) => {
      if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement) {
        const key = el.id || `field_${index}`;
        fields[key] = el.value;
      }
    });
    return { fields };
  };

  // Restores input field and textarea values from persistent storage.
  const loadFields = (state) => {
    const fieldsData = state?.fields;
    const isRecord = window.d2f?.utils?.isRecord;

    // Validate schema integrity before DOM manipulation.
    const isValid = typeof isRecord === 'function'
      ? isRecord(fieldsData)
      : (typeof fieldsData === 'object' && fieldsData !== null && !Array.isArray(fieldsData));

    if (!isValid) return;

    document.querySelectorAll(FIELD_SELECTOR).forEach((el, index) => {
      if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement) {
        const key = el.id || `field_${index}`;
        const val = fieldsData[key];
        if (typeof val === 'string') {
          el.value = val;
          if (el instanceof HTMLTextAreaElement) {
            el.textContent = val;
          }
          el.setAttribute('value', val);
        }
      }
    });
  };

  // Resets all interactive input fields and textareas to empty defaults.
  const resetFields = () => {
    document.querySelectorAll(FIELD_SELECTOR).forEach((el) => {
      if (el instanceof HTMLInputElement) {
        el.value = '';
        el.removeAttribute('value');
      } else if (el instanceof HTMLTextAreaElement) {
        el.value = '';
        el.textContent = '';
        el.removeAttribute('value');
      }
    });
  };

  // Debounced state saver for input and change events.
  const debounceFn = window.d2f?.utils?.debounce;
  const saveStateDebounced = typeof debounceFn === 'function'
    ? debounceFn(() => window.d2f?.storage?.saveState?.(), 300)
    : () => window.d2f?.storage?.saveState?.();

  // Handles input and change events on interactive input fields and textareas.
  const handleInputOrChange = (e) => {
    const { target } = e;
    if (!(target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement)) {
      return;
    }

    if (
      target.classList.contains('input-field') ||
      target.classList.contains('code-table-input')
    ) {
      if (target instanceof HTMLInputElement) {
        target.setAttribute('value', target.value);
      } else if (target instanceof HTMLTextAreaElement) {
        target.textContent = target.value;
        target.setAttribute('value', target.value);
      }
      saveStateDebounced();
    }
  };

  // Expose module methods to the global Doc2Flow namespace.
  window.d2f = window.d2f || {};
  window.d2f.input = {
    saveFields,
    loadFields,
    resetFields,
  };

  // Initialize event bindings, storage hooks, and reset handler.
  if (typeof window !== 'undefined') {
    window.d2f.storage?.registerSaveHandler?.(saveFields);
    window.d2f.storage?.registerLoadHandler?.(loadFields);
    window.d2f.core?.registerResetHandler?.(resetFields);

    document.addEventListener('input', handleInputOrChange);
    document.addEventListener('change', handleInputOrChange);
  }
})();
