"use strict";
(() => {
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
})();
