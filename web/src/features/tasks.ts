import { registerSaveHandler, registerLoadHandler } from '../core/storage.js';

function isRecord(val: unknown): val is Record<string, unknown> {
    return typeof val === 'object' && val !== null && !Array.isArray(val);
}

export function styleItem(cb: HTMLInputElement): void {
    const item = cb.closest<HTMLElement>('.check-item');
    if (item) {
        item.classList.toggle('checked', cb.checked);
    }
}

export {
    autoExpandTextarea,
    getOrCreateCommentBox,
    type CommentBoxResult
} from '../core/comments.js';

export function updateProgress(): void {
    const i18n = window.D2F_I18N ?? {};
    const sections = document.querySelectorAll<HTMLElement>('.section');
    let total = 0;
    let done = 0;

    const updates = Array.from(sections).map((sec) => {
        const cbs = Array.from(sec.querySelectorAll<HTMLInputElement>('input[type="checkbox"]'));
        const badge = sec.querySelector<HTMLElement>('.sbadge');
        const count = cbs.length;
        const checkedCount = cbs.filter((c) => c.checked).length;
        total += count;
        done += checkedCount;
        return { badge, count, checkedCount };
    });

    updates.forEach(({ badge, count, checkedCount }) => {
        if (!badge) return;
        if (count === 0) {
            badge.textContent = '';
            badge.style.display = 'none';
        } else {
            badge.style.display = '';
            badge.textContent = String(checkedCount) + ' / ' + String(count);
            badge.className = 'sbadge' + (checkedCount === count ? ' done' : '');
        }
    });

    const pct = total ? Math.round((done / total) * 100) : 0;
    const pb = document.getElementById('pb');
    if (pb) {
        pb.style.width = String(pct) + '%';
        if (pb.parentElement) {
            pb.parentElement.setAttribute('aria-valuenow', String(pct));
        }
    }

    const pt = document.getElementById('pt');
    if (pt) {
        const tmpl = i18n.progress_template ?? '{done} of {total} tasks completed ({pct}%)';
        pt.textContent = tmpl
            .replace('{done}', String(done))
            .replace('{total}', String(total))
            .replace('{pct}', String(pct));
    }

    const finishBox = document.getElementById('finish-box');
    const finishIcon = document.getElementById('finish-icon');
    const finishTitle = document.getElementById('finish-title');
    const rawPdf = document.getElementById('btn-pdf');
    const btnPdf = rawPdf instanceof HTMLButtonElement ? rawPdf : null;

    if (finishBox) {
        finishBox.classList.remove('completed', 'pending', 'no-tasks');
        if (total === 0) {
            finishBox.classList.add('no-tasks');
            if (btnPdf) btnPdf.disabled = false;
        } else if (done < total) {
            finishBox.classList.add('pending');
            if (finishIcon) finishIcon.innerHTML = '&#x29D6;';
            if (finishTitle) finishTitle.textContent = i18n.setup_in_progress ?? 'Setup in Progress';
            if (btnPdf) btnPdf.disabled = true;
        } else {
            finishBox.classList.add('completed');
            if (finishIcon) finishIcon.innerHTML = '&#x2714;';
            if (finishTitle) finishTitle.textContent = i18n.setup_completed ?? 'Setup Completed';
            if (btnPdf) btnPdf.disabled = false;
        }
    }
}

export function saveTasks(): Record<string, unknown> {
    const checks: Record<string, boolean> = {};
    document.querySelectorAll<HTMLInputElement>('.check-item input[type="checkbox"]').forEach((cb, index) => {
        const key = cb.id || ('cb_' + String(index));
        checks[key] = cb.checked;
    });

    const texts: Record<string, boolean> = {};
    document.querySelectorAll<HTMLElement>('.check-item.text-item, .check-item.simple-item').forEach((item, index) => {
        const key = item.id || ('txt_' + String(index));
        texts[key] = item.classList.contains('checked');
    });

    return {
        checks,
        texts
    };
}

export function loadTasks(state: Record<string, unknown>): boolean {
    const checksData = state['checks'];
    if (isRecord(checksData)) {
        document.querySelectorAll<HTMLInputElement>('.check-item input[type="checkbox"]').forEach((cb, index) => {
            const key = cb.id || ('cb_' + String(index));
            const val = checksData[key];
            if (typeof val === 'boolean') {
                cb.checked = val;
                styleItem(cb);
            }
        });
    }

    const textsData = state['texts'];
    if (isRecord(textsData)) {
        document.querySelectorAll<HTMLElement>('.check-item.text-item, .check-item.simple-item').forEach((item, index) => {
            const key = item.id || ('txt_' + String(index));
            const val = textsData[key];
            if (typeof val === 'boolean') {
                item.classList.toggle('checked', val);
            }
        });
    }

    updateProgress();
    return false;
}

registerSaveHandler(saveTasks);
registerLoadHandler(loadTasks);

if (typeof window !== 'undefined') {
    document.addEventListener('DOMContentLoaded', () => {
        updateProgress();
    });
}
