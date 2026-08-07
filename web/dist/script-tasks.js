(() => {
  // ../src/features/tasks/tasks.ts
  (() => {
    function styleItem(cb) {
      const item = cb.closest(".check-item");
      if (item instanceof HTMLElement) {
        item.classList.toggle("checked", cb.checked);
      }
      if (cb.checked) {
        cb.setAttribute("checked", "");
      } else {
        cb.removeAttribute("checked");
      }
    }
    function updateProgress() {
      const i18n = window.d2f.lang.dictionary ?? {};
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
        const parent = pb.parentElement;
        if (parent) {
          parent.setAttribute("aria-valuenow", String(pct));
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
      return { checks };
    }
    function loadTasks(state) {
      const checksData = state["checks"];
      const isRecord = window.d2f.utils.isRecord;
      if (isRecord && isRecord(checksData)) {
        document.querySelectorAll('.check-item input[type="checkbox"]').forEach((cb, index) => {
          const key = cb.id || "cb_" + String(index);
          const val = checksData[key];
          if (typeof val === "boolean") {
            cb.checked = val;
            styleItem(cb);
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
      updateProgress();
    }
    function handleDocumentClick(e) {
      const target = e.target;
      if (!(target instanceof Element))
        return;
      const checkItem = target.closest(".check-item");
      if (checkItem instanceof HTMLElement) {
        if (target.tagName === "A" || target.tagName === "IMG" || target.closest(".item-comment-box") || target.closest(".item-comment-icon") || target.closest(".item-comment-del")) {
          return;
        }
        const cb = checkItem.querySelector('input[type="checkbox"]');
        if (cb) {
          if (target !== cb && !target.closest("label")) {
            cb.checked = !cb.checked;
          }
          styleItem(cb);
          updateProgress();
          window.d2f.storage.saveState();
        }
      }
    }
    window.d2f_tasks = {
      updateProgress,
      save: saveTasks,
      load: loadTasks,
      reset: resetTasks
    };
    window.d2f.storage.registerSaveHandler(saveTasks);
    window.d2f.storage.registerLoadHandler(loadTasks);
    function init() {
      window.d2f.core.registerResetHandler(resetTasks);
      document.addEventListener("click", handleDocumentClick);
      updateProgress();
    }
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", init);
    } else {
      init();
    }
  })();
})();
