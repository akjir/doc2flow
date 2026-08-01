"use strict";
(() => {
  // src/core/storage.ts
  var saveHandlers = /* @__PURE__ */ new Set();
  var loadHandlers = /* @__PURE__ */ new Set();
  window.d2f.storage = {
    registerSaveHandler,
    registerLoadHandler,
    loadState,
    saveState
  };
  function registerSaveHandler(handler) {
    saveHandlers.add(handler);
  }
  function registerLoadHandler(handler) {
    loadHandlers.add(handler);
  }
  function loadState() {
    const key = getStateKey();
    try {
      const raw = localStorage.getItem(key);
      if (!raw)
        return;
      const parsed = JSON.parse(raw);
      if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed))
        return;
      const state = parsed;
      for (const handler of loadHandlers) {
        try {
          handler(state);
        } catch (e) {
          console.warn("Failed to execute load handler", e);
        }
      }
    } catch (e) {
      console.warn(`Failed to load state from localStorage [key: ${key}]`, e);
    }
  }
  function saveState() {
    const combinedState = {};
    for (const handler of saveHandlers) {
      try {
        const providerState = handler();
        if (providerState && typeof providerState === "object") {
          Object.assign(combinedState, providerState);
        }
      } catch (e) {
        console.warn("Failed to collect state from handler", e);
      }
    }
    const key = getStateKey();
    try {
      localStorage.setItem(key, JSON.stringify(combinedState));
    } catch (e) {
      console.warn(`Failed to save state to localStorage [key: ${key}]`, e);
    }
  }
  function getStateKey() {
    const docId = window.D2F_DOC_ID ?? "";
    const rawFilename = window.location.pathname.split("/").pop() ?? "index.html";
    const filename = decodeURIComponent(rawFilename);
    return "d2f_state_" + (docId ? `${docId}_` : "") + filename;
  }
  if (typeof window !== "undefined") {
    document.addEventListener("DOMContentLoaded", () => {
      loadState();
    });
  }

  // src/core/utils.ts
  function debounce(func, wait) {
    let timeout;
    return function(...args) {
      if (timeout !== void 0) {
        clearTimeout(timeout);
      }
      timeout = setTimeout(() => func(...args), wait);
    };
  }
  function isRecord(val) {
    return typeof val === "object" && val !== null && !Array.isArray(val);
  }
  window.d2f.utils = {
    debounce,
    isRecord
  };

  // src/core/export.ts
  var ExportType = {
    PDF: "PDF",
    DOCUMENT: "DOCUMENT"
  };
  var exportHandlers = /* @__PURE__ */ new Set();
  window.d2f.export = {
    export: performExport,
    registerExportHandler
  };
  function registerExportHandler(handler) {
    exportHandlers.add(handler);
  }
  function performExport(type) {
    for (const handler of exportHandlers) {
      try {
        handler(type);
      } catch (e) {
        console.warn("Failed to execute export handler", e);
      }
    }
    if (type === ExportType.PDF) {
      const collapsed = Array.from(document.querySelectorAll(".sb.collapsed"));
      collapsed.forEach((el) => el.classList.remove("collapsed"));
      const restore = () => {
        collapsed.forEach((el) => el.classList.add("collapsed"));
        window.removeEventListener("afterprint", restore);
      };
      window.addEventListener("afterprint", restore);
      setTimeout(() => window.print(), 100);
      return;
    }
    if (type === ExportType.DOCUMENT) {
      window.d2f.storage.saveState();
      const rawFilename = window.location.pathname.split("/").pop() ?? "index.html";
      const filename = decodeURIComponent(rawFilename || "index.html");
      const htmlContent = "<!DOCTYPE html>\n" + document.documentElement.outerHTML;
      const blob = new Blob([htmlContent], { type: "text/html;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    }
  }

  // src/core/sections.ts
  var SECTION_SELECTOR = ".section";
  function isRecord2(val) {
    return typeof val === "object" && val !== null && !Array.isArray(val);
  }
  function setSectionCollapseState(sec, isCollapsed) {
    const body = sec.querySelector(".sb");
    const sh = sec.querySelector(".sh");
    if (!body)
      return;
    body.classList.toggle("collapsed", isCollapsed);
    if (sh) {
      sh.setAttribute("aria-expanded", isCollapsed ? "false" : "true");
      const toggler = sh.querySelector(".stog");
      if (toggler) {
        toggler.innerHTML = isCollapsed ? "&#9650;" : "&#9660;";
      }
    }
  }
  function toggleSection(target, onSave) {
    let headerElement = null;
    if (typeof target === "string") {
      const sec = document.getElementById(target);
      if (sec) {
        headerElement = sec.querySelector(".sh");
      }
    } else if (target instanceof HTMLElement) {
      headerElement = target.closest(".sh");
    }
    if (!headerElement || headerElement.classList.contains("no-toggle")) {
      return;
    }
    const section = headerElement.closest(".section");
    const body = section ? section.querySelector(".sb") : null;
    if (body && (body.children.length > 0 || body.innerHTML.trim() !== "")) {
      const isCollapsed = !body.classList.contains("collapsed");
      if (section) {
        setSectionCollapseState(section, isCollapsed);
      } else {
        body.classList.toggle("collapsed", isCollapsed);
        headerElement.setAttribute("aria-expanded", isCollapsed ? "false" : "true");
        const toggler = headerElement.querySelector(".stog");
        if (toggler) {
          toggler.innerHTML = isCollapsed ? "&#9650;" : "&#9660;";
        }
      }
      if (onSave) {
        onSave();
      }
    }
  }
  function saveSections() {
    const sections = {};
    document.querySelectorAll(SECTION_SELECTOR).forEach((sec, index) => {
      const body = sec.querySelector(".sb");
      if (body) {
        const key = sec.id || "sec_" + String(index);
        sections[key] = body.classList.contains("collapsed");
      }
    });
    return { sections };
  }
  function loadSections(state) {
    const sectionsData = state["sections"];
    if (isRecord2(sectionsData)) {
      document.querySelectorAll(SECTION_SELECTOR).forEach((sec, index) => {
        const key = sec.id || "sec_" + String(index);
        const shouldCollapse = sectionsData[key];
        if (typeof shouldCollapse === "boolean") {
          setSectionCollapseState(sec, shouldCollapse);
        }
      });
    }
    return false;
  }
  function resetSections() {
    document.querySelectorAll(SECTION_SELECTOR).forEach((sec) => {
      setSectionCollapseState(sec, false);
    });
    document.querySelectorAll(".sb.collapsed").forEach((body) => {
      body.classList.remove("collapsed");
    });
  }
  if (typeof window !== "undefined") {
    document.addEventListener("DOMContentLoaded", () => {
      window.d2f.storage.registerSaveHandler(saveSections);
      window.d2f.storage.registerLoadHandler(loadSections);
      window.d2f.core.registerResetHandler(resetSections);
    });
    document.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === " ") {
        const target = e.target;
        if (target instanceof Element) {
          const sh = target.closest(".sh");
          if (sh && !sh.classList.contains("no-toggle")) {
            e.preventDefault();
            toggleSection(sh, () => window.d2f.storage.saveState());
          }
        }
      }
    });
    document.addEventListener("click", (e) => {
      const target = e.target;
      if (!(target instanceof Element))
        return;
      const sh = target.closest(".sh");
      if (sh && !sh.classList.contains("no-toggle")) {
        toggleSection(sh, () => window.d2f.storage.saveState());
      }
    });
  }

  // src/core/comments.ts
  function autoExpandTextarea(el) {
    if (!el)
      return;
    el.style.height = "auto";
    el.style.height = String(el.scrollHeight) + "px";
  }
  function getOrCreateCommentBox(checkItem, initialValue) {
    if (!checkItem)
      return null;
    let box = checkItem.querySelector(".item-comment-box");
    let input = null;
    if (!box) {
      box = document.createElement("div");
      box.className = "item-comment-box";
      input = document.createElement("textarea");
      input.rows = 1;
      input.className = "item-comment-input";
      const i18n = window.D2F_I18N ?? {};
      const commentLabel = i18n.comment_placeholder ?? "Add a comment...";
      input.placeholder = commentLabel;
      input.setAttribute("aria-label", commentLabel);
      const delBtn = document.createElement("button");
      delBtn.type = "button";
      delBtn.className = "item-comment-del";
      delBtn.title = "Delete comment";
      delBtn.setAttribute("aria-label", "Delete comment");
      delBtn.innerHTML = "&#10006;";
      box.appendChild(input);
      box.appendChild(delBtn);
      checkItem.appendChild(box);
    } else {
      const rawInput = box.querySelector(".item-comment-input");
      input = rawInput instanceof HTMLTextAreaElement ? rawInput : null;
    }
    if (!input)
      return null;
    if (typeof initialValue === "string") {
      input.value = initialValue;
      input.textContent = initialValue;
      input.setAttribute("value", initialValue);
    }
    autoExpandTextarea(input);
    return { box, input };
  }
  function saveComments() {
    const comments = {};
    document.querySelectorAll(".check-item").forEach((item, index) => {
      const input = item.querySelector(".item-comment-input");
      if (input && input.value.trim() !== "") {
        const key = item.id || "item_" + String(index);
        comments[key] = input.value;
      }
    });
    return { comments };
  }
  function loadComments(state) {
    const comments = state["comments"];
    if (typeof comments === "object" && comments !== null && !Array.isArray(comments)) {
      const commentsRecord = comments;
      document.querySelectorAll(".check-item").forEach((item, index) => {
        const key = item.id || "item_" + String(index);
        const val = commentsRecord[key];
        if (val !== void 0 && typeof val === "string") {
          getOrCreateCommentBox(item, val);
        }
      });
    }
    return false;
  }
  function resetComments() {
    document.querySelectorAll(".item-comment-box").forEach((box) => {
      box.remove();
    });
  }
  if (typeof window !== "undefined") {
    document.addEventListener("DOMContentLoaded", () => {
      window.d2f.storage.registerSaveHandler(saveComments);
      window.d2f.storage.registerLoadHandler(loadComments);
      window.d2f.core.registerResetHandler(resetComments);
    });
    const saveStateDebounced = window.d2f.utils.debounce(() => window.d2f.storage.saveState(), 300);
    document.addEventListener("click", (e) => {
      const target = e.target;
      if (!(target instanceof Element))
        return;
      const commentBtn = target.closest(".item-comment-icon");
      if (commentBtn) {
        const checkItem = commentBtn.closest(".check-item");
        if (checkItem) {
          const res = getOrCreateCommentBox(checkItem);
          if (res?.input) {
            res.input.focus();
          }
        }
        return;
      }
      const commentDelBtn = target.closest(".item-comment-del");
      if (commentDelBtn) {
        const box = commentDelBtn.closest(".item-comment-box");
        if (box) {
          box.remove();
          window.d2f.storage.saveState();
        }
      }
    });
    const handleCommentInput = (e) => {
      const target = e.target;
      if (target instanceof HTMLTextAreaElement && target.classList.contains("item-comment-input")) {
        target.textContent = target.value;
        target.setAttribute("value", target.value);
        saveStateDebounced();
      }
    };
    document.addEventListener("input", handleCommentInput);
    document.addEventListener("change", handleCommentInput);
  }

  // src/core/fields.ts
  function syncFieldPair(id1, id2, sourceInput) {
    const raw1 = document.getElementById(id1);
    const raw2 = document.getElementById(id2);
    const el1 = raw1 instanceof HTMLInputElement ? raw1 : null;
    const el2 = raw2 instanceof HTMLInputElement ? raw2 : null;
    if (!el1 || !el2)
      return;
    if (sourceInput === el1) {
      el2.value = el1.value;
    } else if (sourceInput === el2) {
      el1.value = el2.value;
    } else {
      if (el1.value && !el2.value)
        el2.value = el1.value;
      else if (el2.value && !el1.value)
        el1.value = el2.value;
      else if (el1.value)
        el2.value = el1.value;
    }
  }
  function syncLinkedFields(sourceInput) {
    syncFieldPair("f_info_agent", "f_sign_agent", sourceInput);
    syncFieldPair("f_info_date", "f_sign_date", sourceInput);
  }
  function formatDateFromTemplate(now, template) {
    if (!template || typeof template !== "string")
      return null;
    const tokenMap = {
      "YYYY": String(now.getFullYear()),
      "YY": String(now.getFullYear()).slice(-2),
      "MM": String(now.getMonth() + 1).padStart(2, "0"),
      "DD": String(now.getDate()).padStart(2, "0"),
      "M": String(now.getMonth() + 1),
      "D": String(now.getDate())
    };
    const regex = /YYYY|YY|MM|DD|M|D/gi;
    let hasMatches = false;
    const formatted = template.replace(regex, (match) => {
      hasMatches = true;
      const key = match.toUpperCase();
      const value = tokenMap[key];
      return value ?? match;
    });
    return hasMatches && !/[A-Za-z]/.test(formatted) ? formatted : null;
  }
  function getTodayFormatted() {
    const i18n = window.D2F_I18N ?? {};
    const now = /* @__PURE__ */ new Date();
    try {
      const fromTemplate = formatDateFromTemplate(now, i18n.date_placeholder);
      if (fromTemplate)
        return fromTemplate;
    } catch (e) {
      console.warn("Failed to format date", e);
    }
    return now.toLocaleDateString(navigator.language || void 0);
  }
  function checkDateShortcut(input) {
    if (typeof input.value !== "string")
      return false;
    if (input.value.trim().toLowerCase() === "today") {
      input.value = getTodayFormatted();
      return true;
    }
    return false;
  }
  function saveFields() {
    const fields = {};
    document.querySelectorAll("input.persistent-field").forEach((input, index) => {
      const key = input.id || "f_" + String(index);
      fields[key] = input.value;
    });
    return { fields };
  }
  function loadFields(state) {
    const fieldsData = state["fields"];
    if (window.d2f.utils.isRecord(fieldsData)) {
      document.querySelectorAll("input.persistent-field").forEach((input, index) => {
        const key = input.id || "f_" + String(index);
        const val = fieldsData[key];
        if (typeof val === "string") {
          input.value = val;
        }
      });
    }
    syncLinkedFields();
    return false;
  }
  function resetFields() {
    document.querySelectorAll(
      "input, textarea, select"
    ).forEach((el) => {
      if (el.id === "search-input" || el.classList.contains("search-input"))
        return;
      if (el instanceof HTMLInputElement) {
        if (el.type === "checkbox" || el.type === "radio") {
          el.checked = false;
        } else {
          el.value = "";
        }
      } else if (el instanceof HTMLTextAreaElement) {
        el.value = "";
        el.textContent = "";
      } else if (el instanceof HTMLSelectElement) {
        el.selectedIndex = 0;
      }
    });
    syncLinkedFields();
  }
  if (typeof window !== "undefined") {
    document.addEventListener("DOMContentLoaded", () => {
      window.d2f.storage.registerSaveHandler(saveFields);
      window.d2f.storage.registerLoadHandler(loadFields);
      window.d2f.core.registerResetHandler(resetFields);
    });
    const linkedIds = ["f_info_agent", "f_sign_agent", "f_info_date", "f_sign_date"];
    const saveStateDebounced = window.d2f.utils.debounce(() => window.d2f.storage.saveState(), 300);
    const handleInputOrChange = (e) => {
      const target = e.target;
      if (!(target instanceof HTMLInputElement))
        return;
      if (target.classList.contains("persistent-field")) {
        saveStateDebounced();
      }
      if (target.id && linkedIds.includes(target.id)) {
        if (target.id.toLowerCase().includes("date")) {
          checkDateShortcut(target);
        }
        syncLinkedFields(target);
        saveStateDebounced();
      } else if (target.matches('input[id*="date"], input[name*="date"], input.date-field')) {
        checkDateShortcut(target);
        saveStateDebounced();
      }
    };
    document.addEventListener("input", handleInputOrChange);
    document.addEventListener("change", handleInputOrChange);
  }

  // src/core/search.ts
  var preSearchCollapsedState = null;
  var lastMatchedSectionIds = /* @__PURE__ */ new Set();
  function removeHighlights(container) {
    if (!container)
      return;
    const highlights = container.querySelectorAll("mark.d2f-highlight");
    highlights.forEach((mark) => {
      const parent = mark.parentNode;
      if (parent) {
        const textContent = mark.textContent ?? "";
        parent.replaceChild(document.createTextNode(textContent), mark);
        parent.normalize();
      }
    });
  }
  function highlightTextNodes(container, query) {
    if (!container || !query)
      return;
    const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const regex = new RegExp(escaped, "gi");
    const walker = document.createTreeWalker(
      container,
      NodeFilter.SHOW_TEXT,
      {
        acceptNode: (node) => {
          const parent = node.parentNode;
          if (!parent || !(parent instanceof HTMLElement))
            return NodeFilter.FILTER_REJECT;
          const tag = parent.nodeName.toLowerCase();
          if (["script", "style", "input", "textarea", "select", "button"].includes(tag) || parent.classList.contains("d2f-highlight")) {
            return NodeFilter.FILTER_REJECT;
          }
          return NodeFilter.FILTER_ACCEPT;
        }
      }
    );
    const textNodes = [];
    let currentNode = walker.nextNode();
    while (currentNode) {
      if (currentNode instanceof Text) {
        textNodes.push(currentNode);
      }
      currentNode = walker.nextNode();
    }
    textNodes.forEach((textNode) => {
      const val = textNode.nodeValue;
      if (!val || !regex.test(val))
        return;
      regex.lastIndex = 0;
      const frag = document.createDocumentFragment();
      let lastIdx = 0;
      let match = regex.exec(val);
      while (match !== null) {
        if (match.index > lastIdx) {
          frag.appendChild(document.createTextNode(val.slice(lastIdx, match.index)));
        }
        const mark = document.createElement("mark");
        mark.className = "d2f-highlight";
        const matchedText = match[0] ?? "";
        mark.textContent = matchedText;
        frag.appendChild(mark);
        lastIdx = regex.lastIndex;
        if (matchedText.length === 0)
          break;
        match = regex.exec(val);
      }
      if (lastIdx < val.length) {
        frag.appendChild(document.createTextNode(val.slice(lastIdx)));
      }
      if (textNode.parentNode) {
        textNode.parentNode.replaceChild(frag, textNode);
      }
    });
  }
  function performSearchAndFilter() {
    const rawSearchInput = document.getElementById("search-input");
    const searchInput = rawSearchInput instanceof HTMLInputElement ? rawSearchInput : null;
    const searchCounter = document.getElementById("search-counter");
    const sections = document.querySelectorAll(".section");
    if (sections.length === 0)
      return;
    const query = searchInput ? searchInput.value.trim() : "";
    const queryLower = query.toLowerCase();
    let visibleCount = 0;
    const totalCount = sections.length;
    if (queryLower.length > 0) {
      if (preSearchCollapsedState === null) {
        preSearchCollapsedState = /* @__PURE__ */ new Map();
        sections.forEach((sec) => {
          const body = sec.querySelector(".sb");
          if (body && sec.id) {
            preSearchCollapsedState?.set(sec.id, body.classList.contains("collapsed"));
          }
        });
      }
      const currentMatchedIds = /* @__PURE__ */ new Set();
      sections.forEach((sec) => {
        removeHighlights(sec);
        const passesQuery = (sec.textContent ?? "").toLowerCase().includes(queryLower);
        if (passesQuery) {
          sec.style.display = "";
          visibleCount++;
          if (sec.id)
            currentMatchedIds.add(sec.id);
          const body = sec.querySelector(".sb");
          if (body) {
            highlightTextNodes(body, query);
            if (body.classList.contains("collapsed")) {
              body.classList.remove("collapsed");
              const sh = sec.querySelector(".sh");
              if (sh) {
                sh.setAttribute("aria-expanded", "true");
                const toggler = sh.querySelector(".stog");
                if (toggler)
                  toggler.innerHTML = "&#9660;";
              }
            }
          }
        } else {
          sec.style.display = "none";
        }
      });
      lastMatchedSectionIds = currentMatchedIds;
    } else {
      sections.forEach((sec) => {
        removeHighlights(sec);
        sec.style.display = "";
        visibleCount++;
        const secId = sec.id;
        const body = sec.querySelector(".sb");
        const sh = sec.querySelector(".sh");
        if (body && secId && preSearchCollapsedState !== null) {
          const wasMatched = lastMatchedSectionIds.has(secId);
          const wasCollapsedBeforeSearch = preSearchCollapsedState.get(secId);
          if (!wasMatched && wasCollapsedBeforeSearch === true) {
            body.classList.add("collapsed");
            if (sh) {
              sh.setAttribute("aria-expanded", "false");
              const toggler = sh.querySelector(".stog");
              if (toggler)
                toggler.innerHTML = "&#9650;";
            }
          }
        }
      });
      preSearchCollapsedState = null;
      lastMatchedSectionIds.clear();
      window.d2f.storage.saveState();
    }
    const searchClearBtn = document.getElementById("search-clear-btn");
    if (searchClearBtn) {
      searchClearBtn.classList.toggle("hidden", query.length === 0);
    }
    if (searchCounter) {
      const i18n = window.D2F_I18N ?? {};
      const template = i18n.sections_visible ?? "{visible} / {total} sections visible";
      searchCounter.textContent = template.replace("{visible}", String(visibleCount)).replace("{total}", String(totalCount));
    }
  }
  function toggleSearchToolbar(show) {
    const toolbar = document.getElementById("search-toolbar");
    const toggleBtn = document.getElementById("search-toggle-btn");
    const rawInput = document.getElementById("search-input");
    const input = rawInput instanceof HTMLInputElement ? rawInput : null;
    if (!toolbar)
      return;
    const shouldShow = typeof show === "boolean" ? show : toolbar.classList.contains("hidden");
    if (shouldShow) {
      toolbar.classList.remove("hidden");
      if (toggleBtn)
        toggleBtn.classList.add("active");
      if (input) {
        input.focus();
        input.select();
      }
    } else {
      toolbar.classList.add("hidden");
      if (toggleBtn)
        toggleBtn.classList.remove("active");
      if (input)
        input.value = "";
      performSearchAndFilter();
    }
  }
  function resetSearch() {
    preSearchCollapsedState = null;
    lastMatchedSectionIds.clear();
    const rawSearchInput = document.getElementById("search-input");
    const searchInput = rawSearchInput instanceof HTMLInputElement ? rawSearchInput : null;
    if (searchInput) {
      searchInput.value = "";
    }
    const toolbar = document.getElementById("search-toolbar");
    if (toolbar && !toolbar.classList.contains("hidden")) {
      toggleSearchToolbar(false);
    } else {
      performSearchAndFilter();
    }
  }
  if (typeof window !== "undefined") {
    document.addEventListener("DOMContentLoaded", () => {
      window.d2f.core.registerResetHandler(resetSearch);
      const searchToggleBtn = document.getElementById("search-toggle-btn");
      if (searchToggleBtn) {
        searchToggleBtn.addEventListener("click", () => toggleSearchToolbar());
      }
      const rawSearchInput = document.getElementById("search-input");
      const searchInput = rawSearchInput instanceof HTMLInputElement ? rawSearchInput : null;
      if (searchInput) {
        searchInput.addEventListener("input", () => performSearchAndFilter());
      }
      const searchClearBtn = document.getElementById("search-clear-btn");
      if (searchClearBtn) {
        searchClearBtn.addEventListener("click", () => {
          if (searchInput) {
            searchInput.value = "";
            searchInput.focus();
          }
          performSearchAndFilter();
          window.d2f.storage.saveState();
        });
      }
    });
    document.addEventListener("keydown", (e) => {
      if ((e.ctrlKey || e.metaKey) && (e.key === "k" || e.key === "K")) {
        e.preventDefault();
        toggleSearchToolbar(true);
      } else if (e.key === "Escape") {
        const toolbar = document.getElementById("search-toolbar");
        if (toolbar && !toolbar.classList.contains("hidden")) {
          e.preventDefault();
          toggleSearchToolbar(false);
        }
      }
    });
  }

  // src/core/core.ts
  var resetHandlers = /* @__PURE__ */ new Set();
  function registerResetHandler(handler) {
    resetHandlers.add(handler);
  }
  function resetAll() {
    const i18n = window.D2F_I18N;
    const confirmMsg = i18n?.confirm_reset;
    if (!confirmMsg) {
      console.error("Missing i18n translation key: confirm_reset");
      return;
    }
    if (!confirm(confirmMsg))
      return;
    for (const handler of resetHandlers) {
      try {
        handler();
      } catch (e) {
        console.warn("Failed to execute reset handler", e);
      }
    }
    window.d2f.storage.saveState();
  }
  window.d2f.core = {
    registerResetHandler,
    resetAll
  };
  if (typeof window !== "undefined") {
    window.exportPDF = () => window.d2f.export.export(ExportType.PDF);
    window.saveDocumentState = () => window.d2f.export.export(ExportType.DOCUMENT);
    window.resetAll = () => window.d2f.core.resetAll();
  }
})();
