"use strict";
(() => {
  // src/core/collapse.ts
  function updateEmptySections() {
    const sections = document.querySelectorAll(".d2f-section, .section");
    sections.forEach((sec) => {
      const sh = sec.querySelector(".sh");
      const body = sec.querySelector(".sb");
      if (sh && body && body.children.length === 0 && body.innerHTML.trim() === "") {
        sh.classList.add("no-toggle");
        sh.removeAttribute("role");
        sh.removeAttribute("tabindex");
        sh.removeAttribute("aria-expanded");
      }
    });
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
      const isCollapsed = body.classList.toggle("collapsed");
      headerElement.setAttribute("aria-expanded", isCollapsed ? "false" : "true");
      const toggler = headerElement.querySelector(".stog");
      if (toggler) {
        toggler.innerHTML = isCollapsed ? "&#9650;" : "&#9660;";
      }
      if (onSave) {
        onSave();
      }
    }
  }

  // src/core/search.ts
  var preSearchCollapsedState = null;
  var lastMatchedSectionIds = /* @__PURE__ */ new Set();
  function removeHighlights(container) {
    if (!container) return;
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
    if (!container || !query) return;
    const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const regex = new RegExp(escaped, "gi");
    const walker = document.createTreeWalker(
      container,
      NodeFilter.SHOW_TEXT,
      {
        acceptNode: (node) => {
          const parent = node.parentNode;
          if (!parent || !(parent instanceof HTMLElement)) return NodeFilter.FILTER_REJECT;
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
      if (!val || !regex.test(val)) return;
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
        if (matchedText.length === 0) break;
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
  function performSearchAndFilter(onSaveState) {
    const rawSearchInput = document.getElementById("search-input");
    const searchInput = rawSearchInput instanceof HTMLInputElement ? rawSearchInput : null;
    const searchCounter = document.getElementById("search-counter");
    const sections = document.querySelectorAll(".d2f-section, .section");
    if (sections.length === 0) return;
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
          if (sec.id) currentMatchedIds.add(sec.id);
          const body = sec.querySelector(".sb");
          if (body) {
            highlightTextNodes(body, query);
            if (body.classList.contains("collapsed")) {
              body.classList.remove("collapsed");
              const sh = sec.querySelector(".sh");
              if (sh) {
                sh.setAttribute("aria-expanded", "true");
                const toggler = sh.querySelector(".stog");
                if (toggler) toggler.innerHTML = "&#9660;";
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
              if (toggler) toggler.innerHTML = "&#9650;";
            }
          }
        }
      });
      preSearchCollapsedState = null;
      lastMatchedSectionIds.clear();
      if (onSaveState) {
        onSaveState();
      }
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
    if (!toolbar) return;
    const shouldShow = typeof show === "boolean" ? show : toolbar.classList.contains("hidden");
    if (shouldShow) {
      toolbar.classList.remove("hidden");
      if (toggleBtn) toggleBtn.classList.add("active");
      if (input) {
        input.focus();
        input.select();
      }
    } else {
      toolbar.classList.add("hidden");
      if (toggleBtn) toggleBtn.classList.remove("active");
      if (input) input.value = "";
      performSearchAndFilter();
    }
  }

  // src/core/storage.ts
  function debounce(func, wait) {
    let timeout;
    return function(...args) {
      if (timeout !== void 0) {
        clearTimeout(timeout);
      }
      timeout = setTimeout(() => func(...args), wait);
    };
  }
  function getStateKey() {
    const docId = window.D2F_DOC_ID ?? "";
    const rawFilename = window.location.pathname.split("/").pop() ?? "index.html";
    const filename = decodeURIComponent(rawFilename);
    return "d2f_state_" + (docId ? docId + "_" : "") + filename;
  }
  function saveState() {
    const state = {};
    document.querySelectorAll('.check-item input[type="checkbox"]').forEach((cb, index) => {
      const key = cb.id || "cb_" + String(index);
      state[key] = cb.checked;
    });
    const textStates = {};
    document.querySelectorAll(".check-item.text-item, .check-item.simple-item").forEach((item, index) => {
      const key = item.id || "txt_" + String(index);
      textStates[key] = item.classList.contains("checked");
    });
    const fields = {};
    document.querySelectorAll("input.persistent-field").forEach((input, index) => {
      const key = input.id || "f_" + String(index);
      fields[key] = input.value;
    });
    const comments = {};
    document.querySelectorAll(".check-item").forEach((item, index) => {
      const input = item.querySelector(".item-comment-input");
      if (input && input.value.trim() !== "") {
        const key = item.id || "item_" + String(index);
        comments[key] = input.value;
      }
    });
    const sections = {};
    document.querySelectorAll(".d2f-section, .section").forEach((sec, index) => {
      const body = sec.querySelector(".sb");
      if (body) {
        const key = sec.id || "sec_" + String(index);
        sections[key] = body.classList.contains("collapsed");
      }
    });
    try {
      const payload = {
        checks: state,
        texts: textStates,
        fields,
        comments,
        sections
      };
      localStorage.setItem(getStateKey(), JSON.stringify(payload));
    } catch (e) {
      console.warn("Failed to save state to localStorage", e);
    }
  }
  var saveStateDebounced = debounce(saveState, 300);
  function syncFieldPair(id1, id2, sourceInput) {
    const raw1 = document.getElementById(id1);
    const raw2 = document.getElementById(id2);
    const el1 = raw1 instanceof HTMLInputElement ? raw1 : null;
    const el2 = raw2 instanceof HTMLInputElement ? raw2 : null;
    if (!el1 || !el2) return;
    if (sourceInput === el1) {
      el2.value = el1.value;
    } else if (sourceInput === el2) {
      el1.value = el2.value;
    } else {
      if (el1.value && !el2.value) el2.value = el1.value;
      else if (el2.value && !el1.value) el1.value = el2.value;
      else if (el1.value) el2.value = el1.value;
    }
  }
  function syncLinkedFields(sourceInput) {
    syncFieldPair("f_info_agent", "f_sign_agent", sourceInput);
    syncFieldPair("f_info_date", "f_sign_date", sourceInput);
  }
  function formatDateFromTemplate(now, template) {
    if (!template || typeof template !== "string") return null;
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
      return tokenMap[match.toUpperCase()] ?? match;
    });
    return hasMatches && !/[A-Za-z]/.test(formatted) ? formatted : null;
  }
  function getTodayFormatted() {
    const i18n = window.D2F_I18N ?? {};
    const now = /* @__PURE__ */ new Date();
    try {
      const fromTemplate = formatDateFromTemplate(now, i18n.date_placeholder);
      if (fromTemplate) return fromTemplate;
    } catch (e) {
      console.warn("Failed to format date", e);
    }
    return now.toLocaleDateString(navigator.language || void 0);
  }
  function checkDateShortcut(input) {
    if (typeof input.value !== "string") return false;
    if (input.value.trim().toLowerCase() === "today") {
      input.value = getTodayFormatted();
      return true;
    }
    return false;
  }
  function parseStateJson(json) {
    try {
      const parsed = JSON.parse(json);
      if (typeof parsed === "object" && parsed !== null) {
        return parsed;
      }
    } catch {
    }
    return null;
  }
  function loadState(styleItemFn, getOrCreateCommentBoxFn) {
    try {
      const raw = localStorage.getItem(getStateKey());
      if (!raw) return;
      const data = parseStateJson(raw);
      if (!data) return;
      if (data.checks) {
        document.querySelectorAll('.check-item input[type="checkbox"]').forEach((cb, index) => {
          const key = cb.id || "cb_" + String(index);
          const val = data.checks?.[key];
          if (val !== void 0) {
            cb.checked = val;
            if (styleItemFn) styleItemFn(cb);
          }
        });
      }
      if (data.texts) {
        document.querySelectorAll(".check-item.text-item, .check-item.simple-item").forEach((item, index) => {
          const key = item.id || "txt_" + String(index);
          const val = data.texts?.[key];
          if (val !== void 0) {
            item.classList.toggle("checked", val);
          }
        });
      }
      if (data.fields) {
        document.querySelectorAll("input.persistent-field").forEach((input, index) => {
          const key = input.id || "f_" + String(index);
          const val = data.fields?.[key];
          if (val !== void 0) {
            input.value = val;
          }
        });
      }
      if (data.comments && getOrCreateCommentBoxFn) {
        document.querySelectorAll(".check-item").forEach((item, index) => {
          const key = item.id || "item_" + String(index);
          const val = data.comments?.[key];
          if (val !== void 0) {
            getOrCreateCommentBoxFn(item, val);
          }
        });
      }
      if (data.sections) {
        document.querySelectorAll(".d2f-section, .section").forEach((sec, index) => {
          const key = sec.id || "sec_" + String(index);
          const shouldCollapse = data.sections?.[key];
          if (shouldCollapse === void 0) return;
          const body = sec.querySelector(".sb");
          const sh = sec.querySelector(".sh");
          if (!body) return;
          body.classList.toggle("collapsed", shouldCollapse);
          if (sh) {
            sh.setAttribute("aria-expanded", shouldCollapse ? "false" : "true");
            const toggler = sh.querySelector(".stog");
            if (toggler) toggler.innerHTML = shouldCollapse ? "&#9650;" : "&#9660;";
          }
        });
      }
      syncLinkedFields();
    } catch (e) {
      console.warn("Failed to load state from localStorage", e);
    }
  }

  // src/features/tasks.ts
  function styleItem(cb) {
    const item = cb.closest(".check-item");
    if (item) {
      item.classList.toggle("checked", cb.checked);
    }
  }
  function autoExpandTextarea(el) {
    if (!el) return;
    el.style.height = "auto";
    el.style.height = String(el.scrollHeight) + "px";
  }
  function getOrCreateCommentBox(checkItem, initialValue) {
    if (!checkItem) return null;
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
    if (!input) return null;
    if (typeof initialValue === "string") {
      input.value = initialValue;
      input.textContent = initialValue;
      input.setAttribute("value", initialValue);
    }
    autoExpandTextarea(input);
    return { box, input };
  }
  function updateProgress() {
    const i18n = window.D2F_I18N ?? {};
    const sections = document.querySelectorAll(".section");
    let total = 0;
    let done = 0;
    const updates = Array.from(sections).map((sec) => {
      const cbs = Array.from(sec.querySelectorAll('input[type="checkbox"]'));
      const badge = sec.querySelector(".sbadge");
      const count = cbs.length;
      const checkedCount = cbs.filter((c) => c.checked).length;
      total += count;
      done += checkedCount;
      return { badge, count, checkedCount };
    });
    updates.forEach(({ badge, count, checkedCount }) => {
      if (!badge) return;
      if (count === 0) {
        badge.textContent = "";
        badge.style.display = "none";
      } else {
        badge.style.display = "";
        badge.textContent = String(checkedCount) + " / " + String(count);
        badge.className = "sbadge" + (checkedCount === count ? " done" : "");
      }
    });
    const pct = total ? Math.round(done / total * 100) : 0;
    const pb = document.getElementById("pb");
    if (pb) {
      pb.style.width = String(pct) + "%";
      if (pb.parentElement) {
        pb.parentElement.setAttribute("aria-valuenow", String(pct));
      }
    }
    const pt = document.getElementById("pt");
    if (pt) {
      const tmpl = i18n.progress_template ?? "{done} of {total} tasks completed ({pct}%)";
      pt.textContent = tmpl.replace("{done}", String(done)).replace("{total}", String(total)).replace("{pct}", String(pct));
    }
    const finishBox = document.getElementById("finish-box");
    const finishIcon = document.getElementById("finish-icon");
    const finishTitle = document.getElementById("finish-title");
    const rawPdf = document.getElementById("btn-pdf");
    const btnPdf = rawPdf instanceof HTMLButtonElement ? rawPdf : null;
    if (finishBox) {
      finishBox.classList.remove("completed", "pending", "no-tasks");
      if (total === 0) {
        finishBox.classList.add("no-tasks");
        if (btnPdf) btnPdf.disabled = false;
      } else if (done < total) {
        finishBox.classList.add("pending");
        if (finishIcon) finishIcon.innerHTML = "&#x29D6;";
        if (finishTitle) finishTitle.textContent = i18n.setup_in_progress ?? "Setup in Progress";
        if (btnPdf) btnPdf.disabled = true;
      } else {
        finishBox.classList.add("completed");
        if (finishIcon) finishIcon.innerHTML = "&#x2714;";
        if (finishTitle) finishTitle.textContent = i18n.setup_completed ?? "Setup Completed";
        if (btnPdf) btnPdf.disabled = false;
      }
    }
  }
  if (typeof window !== "undefined") {
    document.addEventListener("DOMContentLoaded", () => {
      updateProgress();
    });
  }

  // src/core/main.ts
  (() => {
    "use strict";
    document.addEventListener("DOMContentLoaded", () => {
      document.addEventListener("keydown", (e) => {
        if (e.key === "Enter" || e.key === " ") {
          const target = e.target;
          if (target instanceof Element) {
            const sh = target.closest(".sh");
            if (sh && !sh.classList.contains("no-toggle")) {
              e.preventDefault();
              toggleSection(sh, saveState);
            }
          }
        }
      });
      document.addEventListener("click", (e) => {
        const target = e.target;
        if (!(target instanceof Element)) return;
        const sh = target.closest(".sh");
        if (sh && !sh.classList.contains("no-toggle")) {
          toggleSection(sh, saveState);
          return;
        }
        const commentBtn = target.closest(".item-comment-icon");
        if (commentBtn) {
          const checkItem2 = commentBtn.closest(".check-item");
          if (checkItem2) {
            const res = getOrCreateCommentBox(checkItem2);
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
            saveState();
          }
          return;
        }
        const checkItem = target.closest(".check-item");
        if (checkItem) {
          if (target.tagName === "A" || target.tagName === "IMG" || target.closest(".item-comment-box")) {
            return;
          }
          const cb = checkItem.querySelector('input[type="checkbox"]');
          if (cb) {
            if (target !== cb && !target.closest("label")) {
              cb.checked = !cb.checked;
            }
            styleItem(cb);
            updateProgress();
            saveState();
          } else if (checkItem.classList.contains("text-item") || checkItem.classList.contains("simple-item")) {
            checkItem.classList.toggle("checked");
            saveState();
          }
        }
      });
      const linkedIds = ["f_info_agent", "f_sign_agent", "f_info_date", "f_sign_date"];
      const handleInputOrChange = (e) => {
        const target = e.target;
        if (!(target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement)) return;
        if (target.classList.contains("persistent-field")) {
          saveStateDebounced();
        }
        if (target instanceof HTMLTextAreaElement && target.classList.contains("item-comment-input")) {
          target.textContent = target.value;
          target.setAttribute("value", target.value);
          saveStateDebounced();
        }
        if (target instanceof HTMLInputElement) {
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
        }
      };
      document.addEventListener("input", handleInputOrChange);
      document.addEventListener("change", handleInputOrChange);
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
          performSearchAndFilter(saveState);
        });
      }
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
      updateEmptySections();
      loadState(styleItem, getOrCreateCommentBox);
      syncLinkedFields();
      updateProgress();
      performSearchAndFilter();
    });
  })();
})();
