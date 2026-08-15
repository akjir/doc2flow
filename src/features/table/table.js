(() => {
  const initTables = () => {
    const tables = document.querySelectorAll('.table-wrap table, .table-default');
    tables.forEach((table) => {
      if (!table.classList.contains('table-default')) {
        table.classList.add('table-default');
      }
    });
  };

  const handleMouseOver = (e) => {
    const { target } = e;
    if (!(target instanceof Element)) {
      return;
    }
    const row = target.closest('.table-default tbody tr');
    if (row) {
      row.classList.add('table-row-hover');
    }
  };

  const handleMouseOut = (e) => {
    const { target } = e;
    if (!(target instanceof Element)) {
      return;
    }
    const row = target.closest('.table-default tbody tr');
    if (row) {
      row.classList.remove('table-row-hover');
    }
  };

  window.d2f = window.d2f || {};
  window.d2f.table = {
    init: initTables,
  };

  const init = () => {
    initTables();
    document.addEventListener('mouseover', handleMouseOver);
    document.addEventListener('mouseout', handleMouseOut);
  };

  if (typeof document !== 'undefined') {
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', init);
    } else {
      init();
    }
  }
})();
