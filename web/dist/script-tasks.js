"use strict";
(() => {
  // src/core/storage.ts
  var saveHandlers = /* @__PURE__ */ new Set();
  var loadHandlers = /* @__PURE__ */ new Set();
  function registerSaveHandler(handler) {
    saveHandlers.add(handler);
  }
  function registerLoadHandler(handler) {
    loadHandlers.add(handler);
  }

  // src/core/core.ts
  var resetHandlers = /* @__PURE__ */ new Set();
  function registerResetHandler(handler) {
    resetHandlers.add(handler);
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
  registerSaveHandler(saveComments);
  registerLoadHandler(loadComments);
  registerResetHandler(resetComments);

  // src/features/tasks.ts
  function isRecord(val) {
    return typeof val === "object" && val !== null && !Array.isArray(val);
  }
  function styleItem(cb) {
    const item = cb.closest(".check-item");
    if (item) {
      item.classList.toggle("checked", cb.checked);
    }
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
      if (!badge)
        return;
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
        if (btnPdf)
          btnPdf.disabled = false;
      } else if (done < total) {
        finishBox.classList.add("pending");
        if (finishIcon)
          finishIcon.innerHTML = "&#x29D6;";
        if (finishTitle)
          finishTitle.textContent = i18n.setup_in_progress ?? "Setup in Progress";
        if (btnPdf)
          btnPdf.disabled = true;
      } else {
        finishBox.classList.add("completed");
        if (finishIcon)
          finishIcon.innerHTML = "&#x2714;";
        if (finishTitle)
          finishTitle.textContent = i18n.setup_completed ?? "Setup Completed";
        if (btnPdf)
          btnPdf.disabled = false;
      }
    }
  }
  function saveTasks() {
    const checks = {};
    document.querySelectorAll('.check-item input[type="checkbox"]').forEach((cb, index) => {
      const key = cb.id || "cb_" + String(index);
      checks[key] = cb.checked;
    });
    const texts = {};
    document.querySelectorAll(".check-item.text-item, .check-item.simple-item").forEach((item, index) => {
      const key = item.id || "txt_" + String(index);
      texts[key] = item.classList.contains("checked");
    });
    return {
      checks,
      texts
    };
  }
  function loadTasks(state) {
    const checksData = state["checks"];
    if (isRecord(checksData)) {
      document.querySelectorAll('.check-item input[type="checkbox"]').forEach((cb, index) => {
        const key = cb.id || "cb_" + String(index);
        const val = checksData[key];
        if (typeof val === "boolean") {
          cb.checked = val;
          styleItem(cb);
        }
      });
    }
    const textsData = state["texts"];
    if (isRecord(textsData)) {
      document.querySelectorAll(".check-item.text-item, .check-item.simple-item").forEach((item, index) => {
        const key = item.id || "txt_" + String(index);
        const val = textsData[key];
        if (typeof val === "boolean") {
          item.classList.toggle("checked", val);
        }
      });
    }
    updateProgress();
    return false;
  }
  function resetTasks() {
    document.querySelectorAll('.check-item input[type="checkbox"]').forEach((cb) => {
      cb.checked = false;
      styleItem(cb);
    });
    document.querySelectorAll(".check-item.text-item, .check-item.simple-item").forEach((item) => {
      item.classList.remove("checked");
    });
    updateProgress();
  }
  registerSaveHandler(saveTasks);
  registerLoadHandler(loadTasks);
  registerResetHandler(resetTasks);
  if (typeof window !== "undefined") {
    document.addEventListener("DOMContentLoaded", () => {
      updateProgress();
    });
  }
})();
