"use strict";
(() => {
  // src/features/toc.ts
  function generateTocItems() {
    const headings = document.querySelectorAll(".sh");
    const items = [];
    headings.forEach((sh, index) => {
      const section = sh.closest(".section");
      const id = section?.id || "section_" + String(index + 1);
      const text = sh.textContent?.trim() ?? "Section " + String(index + 1);
      const isH1 = sh.classList.contains("sh-h1");
      items.push({
        id,
        text,
        level: isH1 ? 1 : 2
      });
    });
    return items;
  }
  function highlightActiveTocItem(activeId) {
    const tocLinks = document.querySelectorAll(".toc-link");
    tocLinks.forEach((link) => {
      const isMatch = link.getAttribute("href") === "#" + activeId;
      link.classList.toggle("active", isMatch);
    });
  }
  window.d2f.toc = {
    generateTocItems,
    highlightActiveTocItem
  };
})();
