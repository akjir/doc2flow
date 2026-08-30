(() => {
  // 1. Module-Private State & Constants
  const feedbackTimers = new WeakMap();

  const COPY_SVG =
    '<svg aria-hidden="true" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>';
  const CHECK_SVG =
    '<svg aria-hidden="true" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>';

  const getCopyLabel = () => window.d2f?.lang?.dictionary?.copy_code ?? 'Copy code';
  const getCopiedLabel = () => window.d2f?.lang?.dictionary?.copied ?? 'Copied!';

  // 2. 1-Click Copy Engine
  const showCopiedFeedback = (btn) => {
    btn.classList.add('copied');
    btn.innerHTML = CHECK_SVG;
    const copiedLabel = getCopiedLabel();
    btn.setAttribute('title', copiedLabel);
    btn.setAttribute('aria-label', copiedLabel);

    const existingTimer = feedbackTimers.get(btn);
    if (existingTimer !== undefined) {
      window.clearTimeout(existingTimer);
    }

    const timer = window.setTimeout(() => {
      btn.classList.remove('copied');
      btn.innerHTML = COPY_SVG;
      const copyLabel = getCopyLabel();
      btn.setAttribute('title', copyLabel);
      btn.setAttribute('aria-label', copyLabel);
      feedbackTimers.delete(btn);
    }, 2000);

    feedbackTimers.set(btn, timer);
  };

  const fallbackCopyText = (text, btn) => {
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.style.position = 'fixed';
    ta.style.left = '-9999px';
    ta.style.top = '0';
    ta.setAttribute('readonly', '');
    document.body.appendChild(ta);
    ta.select();
    ta.setSelectionRange(0, text.length);

    try {
      const success = document.execCommand('copy');
      if (success) {
        showCopiedFeedback(btn);
      }
    } catch {
      // Gracefully handle clipboard execution failure
    } finally {
      document.body.removeChild(ta);
    }
  };

  const copyCode = async (btn) => {
    if (!btn) return;
    const pre = btn.closest('pre');
    if (!pre) return;
    const codeEl = pre.querySelector('code');
    if (!codeEl) return;

    const text = codeEl.textContent ?? '';

    if (navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
      try {
        await navigator.clipboard.writeText(text);
        showCopiedFeedback(btn);
        return;
      } catch {
        fallbackCopyText(text, btn);
        return;
      }
    }

    fallbackCopyText(text, btn);
  };

  const initCopyButtons = () => {
    const codeElements = document.querySelectorAll('pre code, .code-default code');
    codeElements.forEach((codeEl) => {
      const pre = codeEl.parentElement;
      if (!pre || pre.querySelector('.code-copy-btn')) return;

      const copyLabel = getCopyLabel();
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = 'code-copy-btn';
      btn.setAttribute('aria-label', copyLabel);
      btn.setAttribute('title', copyLabel);
      btn.innerHTML = COPY_SVG;

      btn.addEventListener('click', () => {
        void copyCode(btn);
      });

      pre.style.position = 'relative';
      pre.appendChild(btn);
    });
  };

  // 3. Live Variable Substitution Engine
  const getVariableMap = () => {
    const map = {};
    const inputs = document.querySelectorAll(
      'input.code-table-input, input.item-table-var-input, input[data-var-key]'
    );
    inputs.forEach((input) => {
      if (input instanceof HTMLInputElement) {
        const key = input.dataset.varKey ?? input.getAttribute('data-var-key');
        if (typeof key === 'string' && key.trim() !== '') {
          map[key.trim()] = input.value;
        }
      }
    });
    return map;
  };

  const replaceCodeVariables = (text, varMap) => {
    if (Object.keys(varMap).length === 0) {
      return text;
    }
    return text.replace(/\{\{([A-Za-z0-9_]+)\}\}/g, (match, key) => {
      const val = varMap[key];
      if (val !== undefined && val.trim() !== '') {
        return val;
      }
      return match;
    });
  };

  const updateAllCodeVariables = () => {
    const varMap = getVariableMap();
    const codeElements = document.querySelectorAll('.code-default code, pre code');
    codeElements.forEach((codeEl) => {
      if (!codeEl.hasAttribute('data-raw-code')) {
        codeEl.setAttribute('data-raw-code', codeEl.textContent ?? '');
      }
      const rawText = codeEl.getAttribute('data-raw-code') ?? '';
      const replacedText = replaceCodeVariables(rawText, varMap);
      if (codeEl.textContent !== replacedText) {
        codeEl.textContent = replacedText;
      }
    });
  };

  const resetCodeVariables = () => {
    const codeElements = document.querySelectorAll('.code-default code, pre code');
    codeElements.forEach((codeEl) => {
      const rawText = codeEl.getAttribute('data-raw-code');
      if (rawText !== null) {
        codeEl.textContent = rawText;
      }
    });
  };

  const handleFocusIn = (e) => {
    const { target } = e;
    if (
      target instanceof HTMLInputElement &&
      (target.classList.contains('code-table-input') ||
        target.classList.contains('item-table-var-input') ||
        target.hasAttribute('data-var-key'))
    ) {
      window.requestAnimationFrame(() => {
        target.select();
      });
    }
  };

  const handleInput = (e) => {
    const { target } = e;
    if (
      target instanceof HTMLInputElement &&
      (target.classList.contains('code-table-input') ||
        target.classList.contains('item-table-var-input') ||
        target.hasAttribute('data-var-key'))
    ) {
      updateAllCodeVariables();
    }
  };

  // 4. Public API Export (Strict d2f namespace)
  window.d2f = window.d2f || {};
  window.d2f.code = {
    copy: copyCode,
    updateVariables: updateAllCodeVariables,
    resetVariables: resetCodeVariables,
    initCopyButtons,
  };

  // 5. Module Initialization
  const init = () => {
    window.d2f.core?.registerResetHandler?.(resetCodeVariables);
    window.d2f.storage?.registerResetHandler?.(resetCodeVariables);

    updateAllCodeVariables();
    initCopyButtons();

    document.addEventListener('focusin', handleFocusIn);
    document.addEventListener('input', handleInput);
  };

  if (typeof window !== 'undefined') {
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', init);
    } else {
      init();
    }
  }
})();
