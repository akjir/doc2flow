(() => {
  // ../src/features/table/table.ts
  (() => {
    function initSectionTables() {
      const wrappers = document.querySelectorAll(".item-table-wrap, .section table");
      wrappers.forEach((wrapper) => {
        const table = wrapper instanceof HTMLTableElement ? wrapper : wrapper.querySelector("table");
        if (!table)
          return;
        if (!table.classList.contains("item-table")) {
          table.classList.add("item-table");
        }
        const tbody = table.querySelector("tbody");
        if (tbody && !tbody.hasAttribute("data-hover-bound")) {
          tbody.setAttribute("data-hover-bound", "true");
          tbody.addEventListener("mouseover", (event) => {
            const target = event.target;
            if (target instanceof Element) {
              const row = target.closest("tr");
              if (row && row.parentElement === tbody) {
                row.classList.add("row-hover");
              }
            }
          });
          tbody.addEventListener("mouseout", (event) => {
            const target = event.target;
            if (target instanceof Element) {
              const row = target.closest("tr");
              if (row && row.parentElement === tbody) {
                row.classList.remove("row-hover");
              }
            }
          });
        }
      });
    }
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", initSectionTables);
    } else {
      initSectionTables();
    }
  })();
})();
