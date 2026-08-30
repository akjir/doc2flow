(() => {
  // Serializes current selection and checkbox state to persistent storage.
  const saveItems = () => {
    const items = {};
    document.querySelectorAll('.item-selectable').forEach((item, index) => {
      // Prioritize explicit Rust-rendered IDs; fallback to index-based generation.
      const key = item.id || `item_${index}`;
      items[key] = item.classList.contains('checked');
    });
    return { items };
  };

  // Restores DOM selection and checkbox checked state from storage.
  const loadItems = (state) => {
    const itemsData = state?.items;
    const isRecord = window.d2f?.utils?.isRecord;

    // Validate schema integrity before DOM manipulation.
    const isValid = typeof isRecord === 'function'
      ? isRecord(itemsData)
      : (typeof itemsData === 'object' && itemsData !== null && !Array.isArray(itemsData));

    if (!isValid) return;

    document.querySelectorAll('.item-selectable').forEach((item, index) => {
      const key = item.id || `item_${index}`;
      const isChecked = itemsData[key];
      if (typeof isChecked === 'boolean') {
        item.classList.toggle('checked', isChecked);
        const checkbox = item.querySelector('input[type="checkbox"], .check-box');
        if (checkbox instanceof HTMLInputElement) {
          checkbox.checked = isChecked;
        }
      }
    });
  };

  // Resets all interactive items and checkboxes to default state.
  const resetItems = () => {
    document.querySelectorAll('.item-selectable').forEach((item) => {
      item.classList.remove('checked');
      const checkbox = item.querySelector('input[type="checkbox"], .check-box');
      if (checkbox instanceof HTMLInputElement) {
        checkbox.checked = false;
      }
    });
  };

  // Handles delegated click events for selectable items and checkboxes.
  // Exactly one saveState invocation is performed per user interaction.
  const handleClick = (e) => {
    const { target } = e;
    if (!(target instanceof Element)) return;

    // Ignore interactive elements, but permit checkboxes.
    if (
      target.closest('a') ||
      target.closest('button') ||
      (target.closest('input') && !(target instanceof HTMLInputElement && target.type === 'checkbox')) ||
      target.closest('textarea') ||
      target.closest('select') ||
      target.closest('summary') ||
      target.closest('.item-comment-icon') ||
      target.closest('.item-comment-box')
    ) {
      return;
    }

    const item = target.closest('.item-selectable');
    if (!item) return;

    const checkbox = item.querySelector('input[type="checkbox"], .check-box');
    if (checkbox instanceof HTMLInputElement) {
      if (target === checkbox) {
        // Direct checkbox click: match item class to input state.
        item.classList.toggle('checked', checkbox.checked);
      } else {
        // Row/text click: toggle checkbox and match item class.
        checkbox.checked = !checkbox.checked;
        item.classList.toggle('checked', checkbox.checked);
      }
    } else {
      // Plain item (text, bullet, ordered list item).
      item.classList.toggle('checked');
    }

    // Exactly one save call per interaction.
    window.d2f?.storage?.saveState?.();
  };

  // Expose module methods to the global Doc2Flow namespace.
  window.d2f = window.d2f || {};
  window.d2f.items = {
    saveItems,
    loadItems,
    resetItems,
  };

  // Initialize event bindings and storage hooks.
  if (typeof window !== 'undefined') {
    window.d2f.storage?.registerSaveHandler?.(saveItems);
    window.d2f.storage?.registerLoadHandler?.(loadItems);
    window.d2f.core?.registerResetHandler?.(resetItems);
    document.addEventListener('click', handleClick);
  }
})();
