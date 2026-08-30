(() => {
  const exportHandlers = new Set();

  const registerExportHandler = (handler) => {
    exportHandlers.add(handler);
  };

  const performExport = (type) => {
    exportHandlers.forEach((handler) => {
      try {
        handler(type);
      } catch (e) {
        console.warn('Failed to execute export handler', e);
      }
    });

    if (type === 'PDF') {
      const collapsed = Array.from(
        document.querySelectorAll('.section-body.collapsed')
      );
      collapsed.forEach((el) => el.classList.remove('collapsed'));

      const restore = () => {
        collapsed.forEach((el) => el.classList.add('collapsed'));
        window.removeEventListener('afterprint', restore);
      };

      window.addEventListener('afterprint', restore);
      window.setTimeout(() => window.print(), 100);
      return;
    }

    if (type === 'DOCUMENT') {
      window.d2f?.storage?.saveState?.();

      document.querySelectorAll(
        'input.input-field, input.code-table-input, input[type="text"], input:not([type])'
      ).forEach((input) => {
        if (input instanceof HTMLInputElement) {
          input.setAttribute('value', input.value);
        }
      });

      document.querySelectorAll('input[type="checkbox"]').forEach((cb) => {
        if (cb instanceof HTMLInputElement) {
          if (cb.checked) {
            cb.setAttribute('checked', '');
          } else {
            cb.removeAttribute('checked');
          }
        }
      });

      document.querySelectorAll('textarea').forEach((ta) => {
        if (ta instanceof HTMLTextAreaElement) {
          ta.textContent = ta.value;
          ta.setAttribute('value', ta.value);
        }
      });

      const rawFilename = window.location.pathname.split('/').pop() || 'index.html';
      const filename = decodeURIComponent(rawFilename || 'index.html');
      const htmlContent = `<!DOCTYPE html>\n${document.documentElement.outerHTML}`;
      const blob = new Blob([htmlContent], { type: 'text/html;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    }
  };

  window.d2f = window.d2f || {};
  window.d2f.export = {
    export: performExport,
    registerExportHandler,
  };

  const init = () => {
    document.addEventListener('click', (e) => {
      const { target } = e;
      if (!(target instanceof Element)) return;

      const pdfBtn = target.closest('#item-button-pdf, .item-button-pdf');
      if (pdfBtn) {
        performExport('PDF');
        return;
      }

      const saveBtn = target.closest('#item-button-save, .item-button-save');
      if (saveBtn) {
        performExport('DOCUMENT');
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
