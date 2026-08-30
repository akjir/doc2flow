(() => {
  // Adjusts textarea height automatically based on its scrollHeight.
  const autoExpandTextarea = (el) => {
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = `${el.scrollHeight}px`;
  };

  // Creates a comment box or returns the existing one for an item.
  const getOrCreateCommentBox = (item, initialValue) => {
    if (!item) return null;
    let box = item.querySelector('.item-comment-box');
    let input = null;

    if (!box) {
      box = document.createElement('div');
      box.className = 'item-comment-box';

      input = document.createElement('textarea');
      input.rows = 1;
      input.className = 'item-comment-input';
      const i18n = window.d2f?.lang?.dictionary;
      const commentLabel = i18n?.comment_placeholder ?? 'Add a comment...';
      input.placeholder = commentLabel;
      input.setAttribute('aria-label', commentLabel);

      const delBtn = document.createElement('button');
      delBtn.type = 'button';
      delBtn.className = 'item-comment-del';
      delBtn.title = 'Delete comment';
      delBtn.setAttribute('aria-label', 'Delete comment');
      delBtn.innerHTML = '&#10006;';

      box.appendChild(input);
      box.appendChild(delBtn);
      item.appendChild(box);
    } else {
      const rawInput = box.querySelector('.item-comment-input');
      input = rawInput instanceof HTMLTextAreaElement ? rawInput : null;
    }

    if (!input) return null;

    if (typeof initialValue === 'string') {
      input.value = initialValue;
      input.textContent = initialValue;
      input.setAttribute('value', initialValue);
    }

    autoExpandTextarea(input);
    return { box, input };
  };

  // Collects all active comments across selectable items for storage.
  const saveComments = () => {
    const comments = {};
    document.querySelectorAll('.item-selectable').forEach((item, index) => {
      const input = item.querySelector('.item-comment-input');
      if (input && input.value.trim() !== '') {
        const key = item.id || `item_${index}`;
        comments[key] = input.value;
      }
    });
    return { comments };
  };

  // Reconstructs comment boxes from stored comment data.
  const loadComments = (state) => {
    const commentsData = state?.comments;
    const isRecord = window.d2f?.utils?.isRecord;
    const isValid = typeof isRecord === 'function'
      ? isRecord(commentsData)
      : (typeof commentsData === 'object' && commentsData !== null && !Array.isArray(commentsData));

    if (!isValid) return;

    document.querySelectorAll('.item-selectable').forEach((item, index) => {
      const key = item.id || `item_${index}`;
      const val = commentsData[key];
      if (typeof val === 'string' && val.trim() !== '') {
        getOrCreateCommentBox(item, val);
      }
    });
  };

  // Removes all active comment boxes from the DOM.
  const resetComments = () => {
    document.querySelectorAll('.item-comment-box').forEach((box) => {
      box.remove();
    });
  };

  // Expose comment functions to the global Doc2Flow namespace.
  window.d2f = window.d2f || {};
  window.d2f.comments = {
    autoExpandTextarea,
    getOrCreateCommentBox,
    saveComments,
    loadComments,
    resetComments,
  };

  // Initialize event bindings, storage hooks, and reset handler.
  if (typeof window !== 'undefined') {
    window.d2f.storage?.registerSaveHandler?.(saveComments);
    window.d2f.storage?.registerLoadHandler?.(loadComments);
    window.d2f.core?.registerResetHandler?.(resetComments);

    const debounceFn = window.d2f?.utils?.debounce;
    const saveStateDebounced = typeof debounceFn === 'function'
      ? debounceFn(() => window.d2f.storage?.saveState?.(), 300)
      : () => window.d2f.storage?.saveState?.();

    document.addEventListener('click', (e) => {
      const { target } = e;
      if (!(target instanceof Element)) return;

      const commentBtn = target.closest('.item-comment-icon');
      if (commentBtn) {
        const item = commentBtn.closest('.item-selectable');
        if (item) {
          const res = getOrCreateCommentBox(item);
          if (res?.input) {
            res.input.focus();
          }
        }
        return;
      }

      const commentDelBtn = target.closest('.item-comment-del');
      if (commentDelBtn) {
        const box = commentDelBtn.closest('.item-comment-box');
        if (box) {
          box.remove();
          window.d2f.storage?.saveState?.();
        }
      }
    });

    const handleCommentInput = (e) => {
      const { target } = e;
      if (target instanceof HTMLTextAreaElement && target.classList.contains('item-comment-input')) {
        target.textContent = target.value;
        target.setAttribute('value', target.value);
        autoExpandTextarea(target);
        saveStateDebounced();
      }
    };

    document.addEventListener('input', handleCommentInput);
    document.addEventListener('change', handleCommentInput);
  }
})();
